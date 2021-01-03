//! Tree of nodes

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::fmt;
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Colon, Expr, ExprBlock, ExprPath, Ident, Lit,
};

use crate::punctuation::Dash;

/// Node in the tree
#[derive(Debug)]
pub struct Node {
    /// Name according to the `NodeType`
    ///
    /// - Element: Name of the element
    /// - Attribute: Key of the element attribute
    /// - Text: `None`
    /// - Block: `None`
    pub name: Option<NodeName>,

    /// Type of the nodes
    pub node_type: NodeType,

    /// Value according to the `NodeType`
    ///
    /// - Element: `None`
    /// - Attribute: Any valid `syn::Expr`
    /// - Text: `syn::Expr::Lit`
    /// - Doctype: `syn::Expr::Lit`
    /// - Block: `syn::Expr::Block`
    pub value: Option<Expr>,

    /// Attributes of `NodeType::Element` are `NodeType::Attribute` or `NodeType::Block`
    pub attributes: Vec<Node>,

    /// Children of `NodeType::Element` can be everything except `NodeType::Attribute`
    pub children: Vec<Node>,
}

impl Node {
    /// Returns `String` if `name` is `Some`
    pub fn name_as_string(&self) -> Option<String> {
        match self.name.as_ref() {
            Some(name) => Some(name.to_string()),
            None => None,
        }
    }

    /// Returns `Span` if `name` is `Some`
    pub fn name_span(&self) -> Option<Span> {
        match self.name.as_ref() {
            Some(name) => Some(name.span()),
            None => None,
        }
    }

    /// Returns `String` if `value` is a `Lit::Str` expression
    pub fn value_as_string(&self) -> Option<String> {
        match self.value.as_ref() {
            Some(Expr::Lit(expr)) => match &expr.lit {
                Lit::Str(lit_str) => Some(lit_str.value()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns `ExprBlock` if `value` is a `Expr::Block` expression
    pub fn value_as_block(&self) -> Option<ExprBlock> {
        match self.value.as_ref() {
            Some(Expr::Block(expr)) => Some(expr.to_owned()),
            _ => None,
        }
    }
}

// https://developer.mozilla.org/en-US/docs/Web/API/Node/nodeType
/// Type of the node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    /// A HTMLElement tag, with optional children and attributes.
    /// Potentially selfclosing. Any tag name is valid.
    Element,

    /// Attributes of opening tags. Every attribute is itself a node.
    Attribute,

    /// Quoted text. It's [planned to support unquoted text] as well
    /// using span start and end, but that currently only works
    /// with nightly rust
    ///
    /// [planned to support unquoted text]: https://github.com/stoically/syn-rsx/issues/2
    Text,


    /// Doctype declaration: `<!DOCTYPE html>` (case insensitive)
    Doctype,
    /// Arbitrary rust code in braced `{}` blocks
    Block,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Element => "NodeType::Element",
                Self::Attribute => "NodeType::Attribute",
                Self::Text => "NodeType::Text",
                Self::Doctype => "NodeType::Doctype",
                Self::Block => "NodeType::Block",
            }
        )
    }
}

/// Name of the node
#[derive(Debug)]
pub enum NodeName {
    /// A plain identifier like `div` is a path of length 1, e.g. `<div />`. Can
    /// be separated by double colons, e.g. `<foo::bar />`
    Path(ExprPath),

    /// Name separated by dashes, e.g. `<div data-foo="bar" />`
    Dash(Punctuated<Ident, Dash>),

    /// Name separated by colons, e.g. `<div on:click={foo} />`
    Colon(Punctuated<Ident, Colon>),
}

impl NodeName {
    /// Returns the `Span` of this `NodeName`
    pub fn span(&self) -> Span {
        match self {
            NodeName::Path(name) => name.span(),
            NodeName::Dash(name) => name.span(),
            NodeName::Colon(name) => name.span(),
        }
    }
}

impl fmt::Display for NodeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NodeName::Path(expr) => expr
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<String>>()
                    .join("::"),
                NodeName::Dash(name) => name
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join("-"),
                NodeName::Colon(name) => name
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join(":"),
            }
        )
    }
}

impl PartialEq for NodeName {
    fn eq(&self, other: &NodeName) -> bool {
        match self {
            Self::Path(this) => match other {
                Self::Path(other) => this == other,
                _ => false,
            },
            Self::Dash(this) => match other {
                Self::Dash(other) => this == other,
                _ => false,
            },
            Self::Colon(this) => match other {
                Self::Colon(other) => this == other,
                _ => false,
            },
        }
    }
}

impl ToTokens for NodeName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NodeName::Path(name) => name.to_tokens(tokens),
            NodeName::Dash(name) => name.to_tokens(tokens),
            NodeName::Colon(name) => name.to_tokens(tokens),
        }
    }
}
