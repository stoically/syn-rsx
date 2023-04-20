use proc_macro2::Span;

use crate::{NodeBlock, NodeName, NodeValueExpr};

///
/// Element attribute with fixed key.
///
/// Example:
/// "key=value" // attribute with ident as value
/// "key" // attribute without value
#[derive(Debug)]
pub struct KeyedAttribute {
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

///
/// Element attribute with that is computed fron rust code.
///
/// Example:
/// {"some-fixed-key"} // attribute without value that is computed from string
#[derive(Debug)]
pub struct DynAttribute {
    pub block: NodeBlock,
}

/// Sum type for Dyn and Keyed attributes.
///
/// Attributes is stored in opening tags.
#[derive(Debug)]
pub enum NodeAttribute {
    Attribute(KeyedAttribute),
    Block(DynAttribute),
}
