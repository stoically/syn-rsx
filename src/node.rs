use syn::{Expr, Lit};

/// Node in the tree
#[cfg_attr(feature = "syn-extra-traits", derive(Debug))]
pub struct Node {
    pub node_name: String,
    pub node_type: NodeType,
    pub node_value: Option<Expr>,
    pub attributes: Vec<Node>,
    pub child_nodes: Vec<Node>,
}

impl Node {
    /// Returns an `String` if the `node_value` is an `Lit::Str` expression
    pub fn get_value_as_string(&self) -> Option<String> {
        match self.node_value.as_ref().unwrap() {
            Expr::Lit(expr) => match &expr.lit {
                Lit::Str(lit_str) => Some(lit_str.value()),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Type of the Node
#[cfg_attr(feature = "syn-extra-traits", derive(Debug))]
pub enum NodeType {
    /// An HTMLElement tag, with optional childs and attributes.
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
