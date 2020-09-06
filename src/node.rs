//! Tree of nodes

use proc_macro2::Span;
use syn::{punctuated::Punctuated, spanned::Spanned, token::Colon, Expr, ExprPath, Ident, Lit};

use crate::punctuation::Dash;

/// Node in the tree
#[derive(Debug)]
pub struct Node {
    /// Name according to the `NodeType`:
    ///
    /// - `Element`: Name of the element
    /// - `Attribute`: Key of the element attribute
    /// - `Text`: `None`
    /// - `Block`: `None`
    pub name: Option<NodeName>,

    /// Type of the nodes
    pub node_type: NodeType,

    /// Value according to the `NodeType`:
    ///
    /// - `Element`: `None`
    /// - `Attribute`: Any valid `syn::Expr`
    /// - `Text`: `syn::Expr::Lit`
    /// - `Block`: `syn::Expr::Block`
    pub value: Option<Expr>,

    /// Has nodes if `NodeType::Element`. Every attribute is
    /// `NodeType::Attribute`
    pub attributes: Vec<Node>,

    /// Has nodes if `NodeType::Element`
    pub children: Vec<Node>,
}

impl Node {
    /// Returns `name` as `String` if it's `Some`
    pub fn name_as_string(&self) -> Option<String> {
        match self.name.as_ref() {
            Some(NodeName::Path(expr)) => Some(
                expr.path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<String>>()
                    .join("::"),
            ),
            Some(NodeName::Dash(name)) => Some(
                name.iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join("-"),
            ),
            Some(NodeName::Colon(name)) => Some(
                name.iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join(":"),
            ),
            None => None,
        }
    }

    /// Returns the `name`'s `Span` if it's `Some`
    pub fn name_span(&self) -> Option<Span> {
        match self.name.as_ref() {
            Some(NodeName::Path(name)) => Some(name.span()),
            Some(NodeName::Dash(name)) => Some(name.span()),
            Some(NodeName::Colon(name)) => Some(name.span()),
            None => None,
        }
    }

    /// Returns `value` as `String` if it's a `Lit::Str` expression
    pub fn value_as_string(&self) -> Option<String> {
        match self.value.as_ref() {
            Some(Expr::Lit(expr)) => match &expr.lit {
                Lit::Str(lit_str) => Some(lit_str.value()),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Type of the node
#[derive(Debug)]
pub enum NodeType {
    /// A HTMLElement tag, with optional children and attributes.
    /// Potentially selfclosing. Any tag name is valid.
    Element,

    /// Attributes of opening tags. Every attribute is itself a node.
    Attribute,

    /// Quoted text. It's planned to support unquoted text as well
    /// using span start and end, but that currently only works
    /// with nightly rust
    Text,

    /// Arbitrary rust code in braced `{}` blocks
    Block,
}

/// Name of the node
#[derive(Debug)]
pub enum NodeName {
    /// [Mod style path] containing no path arguments on any of its segments. A
    /// plain identifier like `x` is a path of length 1.
    ///
    /// [Mod style path]:
    /// https://docs.rs/syn/1.0.30/syn/struct.Path.html#method.parse_mod_style
    Path(ExprPath),

    /// Name separated by dashes, e.g. `<div data-foo="bar" />`
    Dash(Punctuated<Ident, Dash>),

    /// Name separated by colons, e.g. `<div on:click={foo} />`
    Colon(Punctuated<Ident, Colon>),
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
