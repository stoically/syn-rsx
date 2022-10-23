//! RSX Parser

use std::vec;

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{
    braced,
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse, ParseStream, Parser as _, Peek},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Colon, Colon2},
    Block, Error, Expr, ExprBlock, ExprLit, ExprPath, Ident, Path, PathSegment, Result, Token,
};

use crate::{config::TransformBlockFn, node::*, punctuation::*, ParserConfig};

/// RSX Parser
pub struct Parser {
    config: ParserConfig,
}

impl Parser {
    /// Create a new parser with the given config.
    pub fn new(config: ParserConfig) -> Parser {
        Parser { config }
    }

    /// Parse a given `syn::ParseStream`.
    pub fn parse(&self, input: ParseStream) -> Result<Vec<Node>> {
        let mut nodes = vec![];
        let mut top_level_nodes = 0;
        while !input.cursor().eof() {
            let parsed_nodes = &mut self.node(input)?;

            if let Some(type_of_top_level_nodes) = &self.config.type_of_top_level_nodes {
                if &parsed_nodes[0].r#type() != type_of_top_level_nodes {
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
            if input.peek2(Token![!]) {
                if input.peek3(Ident) {
                    self.doctype(input)
                } else {
                    self.comment(input)
                }
            } else if input.peek2(Token![>]) {
                self.fragment(input)
            } else {
                self.element(input)
            }
        } else if input.peek(Brace) {
            self.block(input)
        } else {
            self.text(input)
        }?;

        let mut nodes = vec![node];
        if self.config.flat_tree {
            let mut children = nodes[0]
                .children_mut()
                .map(|children| children.drain(..))
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            nodes.append(&mut children);
        }

        Ok(nodes)
    }

    fn text(&self, input: ParseStream) -> Result<Node> {
        let value = input.parse::<ExprLit>()?.into();

        Ok(Node::Text(NodeText { value }))
    }

    fn block(&self, input: ParseStream) -> Result<Node> {
        let value = if let Some(transform_fn) = &self.config.transform_block {
            self.block_transform(input, transform_fn)?
        } else {
            self.block_expr(input)?
        }
        .into();

        Ok(Node::Block(NodeBlock { value }))
    }

    fn block_transform(&self, input: ParseStream, transform_fn: &TransformBlockFn) -> Result<Expr> {
        let parser = move |block_content: ParseStream| {
            let forked_block_content = block_content.fork();

            match transform_fn(&forked_block_content) {
                Ok(transformed_tokens) => match transformed_tokens {
                    Some(tokens) => {
                        let parser = move |input: ParseStream| {
                            Ok(self.block_content_to_block(input, block_content.span()))
                        };
                        let transformed_content = parser.parse2(tokens)?;
                        block_content.advance_to(&forked_block_content);
                        transformed_content
                    }
                    None => self.block_content_to_block(block_content, block_content.span()),
                },
                Err(error) => Err(error),
            }
        };

        input.step(|cursor| {
            let (tree, next) = cursor
                .token_tree()
                .ok_or_else(|| cursor.error("unexpected: no TokenTree found"))?;

            match tree {
                TokenTree::Group(block_group) => Ok((parser.parse2(block_group.stream())?, next)),
                _ => Err(cursor.error("unexpected: no Group in TokenTree found")),
            }
        })
    }

    fn block_content_to_block(&self, input: ParseStream, span: Span) -> Result<Expr> {
        Ok(ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token: Brace { span },
                stmts: Block::parse_within(input)?,
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

        if self.tag_close(&input.fork()).is_ok() {
            return Err(fork.error("close tag has no corresponding open tag"));
        }
        let (name, attributes, self_closing) = self.tag_open(fork)?;

        let mut children = vec![];
        if !self_closing {
            loop {
                if !self.element_has_children(&name, fork)? {
                    break;
                }

                children.append(&mut self.node(fork)?);
            }

            self.tag_close(fork)?;
        }
        input.advance_to(fork);

        Ok(Node::Element(NodeElement {
            name,
            attributes,
            children,
        }))
    }

    fn element_has_children(&self, tag_open_name: &NodeName, input: ParseStream) -> Result<bool> {
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
            let value = self.block_expr(fork)?.into();
            input.advance_to(fork);

            Ok(Node::Block(NodeBlock { value }))
        } else {
            let key = self.node_name(fork)?;
            let eq = fork.parse::<Option<Token![=]>>()?;
            let value = if eq.is_some() {
                if fork.is_empty() {
                    return Err(Error::new(key.span(), "missing attribute value"));
                }

                if fork.peek(Brace) {
                    Some(NodeValueExpr::new(self.block_expr(fork)?))
                } else {
                    Some(NodeValueExpr::new(fork.parse()?))
                }
            } else {
                None
            };
            input.advance_to(fork);

            Ok(Node::Attribute(NodeAttribute { key, value }))
        }
    }

    fn doctype(&self, input: ParseStream) -> Result<Node> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![!]>()?;
        let ident = input.parse::<Ident>()?;
        if ident.to_string().to_lowercase() != "doctype" {
            return Err(input.error("expected Doctype"));
        }
        let doctype = input.parse::<Ident>()?;
        input.parse::<Token![>]>()?;

        let mut segments = Punctuated::new();
        segments.push_value(PathSegment::from(doctype));
        let value = NodeValueExpr::new(
            ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }
            .into(),
        );

        Ok(Node::Doctype(NodeDoctype { value }))
    }

    fn comment(&self, input: ParseStream) -> Result<Node> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![!]>()?;
        input.parse::<Token![-]>()?;
        input.parse::<Token![-]>()?;
        let value = NodeValueExpr::new(input.parse::<ExprLit>()?.into());
        input.parse::<Token![-]>()?;
        input.parse::<Token![-]>()?;
        input.parse::<Token![>]>()?;

        Ok(Node::Comment(NodeComment { value }))
    }

    fn fragment(&self, input: ParseStream) -> Result<Node> {
        self.fragment_open(input)?;

        let mut children = vec![];
        loop {
            if input.is_empty() {
                return Err(input.error("unexpected end of input"));
            }

            if self.fragment_close(&input.fork()).is_ok() {
                self.fragment_close(input)?;
                break;
            }

            children.append(&mut self.node(input)?);
        }

        Ok(Node::Fragment(NodeFragment { children }))
    }

    fn fragment_open(&self, input: ParseStream) -> Result<()> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![>]>()?;

        Ok(())
    }

    fn fragment_close(&self, input: ParseStream) -> Result<()> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        input.parse::<Token![>]>()?;

        Ok(())
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
                .map(NodeName::Colon)
        } else if input.peek2(Dash) {
            self.node_name_punctuated_ident::<Dash, fn(_) -> Dash, Ident>(input, Dash)
                .map(NodeName::Dash)
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
            Err(input.error("invalid tag name or attribute key"))
        }
    }

    // we can't replace this with [`Punctuated::parse_separated_nonempty`] since
    // that doesn't support reserved keywords. might be worth to consider a PR
    // upstream
    //
    // [`Punctuated::parse_separated_nonempty`]: https://docs.rs/syn/1.0.58/syn/punctuated/struct.Punctuated.html#method.parse_separated_nonempty
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
