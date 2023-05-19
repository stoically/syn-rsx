//! Tree of nodes.

use std::fmt;

use atoms::{tokens, FragmentClose, FragmentOpen};
use proc_macro2::Ident;
use syn::{ExprPath, LitStr, Token};

pub mod atoms;
mod attribute;
mod node_name;
mod node_value;
pub mod parse;
mod raw_text;

pub use attribute::{KeyedAttribute, NodeAttribute};
pub use node_name::NodeName;
pub use node_value::NodeBlock;

pub use self::raw_text::RawText;

/// Node types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Element,
    Text,
    Comment,
    Doctype,
    Block,
    Fragment,
    RawText,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Element => "NodeType::Element",
                Self::Text => "NodeType::Text",
                Self::RawText => "NodeType::RawText",
                Self::Comment => "NodeType::Comment",
                Self::Doctype => "NodeType::Doctype",
                Self::Block => "NodeType::Block",
                Self::Fragment => "NodeType::Fragment",
            }
        )
    }
}

/// Node in the tree.
#[derive(Clone, Debug, syn_derive::ToTokens)]
pub enum Node {
    Comment(NodeComment),
    Doctype(NodeDoctype),
    Fragment(NodeFragment),
    Element(NodeElement),
    Block(NodeBlock),
    Text(NodeText),
    RawText(RawText),
}

impl Node {
    pub fn flatten(mut self) -> Vec<Self> {
        let children = self
            .children_mut()
            .map(|children| children.drain(..))
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        std::iter::once(self)
            .chain(children.into_iter().flat_map(Self::flatten))
            .collect()
    }
    /// Get the type of the node.
    pub fn r#type(&self) -> NodeType {
        match &self {
            Self::Element(_) => NodeType::Element,
            Self::Text(_) => NodeType::Text,
            Self::Comment(_) => NodeType::Comment,
            Self::Doctype(_) => NodeType::Element,
            Self::Block(_) => NodeType::Block,
            Self::Fragment(_) => NodeType::Fragment,
            Self::RawText(_) => NodeType::RawText,
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
#[derive(Clone, Debug, syn_derive::ToTokens)]
pub struct NodeElement {
    pub open_tag: atoms::OpenTag,
    #[to_tokens(parse::to_tokens_array)]
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
/// Quoted text. Unquoted can be found in `RawText`.
#[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
pub struct NodeText {
    /// The text value.
    pub value: LitStr,
}

impl NodeText {
    /// Returns value of inner LitStr
    pub fn value_string(&self) -> String {
        self.value.value()
    }
}

/// Comment node.
///
/// Comment: `<!-- "comment" -->`, currently has the same restrictions as
/// `Text` (comment needs to be quoted).
#[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
pub struct NodeComment {
    pub token_start: tokens::ComStart,
    /// The comment value.
    pub value: LitStr,
    pub token_end: tokens::ComEnd,
}
/// Doctype node.
///
/// Doctype declaration: `<!DOCTYPE html>` (case insensitive), `html` is the
/// node value in this case.
/// Usually doctype only contaim html, but also can contain arbitrary DOCTYPE
/// legacy string, or "obsolete permitted DOCTYPE string", therewhy value is
/// RawText.
#[derive(Clone, Debug, syn_derive::ToTokens)]
pub struct NodeDoctype {
    pub token_start: tokens::DocStart,
    /// "doctype"
    pub token_doctype: Ident,
    /// The doctype value.
    pub value: RawText,
    pub token_end: Token![>],
}

/// Fragement node.
///
/// Fragment: `<></>`
#[derive(Clone, Debug, syn_derive::ToTokens)]
pub struct NodeFragment {
    /// Open fragment token
    pub tag_open: FragmentOpen,
    /// Children of the fragment node.
    #[to_tokens(parse::to_tokens_array)]
    pub children: Vec<Node>,
    /// Close fragment token
    pub tag_close: Option<FragmentClose>,
}

fn path_to_string(expr: &ExprPath) -> String {
    expr.path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}
