//! RSX Parser

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{
    braced,
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse, ParseStream, Parser as _, Peek},
    punctuated::Punctuated,
    token::{Brace, Colon, Colon2},
    Block, Error, Expr, ExprBlock, ExprLit, ExprPath, Ident, Path, PathSegment, Result, Token,
};

use crate::{node::*, punctuation::*};

/// Configures the `Parser` behavior
#[derive(Default)]
pub struct ParserConfig {
    flat_tree: bool,
    number_of_top_level_nodes: Option<usize>,
    type_of_top_level_nodes: Option<NodeType>,
    transform_block: Option<Box<dyn Fn(ParseStream) -> Result<Option<TokenStream>>>>,
}

impl ParserConfig {
    /// Create new `ParserConfig` with default config
    pub fn new() -> ParserConfig {
        ParserConfig::default()
    }

    /// Return flat tree instead of nested tree
    pub fn flat_tree(mut self) -> Self {
        self.flat_tree = true;
        self
    }

    /// Exact number of required top level nodes
    pub fn number_of_top_level_nodes(mut self, number: usize) -> Self {
        self.number_of_top_level_nodes = Some(number);
        self
    }

    /// Enforce the `NodeType` of top level nodes
    pub fn type_of_top_level_nodes(mut self, node_type: NodeType) -> Self {
        self.type_of_top_level_nodes = Some(node_type);
        self
    }

    /// Transforms the `value` of all `NodeType::Block`s with the given closure
    /// callback. The given `ParseStream` is the content of the block.
    ///
    /// When `Some(TokenStream)` is returned, the `TokenStream` is parsed as
    /// Rust block content. The `ParseStream` must be completely consumed in
    /// this case (no tokens left).
    ///
    /// If `None` is returned, the `ParseStream` is parsed as Rust block
    /// content. The `ParseStream` isn't forked, so partial parsing inside the
    /// transform callback will break this mechanism - fork if you want to avoid
    /// breaking.
    ///
    /// An example usage might be a custom syntax inside blocks which isn't
    /// valid Rust. The given example simply translates the `%` character into
    /// the string `percent`
    ///
    /// ```rust
    /// use quote::quote;
    /// use syn::Token;
    /// use syn_rsx::{parse2_with_config, ParserConfig};
    ///
    /// let tokens = quote! {
    ///     <div>{%}</div>
    /// };
    ///
    /// let config = ParserConfig::new().transform_block(|input| {
    ///     input.parse::<Token![%]>()?;
    ///     Ok(Some(quote! { "percent" }))
    /// });
    ///
    /// parse2_with_config(tokens, config).unwrap();
    /// ```
    pub fn transform_block<F>(mut self, callback: F) -> Self
    where
        F: Fn(ParseStream) -> Result<Option<TokenStream>> + 'static,
    {
        self.transform_block = Some(Box::new(callback));
        self
    }
}

/// RSX Parser
pub struct Parser {
    config: ParserConfig,
}

impl Parser {
    /// Create a new parser with the given config
    pub fn new(config: ParserConfig) -> Parser {
        Parser { config }
    }

    /// Parse a given `syn::ParseStream`
    pub fn parse(&self, input: ParseStream) -> Result<Vec<Node>> {
        let mut nodes = vec![];
        let mut top_level_nodes = 0;
        while !input.cursor().eof() {
            let parsed_nodes = &mut self.node(input)?;

            if let Some(type_of_top_level_nodes) = &self.config.type_of_top_level_nodes {
                if &parsed_nodes[0].node_type != type_of_top_level_nodes {
                    return Err(input.error(format!(
                        "top level nodes need to be of type {}",
                        type_of_top_level_nodes
                    )));
                }
            }

            nodes.append(parsed_nodes);
            top_level_nodes += 1;
        }

        if let Some(number_of_top_level_nodes) = &self.config.number_of_top_level_nodes {
            if &top_level_nodes != number_of_top_level_nodes {
                return Err(input.error(format!(
                    "saw {} top level nodes but exactly {} are required",
                    top_level_nodes, number_of_top_level_nodes
                )));
            }
        }

        Ok(nodes)
    }

