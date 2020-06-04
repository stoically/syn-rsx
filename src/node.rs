use syn::{punctuated::Punctuated, token::Colon, Expr, ExprPath, Ident, Lit};

use crate::parser::Dash;

/// Node in the tree
#[derive(Debug)]
pub struct Node {
    /// Content depends on the `NodeType`:
    ///
    /// - `Element`: name of the tag
    /// - `Attribute`: key of the attribute
    /// - `Text`: `None`
    /// - `Block`: `None`
    pub name: Option<NodeName>,

    /// Type of the node
    pub node_type: NodeType,

    /// Holds a value according to the `NodeType`
    pub value: Option<Expr>,

    /// Only might have nodes if `NodeType::Element`. Holds every attribute
    /// as `NodeType::Attribute`
    pub attributes: Vec<Node>,

    /// Only might have nodes if `NodeType::Element`
    pub children: Vec<Node>,
}

impl Node {
    /// Returns the `name` path as `String`
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

    /// Returns `value` as `String` if the value is a `Lit::Str` expression
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

/// Type of the Node
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
    /// [Mod style path] containing no path arguments on any of its segments
    ///
    /// [Mod style path]: https://docs.rs/syn/1.0.30/syn/struct.Path.html#method.parse_mod_style
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
