//! Tree of nodes.

use std::fmt;

use proc_macro2::Ident;
use syn::{ExprPath, Token};

mod atoms;
mod attribute;
mod node_name;
mod node_value;
pub mod tokens;

pub use attribute::{DynAttribute, KeyedAttribute, NodeAttribute};
pub use node_name::NodeName;
pub use node_value::{NodeBlock, NodeValueExpr};

pub use self::atoms::*;

/// Node types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Element,
    Text,
    Comment,
    Doctype,
    Block,
    Fragment,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Element => "NodeType::Element",
                Self::Text => "NodeType::Text",
                Self::Comment => "NodeType::Comment",
                Self::Doctype => "NodeType::Doctype",
                Self::Block => "NodeType::Block",
                Self::Fragment => "NodeType::Fragment",
            }
        )
    }
}

/// Node in the tree.
#[derive(Debug, syn_derive::ToTokens)]
pub enum Node {
    Element(NodeElement),
    Text(NodeText),
    Comment(NodeComment),
    Doctype(NodeDoctype),
    Block(NodeBlock),
    Fragment(NodeFragment),
}

impl Node {
    /// Get the type of the node.
    pub fn r#type(&self) -> NodeType {
        match &self {
            Self::Element(_) => NodeType::Element,
            Self::Text(_) => NodeType::Text,
            Self::Comment(_) => NodeType::Comment,
            Self::Doctype(_) => NodeType::Element,
            Self::Block(_) => NodeType::Block,
            Self::Fragment(_) => NodeType::Fragment,
        }
    }

    /// Get node children.
    pub fn children(&self) -> Option<&Vec<Node>> {
        match self {
            Self::Fragment(NodeFragment { children, .. })
            | Self::Element(NodeElement { children, .. }) => Some(children),
            _ => None,
        }
    }

    /// Get mutable node children.
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Self::Fragment(NodeFragment { children, .. })
            | Self::Element(NodeElement { children, .. }) => Some(children),
            _ => None,
        }
    }
}

/// Element node.
///
/// A HTMLElement tag, with optional children and attributes.
/// Potentially selfclosing. Any tag name is valid.
#[derive(Debug, syn_derive::ToTokens)]
pub struct NodeElement {
    pub open_tag: atoms::OpenTag,
    #[to_tokens(tokens::to_tokens_array)]
    pub children: Vec<Node>,
    pub close_tag: Option<atoms::CloseTag>,
}

impl NodeElement {
    pub fn name(&self) -> &NodeName {
        &self.open_tag.name
    }
    pub fn attributes(&self) -> &[NodeAttribute] {
        &self.open_tag.attributes
    }
}

/// Text node.
///
/// Quoted text. It's [planned to support unquoted text] as well
/// using span start and end, but that currently only works
/// with nightly rust.
///
/// [planned to support unquoted text]: https://github.com/stoically/syn-rsx/issues/2
#[derive(Debug, syn_derive::ToTokens)]
pub struct NodeText {
    /// The text value.
    pub value: NodeValueExpr,
}

/// Comment node.
///
/// Comment: `<!-- "comment" -->`, currently has the same restrictions as
/// `Text` (comment needs to be quoted).
#[derive(Debug, syn_derive::ToTokens)]
pub struct NodeComment {
    pub token_start: token::ComStart,
    /// The comment value.
    pub value: NodeValueExpr,
    pub token_end: token::ComEnd,
}
/// Doctype node.
///
/// Doctype declaration: `<!DOCTYPE html>` (case insensitive), `html` is the
/// node value in this case.
#[derive(Debug, syn_derive::ToTokens)]
pub struct NodeDoctype {
    pub token_start: token::DocStart,
    /// "doctype"
    pub token_doctype: Ident,
    /// The doctype value.
    pub value: NodeValueExpr,
    pub token_end: Token![>],
}

/// Fragement node.
///
/// Fragment: `<></>`
#[derive(Debug, syn_derive::ToTokens)]
pub struct NodeFragment {
    /// Open fragment token
    pub tag_open: FragmentOpen,
    /// Children of the fragment node.
    #[to_tokens(tokens::to_tokens_array)]
    pub children: Vec<Node>,
    /// Close fragment token
    pub tag_close: FragmentClose,
}

fn path_to_string(expr: &ExprPath) -> String {
    expr.path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}
