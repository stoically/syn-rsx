//! Tree of nodes.

use std::fmt;

use proc_macro2::Span;
use syn::{spanned::Spanned, ExprPath};

mod node_name;
mod node_value;
mod tokens;

pub use node_name::NodeName;
pub use node_value::NodeValueExpr;

/// Node types.
#[derive(Debug, PartialEq, Eq)]
pub enum NodeType {
    Element,
    Attribute,
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
                Self::Attribute => "NodeType::Attribute",
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
#[derive(Debug)]
pub enum Node {
    Element(NodeElement),
    Attribute(NodeAttribute),
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
            Self::Attribute(_) => NodeType::Attribute,
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

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Element(_) => "Node::Element",
                Self::Attribute(_) => "Node::Attribute",
                Self::Text(_) => "Node::Text",
                Self::Comment(_) => "Node::Comment",
                Self::Doctype(_) => "Node::Doctype",
                Self::Block(_) => "Node::Block",
                Self::Fragment(_) => "Node::Fragment",
            }
        )
    }
}

/// Element node.
///
/// A HTMLElement tag, with optional children and attributes.
/// Potentially selfclosing. Any tag name is valid.
#[derive(Debug)]
pub struct NodeElement {
    /// Name of the element.
    pub name: NodeName,
    /// Attributes of the element node.
    pub attributes: Vec<Node>,
    /// Children of the element node.
    pub children: Vec<Node>,
    /// Source span of the element for error reporting.
    ///
    /// Note: This should cover the entire node in nightly, but is a "close
    /// enough" approximation in stable until [Span::join] is stabilized.
    pub span: Span,
}

impl fmt::Display for NodeElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeElement")
    }
}

impl Spanned for NodeElement {
    fn span(&self) -> Span {
        self.span
    }
}

/// Attribute node.
///
/// Attributes of opening tags. Every attribute is itself a node.
#[derive(Debug)]
pub struct NodeAttribute {
    /// Key of the element attribute.
    pub key: NodeName,
    /// Value of the element attribute.
    pub value: Option<NodeValueExpr>,
    /// Source span of the attribute for error reporting.
    ///
    /// Note: This should cover the entire node in nightly, but is a "close
    /// enough" approximation in stable until [Span::join] is stabilized.
    pub span: Span,
}

impl fmt::Display for NodeAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeAttribute")
    }
}

impl Spanned for NodeAttribute {
    fn span(&self) -> Span {
        self.span
    }
}

/// Text node.
///
/// Quoted text. It's [planned to support unquoted text] as well
/// using span start and end, but that currently only works
/// with nightly rust.
///
/// [planned to support unquoted text]: https://github.com/stoically/syn-rsx/issues/2
#[derive(Debug)]
pub struct NodeText {
    /// The text value.
    pub value: NodeValueExpr,
}

impl fmt::Display for NodeText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeText")
    }
}

/// Comment node.
///
/// Comment: `<!-- "comment" -->`, currently has the same restrictions as
/// `Text` (comment needs to be quoted).
#[derive(Debug)]
pub struct NodeComment {
    /// The comment value.
    pub value: NodeValueExpr,
    /// Source span of the comment for error reporting.
    ///
    /// Note: This should cover the entire node in nightly, but is a "close
    /// enough" approximation in stable until [Span::join] is stabilized.
    pub span: Span,
}

impl fmt::Display for NodeComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeComment")
    }
}

/// Doctype node.
///
/// Doctype declaration: `<!DOCTYPE html>` (case insensitive), `html` is the
/// node value in this case.
#[derive(Debug)]
pub struct NodeDoctype {
    /// The doctype value.
    pub value: NodeValueExpr,
    /// Source span of the doctype node for error reporting.
    ///
    /// Note: This should cover the entire node in nightly, but is a "close
    /// enough" approximation in stable until [Span::join] is stabilized.
    pub span: Span,
}

impl fmt::Display for NodeDoctype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeDoctype")
    }
}

impl Spanned for NodeDoctype {
    fn span(&self) -> Span {
        self.span
    }
}

/// Fragement node.
///
/// Fragment: `<></>`
#[derive(Debug)]
pub struct NodeFragment {
    /// Children of the fragment node.
    pub children: Vec<Node>,
    /// Source span of the fragment for error reporting.
    ///
    /// Note: This should cover the entire node in nightly, but is a "close
    /// enough" approximation in stable until [Span::join] is stabilized.
    pub span: Span,
}

impl fmt::Display for NodeFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeFragment")
    }
}

/// Block node.
///
/// Arbitrary rust code in braced `{}` blocks.
#[derive(Debug)]
pub struct NodeBlock {
    /// The block value..
    pub value: NodeValueExpr,
}

impl fmt::Display for NodeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeBlock")
    }
}

fn path_to_string(expr: &ExprPath) -> String {
    expr.path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}
