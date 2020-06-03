use proc_macro2::TokenTree;
use std::iter;
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, ParseStream, Parser as _},
    punctuated::Punctuated,
    token, Expr, ExprBlock, ExprLit, ExprPath, Ident, Path, PathSegment, Result, Token,
};

use crate::node::*;

struct Tag {
    name: ExprPath,
    attributes: Vec<Node>,
    selfclosing: bool,
}

/// Configures the `Parser` behavior
pub struct ParserConfig {
    /// Whether the returned node tree should be nested or flat. Defaults to `false`
    pub flatten: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self { flatten: false }
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
        while !input.cursor().eof() {
            nodes.append(&mut self.node(input)?)
        }

        Ok(nodes)
    }

    fn node(&self, input: ParseStream) -> Result<Vec<Node>> {
        let node = self
            .text(input)
            .or_else(|_| self.block(input))
            .or_else(|_| self.element(input))?;

        let mut nodes = vec![node];
        if self.config.flatten {
            let mut childs = vec![];
            childs.append(&mut nodes[0].childs);
            nodes.append(&mut childs);
        }

        Ok(nodes)
    }

    fn element(&self, input: ParseStream) -> Result<Node> {
        let fork = input.fork();
        if let Ok(_) = self.tag_close(&input.fork()) {
            return Err(fork.error("close tag has no corresponding open tag"));
        }
        let tag_open = self.tag_open(&fork)?;

        let mut childs = vec![];
        if !tag_open.selfclosing {
            loop {
                if !self.has_childs(&tag_open, &fork)? {
                    break;
                }

                childs.append(&mut self.node(&fork)?);
            }

            self.tag_close(&fork)?;
        }
        input.advance_to(&fork);

        Ok(Node {
            name: Some(tag_open.name),
            value: None,
            node_type: NodeType::Element,
            attributes: tag_open.attributes,
            childs,
        })
    }

    fn has_childs(&self, tag_open: &Tag, input: ParseStream) -> Result<bool> {
        // an empty input at this point means the tag wasn't closed
        if input.is_empty() {
            return Err(input.error("open tag has no corresponding close tag"));
        }

        if let Ok(tag_close_ident) = self.tag_close(&input.fork()) {
            if tag_open.name == tag_close_ident {
                // if the next token is a matching close tag then there are no child nodes
                return Ok(false);
            } else {
                // if the next token is a closing tag with a different name it's an invalid tree
                return Err(input.error("close tag has no corresponding open tag"));
            }
        }

        Ok(true)
    }

    fn tag_open(&self, input: ParseStream) -> Result<Tag> {
        input.parse::<Token![<]>()?;
        let name = self.parse_mod_style_any(input)?;

        let mut attributes: Vec<TokenTree> = vec![];
        let selfclosing = loop {
            if let Ok(selfclosing) = self.tag_open_end(input) {
                break selfclosing;
            }

            attributes.push(input.parse()?);
        };

        let parser = move |input: ParseStream| self.attributes(input);
        let attributes = parser.parse2(attributes.into_iter().collect())?;

        Ok(Tag {
            name,
            attributes,
            selfclosing,
        })
    }

    fn tag_open_end(&self, input: ParseStream) -> Result<bool> {
        let selfclosing = input.parse::<Option<Token![/]>>()?.is_some();
        input.parse::<Token![>]>()?;

        Ok(selfclosing)
    }

    fn tag_close(&self, input: ParseStream) -> Result<ExprPath> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let name = self.parse_mod_style_any(input)?;
        input.parse::<Token![>]>()?;

        Ok(name)
    }

    fn attributes(&self, input: ParseStream) -> Result<Vec<Node>> {
        let mut nodes = vec![];
        if input.is_empty() {
            return Ok(nodes);
        }

        while self.attribute(&input.fork()).is_ok() {
            let (key, value) = self.attribute(input)?;

            nodes.push(Node {
                name: Some(key),
                node_type: NodeType::Attribute,
                value,
                attributes: vec![],
                childs: vec![],
            });

            if input.is_empty() {
                break;
            }
        }

        Ok(nodes)
    }

    fn attribute(&self, input: ParseStream) -> Result<(ExprPath, Option<Expr>)> {
        let key = self.parse_mod_style_any(input)?;
        let eq = input.parse::<Option<Token![=]>>()?;
        let value = if eq.is_some() {
            if input.peek(token::Brace) {
                Some(self.block_expr(input)?)
            } else {
                Some(input.parse()?)
            }
        } else {
            None
        };

        Ok((key, value))
    }

    fn text(&self, input: ParseStream) -> Result<Node> {
        let fork = input.fork();
        let text = fork.parse::<ExprLit>()?.into();
        input.advance_to(&fork);

        Ok(Node {
            name: None,
            value: Some(text),
            node_type: NodeType::Text,
            attributes: vec![],
            childs: vec![],
        })
    }

    fn block(&self, input: ParseStream) -> Result<Node> {
        let fork = input.fork();
        let block = self.block_expr(&fork)?;
        input.advance_to(&fork);

        Ok(Node {
            name: None,
            value: Some(block),
            node_type: NodeType::Block,
            attributes: vec![],
            childs: vec![],
        })
    }

    fn block_expr(&self, input: ParseStream) -> Result<Expr> {
        let parser = move |input: ParseStream| input.parse();
        let group: TokenTree = input.parse()?;
        let block: ExprBlock = parser.parse2(iter::once(group).collect())?;

        Ok(block.into())
    }

    // Modified version of `Path::parse_mod_style` that uses `Ident::peek_any`
    // in order to allow parsing reserved keywords
    //
    // https://docs.rs/syn/1.0.30/src/syn/path.rs.html#388-418
    // TODO: PR upstream
    fn parse_mod_style_any(&self, input: ParseStream) -> Result<ExprPath> {
        let path = Path {
            leading_colon: input.parse()?,
            segments: {
                let mut segments = Punctuated::new();
                loop {
                    if !input.peek(Ident::peek_any)
                        && !input.peek(Token![super])
                        && !input.peek(Token![self])
                        && !input.peek(Token![Self])
                        && !input.peek(Token![crate])
                    {
                        break;
                    }
                    let ident = Ident::parse_any(input)?;
                    segments.push_value(PathSegment::from(ident));
                    if !input.peek(Token![::]) {
                        break;
                    }
                    let punct = input.parse()?;
                    segments.push_punct(punct);
                }
                if segments.is_empty() {
                    return Err(input.error("expected path"));
                } else if segments.trailing_punct() {
                    return Err(input.error("expected path segment"));
                }
                segments
            },
        };

        Ok(ExprPath {
            attrs: vec![],
            qself: None,
            path,
        })
    }
}