    fn node(&self, input: ParseStream) -> Result<Vec<Node>> {
        let node = if input.peek(Token![<]) {
            self.element(input)
        } else if input.peek(Brace) {
            self.block(input)
        } else {
            self.text(input)
        }?;

        let mut nodes = vec![node];
        if self.config.flat_tree {
            let mut children = vec![];
            children.append(&mut nodes[0].children);
            nodes.append(&mut children);
        }

        Ok(nodes)
    }

    fn text(&self, input: ParseStream) -> Result<Node> {
        let text = input.parse::<ExprLit>()?.into();

        Ok(Node {
            name: None,
            value: Some(text),
            node_type: NodeType::Text,
            attributes: vec![],
            children: vec![],
        })
    }

    fn block(&self, input: ParseStream) -> Result<Node> {
        let block = if self.config.transform_block.is_some() {
            self.block_transform(input)?
        } else {
            self.block_expr(input)?
        };

        Ok(Node {
            name: None,
            value: Some(block),
            node_type: NodeType::Block,
            attributes: vec![],
            children: vec![],
        })
    }

    fn block_transform(&self, input: ParseStream) -> Result<Expr> {
        let transform_block = self.config.transform_block.as_ref().unwrap();

        input.step(|cursor| {
            if let Some((tree, next)) = cursor.token_tree() {
                match tree {
                    TokenTree::Group(block_group) => {
                        let block_span = block_group.span();
                        let parser = move |block_content: ParseStream| match transform_block(
                            block_content,
                        ) {
                            Ok(transformed_tokens) => match transformed_tokens {
                                Some(tokens) => {
                                    let parser = move |input: ParseStream| {
                                        Ok(self.block_content_to_block(input, block_span))
                                    };
                                    parser.parse2(tokens)?
                                }
                                None => self.block_content_to_block(block_content, block_span),
                            },
                            Err(error) => Err(error),
                        };
                        Ok((parser.parse2(block_group.stream())?, next))
                    }
                    _ => Err(cursor.error("unexpected: no Group in TokenTree found")),
                }
            } else {
                Err(cursor.error("unexpected: no TokenTree found"))
            }
        })
    }

