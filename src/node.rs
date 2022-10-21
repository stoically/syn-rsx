//! Tree of nodes.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::{convert::TryFrom, fmt, ops::Deref};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Colon, Expr, ExprBlock, ExprLit, ExprPath,
    Ident, Lit,
};

use crate::{punctuation::Dash, Error};

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
            Self::Fragment(NodeFragment { children })
            | Self::Element(NodeElement { children, .. }) => Some(children),
            _ => None,
        }
    }

    /// Get mutable node children.
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Self::Fragment(NodeFragment { children })
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
}

impl fmt::Display for NodeElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeElement")
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
}

impl fmt::Display for NodeAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeAttribute")
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
}

impl fmt::Display for NodeDoctype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeDoctype")
    }
}

/// Fragement node.
///
/// Fragment: `<></>`
#[derive(Debug)]
pub struct NodeFragment {
    /// Children of the fragment node.
    pub children: Vec<Node>,
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

/// Name of the node
#[derive(Debug)]
pub enum NodeName {
    /// A plain identifier like `div` is a path of length 1, e.g. `<div />`. Can
    /// be separated by double colons, e.g. `<foo::bar />`.
    Path(ExprPath),

    /// Name separated by dashes, e.g. `<div data-foo="bar" />`.
    Dash(Punctuated<Ident, Dash>),

    /// Name separated by colons, e.g. `<div on:click={foo} />`.
    Colon(Punctuated<Ident, Colon>),

    /// Arbitrary rust code in braced `{}` blocks.
    Block(Expr),
}

impl NodeName {
    /// Returns the `Span` of this `NodeName`.
    pub fn span(&self) -> Span {
        match self {
            NodeName::Path(name) => name.span(),
            NodeName::Dash(name) => name.span(),
            NodeName::Colon(name) => name.span(),
            NodeName::Block(name) => name.span(),
        }
    }
}

impl TryFrom<&NodeName> for ExprBlock {
    type Error = Error;

    fn try_from(node: &NodeName) -> Result<Self, Self::Error> {
        match node {
            NodeName::Block(Expr::Block(expr)) => Ok(expr.to_owned()),
            _ => Err(Error::TryFrom(
                "NodeName does not match NodeName::Block(Expr::Block(_))".into(),
            )),
        }
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
            Self::Block(this) => match other {
                Self::Block(other) => this == other,
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
            NodeName::Block(name) => name.to_tokens(tokens),
        }
    }
}

impl fmt::Display for NodeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NodeName::Path(expr) => path_to_string(expr),
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
                NodeName::Block(_) => String::from("{}"),
            }
        )
    }
}

/// Smart pointer to `syn::Expr`.
#[derive(Debug)]
pub struct NodeValueExpr {
    expr: Expr,
}

impl NodeValueExpr {
    /// Create a `NodeValueExpr`.
    pub fn new(expr: Expr) -> Self {
        Self { expr }
    }
}

impl AsRef<Expr> for NodeValueExpr {
    fn as_ref(&self) -> &Expr {
        &self.expr
    }
}

impl Deref for NodeValueExpr {
    type Target = Expr;

    fn deref(&self) -> &Self::Target {
        &self.expr
    }
}

impl From<Expr> for NodeValueExpr {
    fn from(expr: Expr) -> Self {
        Self { expr }
    }
}

impl From<ExprLit> for NodeValueExpr {
    fn from(expr: ExprLit) -> Self {
        Self { expr: expr.into() }
    }
}

impl From<ExprBlock> for NodeValueExpr {
    fn from(expr: ExprBlock) -> Self {
        Self { expr: expr.into() }
    }
}

impl From<NodeValueExpr> for Expr {
    fn from(value: NodeValueExpr) -> Self {
        value.expr
    }
}

impl<'a> From<&'a NodeValueExpr> for &'a Expr {
    fn from(value: &'a NodeValueExpr) -> Self {
        &value.expr
    }
}

impl TryFrom<NodeValueExpr> for ExprBlock {
    type Error = Error;

    fn try_from(value: NodeValueExpr) -> Result<Self, Self::Error> {
        if let Expr::Block(block) = value.expr {
            Ok(block)
        } else {
            Err(Error::TryFrom(
                "NodeValueExpr does not match Expr::Block(_)".into(),
            ))
        }
    }
}

impl TryFrom<NodeValueExpr> for ExprLit {
    type Error = Error;

    fn try_from(value: NodeValueExpr) -> Result<Self, Self::Error> {
        if let Expr::Lit(lit) = value.expr {
            Ok(lit)
        } else {
            Err(Error::TryFrom(
                "NodeValueExpr does not match Expr::Lit(_)".into(),
            ))
        }
    }
}

impl TryFrom<&NodeValueExpr> for String {
    type Error = Error;

    fn try_from(value: &NodeValueExpr) -> Result<Self, Self::Error> {
        match &value.expr {
            Expr::Lit(expr) => match &expr.lit {
                Lit::Str(lit_str) => Some(lit_str.value()),
                _ => None,
            },
            Expr::Path(expr) => Some(path_to_string(&expr)),
            _ => None,
        }
        .ok_or_else(|| {
            Error::TryFrom(
                "NodeValueExpr does not match Expr::Lit(Lit::Str(_)) or Expr::Path(_)".into(),
            )
        })
    }
}

pub fn path_to_string(expr: &ExprPath) -> String {
    expr.path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}
