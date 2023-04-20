//! RSX Parser

use std::vec;

use proc_macro2::{Punct, Span, TokenStream, TokenTree, Group};
use syn::{
    braced,
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse, ParseStream, Parser as _, Peek},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Colon, PathSep},
    Block, Error, Expr, ExprBlock, ExprLit, ExprPath, Ident, Path, PathSegment, Result, Token,
};

use crate::{config::TransformBlockFn, node::*, punctuation::*, ParserConfig};

/// RSX Parser
pub struct Parser {
    config: ParserConfig,
}

impl Parser {
    /// Create a new parser with the given [`ParserConfig`].
    pub fn new(config: ParserConfig) -> Parser {
        Parser { config }
    }

    /// Parse a given [`ParseStream`].
    pub fn parse(&self, input: ParseStream) -> Result<Vec<Node>> {
        let mut nodes = vec![];
        let mut top_level_nodes = 0;
        while !input.cursor().eof() {
            let mut parsed_nodes = self.node(input)?;

            if let Some(type_of_top_level_nodes) = &self.config.type_of_top_level_nodes {
                if &parsed_nodes[0].r#type() != type_of_top_level_nodes {
                    return Err(input.error(format!(
                        "top level nodes need to be of type {}",
                        type_of_top_level_nodes
                    )));
                }
            }

            top_level_nodes += 1;
            nodes.append(&mut parsed_nodes);
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

    /// Parse the next [`Node`] in the tree.
    ///
    /// To improve performance it peeks the next 1-3 tokens and calls the
    /// according node parser function depending on that.
    fn node(&self, input: ParseStream) -> Result<Vec<Node>> {
        let mut node = if input.peek(Token![<]) {
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

        if self.config.flat_tree {
            let mut children = node
                .children_mut()
                .map(|children| children.drain(..))
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            let mut nodes = vec![node];
            nodes.append(&mut children);
            Ok(nodes)
        } else {
            Ok(vec![node])
        }
    }

    /// Parse the stream as [`Node::Text`].
    fn text(&self, input: ParseStream) -> Result<Node> {
        let value = input.parse::<ExprLit>()?.into();

        Ok(Node::Text(NodeText { value }))
    }

    /// Parse the stream as [`Node::Block`].
    fn block(&self, input: ParseStream) -> Result<Node> {
        let value = if let Some(transform_fn) = &self.config.transform_block {
            self.block_transform(input, transform_fn)?
        } else {
            self.block_expr(input)?
        }
        .into();

        Ok(Node::Block(NodeBlock { value }))
    }

    /// Replace the next [`TokenTree::Group`] in the given parse stream with a
    /// token stream returned by a user callback, or parse as original block if
    /// no token stream is returned.
    fn block_transform(&self, input: ParseStream, transform_fn: &TransformBlockFn) -> Result<Expr> {
        let parser = move |block_content: ParseStream| {
            let forked_block_content = block_content.fork();

            match transform_fn(&forked_block_content) {
                Ok(transformed_tokens) => match transformed_tokens {
                    Some(tokens) => {
                        let parser =
                            move |input: ParseStream| Ok(self.block_content_to_block(input, block_content.span()));
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

    /// Parse the given stream and span as [`Expr::Block`].
    fn block_content_to_block(&self, input: ParseStream, span: Span) -> Result<Expr> {
        let mut delim_span = Group::new(proc_macro2::Delimiter::None, TokenStream::new());
        delim_span.set_span(span);
        Ok(ExprBlock {
            attrs: vec![],
            label: None,
            block: Block {
                brace_token: Brace { span: delim_span.delim_span() },
                stmts: Block::parse_within(input)?,
            },
        }
        .into())
    }

    /// Parse the given stream as [`Expr::Block`].
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

    /// Parse the given stream as [`NodeElement`].
    fn element(&self, input: ParseStream) -> Result<Node> {
        let fork = &input.fork();

        if self.tag_close(&input.fork()).is_ok() {
            return Err(fork.error("close tag has no corresponding open tag"));
        }
        let (name, attributes, self_closing, mut span) = self.tag_open(fork)?;

        let mut children = vec![];
        if !self_closing {
            loop {
                if !self.element_has_children(&name, fork)? {
                    break;
                }

                children.append(&mut self.node(fork)?);
            }

            let (_, closing_span) = self.tag_close(fork)?;
            span = span.join(closing_span).unwrap_or(span);
        };

        input.advance_to(fork);
        Ok(Node::Element(NodeElement {
            name,
            attributes,
            children,
            span,
        }))
    }

    /// Check whether the next token in the stream is a closing tag to decide
    /// whether the node element has children.
    fn element_has_children(&self, tag_open_name: &NodeName, input: ParseStream) -> Result<bool> {
        // An empty input at this point means the tag wasn't closed.
        if input.is_empty() {
            return Err(Error::new(
                tag_open_name.span(),
                "open tag has no corresponding close tag and is not self-closing",
            ));
        }

        if let Ok((tag_close_name, _)) = self.tag_close(&input.fork()) {
            if tag_open_name == &tag_close_name {
                // If the next token is a matching close tag then there are no child nodes.
                return Ok(false);
            } else {
                // If the next token is a closing tag with a different name it's an invalid
                // tree.
                return Err(input.error("close tag has no corresponding open tag"));
            }
        }

        Ok(true)
    }

    /// Parse the stream as opening or self-closing tag and extract its
    /// attributes.
    fn tag_open(&self, input: ParseStream) -> Result<(NodeName, Vec<Node>, bool, Span)> {
        let span_start = input.span();
        input.parse::<Token![<]>()?;
        let name = self.node_name(input)?;

        let mut attributes = TokenStream::new();
        let (self_closing, span_end) = loop {
            if let Ok(end) = self.tag_open_end(input) {
                break end;
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

        let span = span_start.join(span_end).unwrap_or(name.span());

        Ok((name, attributes, self_closing, span))
    }

    /// Check whether an element tag ended or is self-closing.
    fn tag_open_end(&self, input: ParseStream) -> Result<(bool, Span)> {
        let span_start = input.span();
        let self_closing = input.parse::<Option<Token![/]>>()?.is_some();
        let span_end = input.span();
        input.parse::<Token![>]>()?;
        let span = span_start.join(span_end).unwrap_or(span_start);

        Ok((self_closing, span))
    }

    /// Parse a closing tag and return its [`NodeName`] and [Span]
    fn tag_close(&self, input: ParseStream) -> Result<(NodeName, Span)> {
        let start_span = input.span();
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let name = self.node_name(input)?;
        let span_end = input.span();
        input.parse::<Token![>]>()?;

        let span = start_span.join(span_end).unwrap_or(span_end);
        Ok((name, span))
    }

    /// Parse the stream as vector of attributes.
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

    /// Parse the stream as [`Node::Attribute`].
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
            let span = if let Some(ref val) = value {
                key.span().join(val.span()).unwrap_or(key.span())
            } else {
                key.span()
            };
            Ok(Node::Attribute(NodeAttribute { key, value, span }))
        }
    }

    /// Parse the stream as [`Node::Doctype`].
    fn doctype(&self, input: ParseStream) -> Result<Node> {
        let span_start = input.span();
        input.parse::<Token![<]>()?;
        input.parse::<Token![!]>()?;
        let ident = input.parse::<Ident>()?;
        if ident.to_string().to_lowercase() != "doctype" {
            return Err(input.error("expected Doctype"));
        }
        let doctype = input.parse::<Ident>()?;
        let span_end = input.span();
        let doctype_span = doctype.span();
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

        let span = span_start.join(span_end).unwrap_or(doctype_span);
        Ok(Node::Doctype(NodeDoctype { value, span }))
    }

    /// Parse the stream as [`Node::Comment`].
    fn comment(&self, input: ParseStream) -> Result<Node> {
        let span_start = input.span();
        input.parse::<Token![<]>()?;
        input.parse::<Token![!]>()?;
        input.parse::<Token![-]>()?;
        input.parse::<Token![-]>()?;
        let value = NodeValueExpr::new(input.parse::<ExprLit>()?.into());
        input.parse::<Token![-]>()?;
        input.parse::<Token![-]>()?;
        let span_end = input.span();
        input.parse::<Token![>]>()?;

        let span = span_start.join(span_end).unwrap_or(value.span());
        Ok(Node::Comment(NodeComment { value, span }))
    }

    /// Parse the stream as [`Node::Fragement`].
    fn fragment(&self, input: ParseStream) -> Result<Node> {
        let mut span = self.fragment_open(input)?;

        let mut children = vec![];
        loop {
            if input.is_empty() {
                return Err(input.error("unexpected end of input"));
            }

            let fork = input.fork();
            if let Ok(closing_span) = self.fragment_close(&fork) {
                input.advance_to(&fork);
                span = span.join(closing_span).unwrap_or(span);
                break;
            }

            children.append(&mut self.node(input)?);
        }

        Ok(Node::Fragment(NodeFragment { children, span }))
    }

    /// Parse the stream as opening fragment tag.
    fn fragment_open(&self, input: ParseStream) -> Result<Span> {
        let span_start = input.span();
        input.parse::<Token![<]>()?;
        let span_end = input.span();
        input.parse::<Token![>]>()?;

        let span = span_start.join(span_end).unwrap_or(span_start);
        Ok(span)
    }

    /// Parse the stream as closing fragment tag.
    fn fragment_close(&self, input: ParseStream) -> Result<Span> {
        let span_start = input.span();
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let span_end = input.span();
        input.parse::<Token![>]>()?;

        let span = span_start.join(span_end).unwrap_or(span_end);

        Ok(span)
    }

    /// Parse the stream as [`NodeName`].
    fn node_name(&self, input: ParseStream) -> Result<NodeName> {
        if input.peek2(PathSep) {
            self.node_name_punctuated_ident::<PathSep, fn(_) -> PathSep, PathSegment>(
                input, PathSep,
            )
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
        } else if input.peek2(Colon) || input.peek2(Dash) {
            self.node_name_punctuated_ident_with_alternate::<Punct, fn(_) -> Colon, fn(_) -> Dash, Ident>(
                input, Colon, Dash,
            )
            .map(NodeName::Punctuated)
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

    /// Parse the stream as punctuated idents.
    ///
    /// We can't replace this with [`Punctuated::parse_separated_nonempty`]
    /// since that doesn't support reserved keywords. Might be worth to
    /// consider a PR upstream.
    ///
    /// [`Punctuated::parse_separated_nonempty`]: https://docs.rs/syn/1.0.58/syn/punctuated/struct.Punctuated.html#method.parse_separated_nonempty
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

    /// Parse the stream as punctuated idents, with two possible punctuations
    /// available
    fn node_name_punctuated_ident_with_alternate<T: Parse, F: Peek, G: Peek, X: From<Ident>>(
        &self,
        input: ParseStream,
        punct: F,
        alternate_punct: G,
    ) -> Result<Punctuated<X, T>> {
        let fork = &input.fork();
        let mut segments = Punctuated::<X, T>::new();

        while !fork.is_empty() && fork.peek(Ident::peek_any) {
            let ident = Ident::parse_any(fork)?;
            segments.push_value(ident.clone().into());

            if fork.peek(punct) || fork.peek(alternate_punct) {
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