    fn block_content_to_block(&self, input: ParseStream, span: Span) -> Result<Expr> {
        Ok(ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token: Brace { span },
                stmts: Block::parse_within(&input)?,
            },
        }
        .into())
    }

    fn block_expr(&self, input: ParseStream) -> Result<Expr> {
        let fork = input.fork();
        let content;
        let brace_token = braced!(content in fork);
        let block = ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token,
                stmts: Block::parse_within(&content)?,
            },
        };
        input.advance_to(&fork);

        Ok(block.into())
    }

    fn element(&self, input: ParseStream) -> Result<Node> {
        let fork = &input.fork();
        if let Ok(_) = self.tag_close(&input.fork()) {
            return Err(fork.error("close tag has no corresponding open tag"));
        }
        let (name, attributes, self_closing) = self.tag_open(fork)?;

        let mut children = vec![];
        if !self_closing {
            loop {
                if !self.has_children(&name, fork)? {
                    break;
                }

                children.append(&mut self.node(fork)?);
            }

            self.tag_close(fork)?;
        }
        input.advance_to(fork);

        Ok(Node {
            name: Some(name),
            value: None,
            node_type: NodeType::Element,
            attributes,
            children,
        })
    }

    fn has_children(&self, tag_open_name: &NodeName, input: ParseStream) -> Result<bool> {
        // an empty input at this point means the tag wasn't closed
        if input.is_empty() {
            return Err(Error::new(
                tag_open_name.span(),
                "open tag has no corresponding close tag and is not self-closing",
            ));
        }

        if let Ok(tag_close_name) = self.tag_close(&input.fork()) {
            if tag_open_name == &tag_close_name {
                // if the next token is a matching close tag then there are no child nodes
                return Ok(false);
            } else {
                // if the next token is a closing tag with a different name it's an invalid tree
                return Err(input.error("close tag has no corresponding open tag"));
            }
        }

        Ok(true)
    }

    fn tag_open(&self, input: ParseStream) -> Result<(NodeName, Vec<Node>, bool)> {
        input.parse::<Token![<]>()?;
        let name = self.node_name(input)?;

        let mut attributes = TokenStream::new();
        let self_closing = loop {
            if let Ok(self_closing) = self.tag_open_end(input) {
                break self_closing;
            }

            if input.is_empty() {
                return Err(input.error("expected closing caret >"));
            }

            let next: TokenTree = input.parse()?;
            attributes.extend(Some(next));
        };

        let attributes = if !attributes.is_empty() {
            let parser = move |input: ParseStream| self.attributes(input);
            parser.parse2(attributes)?
        } else {
            vec![]
        };

        Ok((name, attributes, self_closing))
    }

    fn tag_open_end(&self, input: ParseStream) -> Result<bool> {
        let self_closing = input.parse::<Option<Token![/]>>()?.is_some();
        input.parse::<Token![>]>()?;

        Ok(self_closing)
    }

    fn tag_close(&self, input: ParseStream) -> Result<NodeName> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let name = self.node_name(input)?;
        input.parse::<Token![>]>()?;

        Ok(name)
    }

    fn attributes(&self, input: ParseStream) -> Result<Vec<Node>> {
        let mut nodes = vec![];

        loop {
            if input.is_empty() {
                break;
            }

            nodes.push(self.attribute(input)?);
        }

        Ok(nodes)
    }

    fn attribute(&self, input: ParseStream) -> Result<Node> {
        let fork = &input.fork();
        if fork.peek(Brace) {
            let value = Some(self.block_expr(fork)?);
            input.advance_to(fork);

            Ok(Node {
                name: None,
                node_type: NodeType::Block,
                value,
                attributes: vec![],
                children: vec![],
            })
        } else {
            let key = self.node_name(fork)?;
            let eq = fork.parse::<Option<Token![=]>>()?;
            let value = if eq.is_some() {
                if fork.is_empty() {
                    return Err(Error::new(key.span(), "missing attribute value"));
                }

                if fork.peek(Brace) {
                    Some(self.block_expr(fork)?)
                } else {
                    Some(fork.parse()?)
                }
            } else {
                None
            };
            input.advance_to(fork);

            Ok(Node {
                name: Some(key),
                node_type: NodeType::Attribute,
                value,
                attributes: vec![],
                children: vec![],
            })
        }
    }

    fn node_name(&self, input: ParseStream) -> Result<NodeName> {
        if input.peek2(Colon2) {
            self.node_name_punctuated_ident::<Colon2, fn(_) -> Colon2, PathSegment>(input, Colon2)
                .map(|segments| {
                    NodeName::Path(ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments,
                        },
                    })
                })
        } else if input.peek2(Colon) {
            self.node_name_punctuated_ident::<Colon, fn(_) -> Colon, Ident>(input, Colon)
                .map(|ok| NodeName::Colon(ok))
        } else if input.peek2(Dash) {
            self.node_name_punctuated_ident::<Dash, fn(_) -> Dash, Ident>(input, Dash)
                .map(|ok| NodeName::Dash(ok))
        } else if input.peek(Brace) {
            let fork = &input.fork();
            let value = self.block_expr(fork)?;
            input.advance_to(fork);
            Ok(NodeName::Block(value))
        } else if input.peek(Ident::peek_any) {
            let mut segments = Punctuated::new();
            let ident = Ident::parse_any(input)?;
            segments.push_value(PathSegment::from(ident));
            Ok(NodeName::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }))
        } else {
            return Err(input.error("invalid tag name or attribute key"));
        }
    }

    fn node_name_punctuated_ident<T: Parse, F: Peek, X: From<Ident>>(
        &self,
        input: ParseStream,
        punct: F,
    ) -> Result<Punctuated<X, T>> {
        let fork = &input.fork();
        let mut segments = Punctuated::<X, T>::new();

        while !fork.is_empty() && fork.peek(Ident::peek_any) {
            let ident = Ident::parse_any(fork)?;
            segments.push_value(ident.clone().into());

            if fork.peek(punct) {
                segments.push_punct(fork.parse()?);
            } else {
                break;
            }
        }

        if segments.len() > 1 {
            input.advance_to(fork);
            Ok(segments)
        } else {
            Err(fork.error("expected punctuated node name"))
        }
    }
}
