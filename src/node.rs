use syn::{Expr, Lit, Path};

/// Node in the tree
#[derive(Debug)]
pub struct Node {
    /// Content depends on the `NodeType`:
    ///
    /// - `Element`: name of the tag
    /// - `Attribute`: key of the attribute
    /// - `Text`: `None`
    /// - `Block`: `None`
    pub node_name: Option<Path>,

    /// Type of the node
    pub node_type: NodeType,

    /// Content depends on the `NodeType`
    pub node_value: Option<Expr>,

    /// Only might have nodes if `NodeType::Element`. Holds every attribute
    /// as `NodeType::Attribute`
    pub attributes: Vec<Node>,

    /// Only might have nodes if `NodeType::Element`. Holds every child as
    /// `Node`
    pub child_nodes: Vec<Node>,
}

impl Node {
    /// Returns a `String` if the `node_name` `Path` consists of a single ident
    pub fn node_name_as_string(&self) -> Option<String> {
        if let Some(path) = self.node_name.as_ref() {
            path.get_ident().map(|ident| ident.to_string())
        } else {
            None
        }
    }

    /// Returns a `String` if the `node_value` is an `Lit::Str` expression
    pub fn node_value_as_string(&self) -> Option<String> {
        match self.node_value.as_ref() {
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
