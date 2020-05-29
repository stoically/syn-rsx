pub use syn::{Expr, Lit};

/// Node in the DOM tree
#[cfg_attr(feature = "syn-extra-traits", derive(Debug))]
pub struct Node {
    pub node_name: String,
    pub node_type: NodeType,
    pub node_value: Option<Expr>,
    pub attributes: Vec<Node>,
    pub child_nodes: Vec<Node>,
}

impl Node {
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

#[cfg_attr(feature = "syn-extra-traits", derive(Debug))]
pub enum NodeType {
    Element,
    Attribute,
    Text,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            node_name: "#text".to_owned(),
            node_value: None,
            node_type: NodeType::Text,
            attributes: vec![],
            child_nodes: vec![],
        }
    }
}
