//!
//! Implementation of ToTokens and Spanned for node related structs

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;

use super::{Node, NodeBlock, NodeComment, NodeFragment, NodeName, NodeText};

impl Spanned for Node {
    fn span(&self) -> Span {
        match self {
            Node::Element(node) => node.span(),
            Node::Attribute(node) => node.span(),
            Node::Text(node) => node.span(),
            Node::Comment(node) => node.span(),
            Node::Doctype(node) => node.span(),
            Node::Block(node) => node.span(),
            Node::Fragment(node) => node.span(),
        }
    }
}

impl Spanned for NodeText {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl Spanned for NodeComment {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for NodeFragment {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for NodeBlock {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl ToTokens for NodeName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NodeName::Path(name) => name.to_tokens(tokens),
            NodeName::Punctuated(name) => name.to_tokens(tokens),
            NodeName::Block(name) => name.to_tokens(tokens),
        }
    }
}
