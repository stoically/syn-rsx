use syn::{Expr, ExprPath, Lit};

/// Node in the tree
#[derive(Debug)]
pub struct Node {
    /// Content depends on the `NodeType`:
    ///
    /// - `Element`: name of the tag
    /// - `Attribute`: key of the attribute
    /// - `Text`: `None`
    /// - `Block`: `None`
    pub name: Option<ExprPath>,

    /// Type of the node
    pub node_type: NodeType,

    /// Holds a value according to the `NodeType`
    pub value: Option<Expr>,

    /// Only might have nodes if `NodeType::Element`. Holds every attribute
    /// as `NodeType::Attribute`
    pub attributes: Vec<Node>,

    /// Only might have nodes if `NodeType::Element`. Holds every child as
    /// `Node`
    pub children: Vec<Node>,
}

impl Node {
    /// Returns `node_name` path as `String`
    pub fn name_as_string(&self) -> Option<String> {
        match self.name.as_ref() {
            Some(ExprPath { path, .. }) => Some(
                path.segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<String>>()
                    .join("::"),
            ),
            _ => None,
        }
    }

    /// Returns `node_value` as `String` if the value is a `Lit::Str` expression
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
