use proc_macro2::{TokenStream, TokenTree};
use syn::{
    braced,
    ext::IdentExt,
    parse::{ParseStream, Parser as _},
    token::Brace,
    Expr, ExprLit, Ident, Result, Token,
};

use crate::node::*;

pub struct Tag {
    pub ident: Ident,
    pub attributes: Vec<Node>,
    pub selfclosing: bool,
}

/// Configures the `Parser` behavior
pub struct ParserConfig {
    /// Whether the returned node tree should be nested or flat
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
        let mut node = if self.text(&input.fork()).is_ok() {
            self.text(input)
        } else {
            self.element(input)
        }?;

        let nodes = if self.config.flatten {
            // TODO there has to be a more elegant way to do this
            let mut childs = vec![];
            childs.append(&mut node.child_nodes);
            let mut nodes = vec![node];
            nodes.append(&mut childs);
            nodes
        } else {
            vec![node]
        };

        Ok(nodes)
    }

    fn element(&self, input: ParseStream) -> Result<Node> {
        if let Ok(next_close_ident) = self.next_close_ident(&input.fork()) {
            return Err(syn::Error::new(
                next_close_ident.span(),
                "close tag has no corresponding open tag",
            ));
        }

        let tag_open = self.tag_open(input)?;

        let mut child_nodes = vec![];

        if !tag_open.selfclosing {
            loop {
                if !self.has_child_nodes(&tag_open, &input)? {
                    break;
                }

                child_nodes.append(&mut self.node(input)?);
            }

            self.next_close_ident(input)?;
        }

        Ok(Node {
            node_name: tag_open.ident.to_string(),
            node_value: None,
            node_type: NodeType::Element,
            attributes: tag_open.attributes,
            child_nodes,
        })
    }

    fn has_child_nodes(&self, tag_open: &Tag, input: &ParseStream) -> Result<bool> {
        // an empty input at this point means the tag wasn't closed
        if input.is_empty() {
            return Err(syn::Error::new(
                tag_open.ident.span(),
                "open tag has no corresponding close tag",
            ));
        }

        if let Ok(next_close_ident) = self.next_close_ident(&input.fork()) {
            if tag_open.ident == next_close_ident {
                // if the next token is a matching close tag then there are no child nodes
                return Ok(false);
            } else {
                // if the next token is a closing tag with a different name it's an invalid tree
                return Err(syn::Error::new(
                    next_close_ident.span(),
                    "close tag has no corresponding open tag",
                ));
            }
        }

        Ok(true)
    }

    fn attributes(&self, input: ParseStream) -> Result<Vec<Node>> {
        let mut nodes = vec![];
        if input.is_empty() {
            return Ok(nodes);
        }

        while let Ok(_) = self.attribute(&input.fork()) {
            let (key, value) = self.attribute(input)?;

            nodes.push(Node {
                node_name: key,
                node_type: NodeType::Attribute,
                node_value: value,
                attributes: vec![],
                child_nodes: vec![],
            });

            if input.is_empty() {
                break;
            }
        }

        Ok(nodes)
    }

    fn attribute(&self, input: ParseStream) -> Result<(String, Option<Expr>)> {
        let key = input.call(Ident::parse_any)?.to_string();
        let eq = input.parse::<Option<Token![=]>>()?;
        let value = if eq.is_some() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok((key, value))
    }

    fn tag_open(&self, input: ParseStream) -> Result<Tag> {
        input.parse::<Token![<]>()?;
        let ident = input.parse()?;

        let mut attributes: Vec<TokenTree> = vec![];

        let selfclosing = loop {
            if let Ok(selfclosing) = self.tag_closed(input) {
                break selfclosing;
            }

            attributes.push(input.parse()?);
        };

        let attributes: TokenStream = attributes.into_iter().collect();
        let parser = move |input: ParseStream| self.attributes(input);
        let attributes = parser.parse2(attributes)?;

        Ok(Tag {
            ident,
            attributes,
            selfclosing,
        })
    }

    fn tag_closed(&self, input: ParseStream) -> Result<bool> {
        let selfclosing = input.parse::<Option<Token![/]>>()?.is_some();
        input.parse::<Token![>]>()?;

        Ok(selfclosing)
    }

    fn next_close_ident(&self, input: ParseStream) -> Result<Ident> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let ident = input.parse()?;
        input.parse::<Token![>]>()?;

        Ok(ident)
    }

    fn text(&self, input: ParseStream) -> Result<Node> {
        let value = if input.peek(Brace) {
            // special-case {expr} because `Group` has no `Parse` implementation
            let group;
            braced!(group in input);
            group.parse()?
        } else {
            input.parse::<ExprLit>()?.into()
        };

        Ok(Node {
            node_name: "".to_owned(),
            node_value: Some(value),
            node_type: NodeType::Text,
            attributes: vec![],
            child_nodes: vec![],
        })
    }
}
