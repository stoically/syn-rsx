//!
//! Implementation of ToTokens and Spanned for node related structs

use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};

use crate::{NodeValueExpr, NodeElement, NodeAttribute};

use super::{Node, NodeBlock, NodeComment, NodeDoctype, NodeFragment, NodeName, NodeText};


impl ToTokens for NodeValueExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let obj = self.as_ref();
        obj.to_tokens(tokens)
    }
}

impl ToTokens for NodeElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let name = &self.name;
        let attributes = &self.attributes;
        let children = &self.children;

        // self closing
        if children.is_empty() {
            tokens.extend(quote_spanned!(self.span => 
            <#name #(#attributes)* /> ))
        } else {
            tokens.extend(quote_spanned!(self.span => 
            <#name #(#attributes)*> #(#children)* </#name> ))
        }
    }
}

impl ToTokens for NodeAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let key = &self.key;
        let value = &self.value;

        // self closing
        if let Some(value) = value{
            tokens.extend(quote_spanned!(self.span => 
            #key = #value ))
        } else {
            tokens.extend(quote_spanned!(self.span => 
            #key ))
        }
    }
}

impl ToTokens for NodeBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens)
    }
}

impl ToTokens for NodeComment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = &self.value;
        tokens.extend(quote_spanned!(self.span=> <!-- #value -->))
    }
}

impl ToTokens for NodeDoctype {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let value = &self.value;
        tokens.extend(quote_spanned!(self.span=> <! #value >))
    }
}

impl ToTokens for NodeFragment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let childrens = &self.children;
        tokens.extend(quote_spanned!(self.span => <> #(#childrens)* </>))
    }
}

impl ToTokens for NodeText {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens);
    }
}


impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Node::Attribute(a) => a.to_tokens(tokens),
            Node::Block(b) => b.to_tokens(tokens),
            Node::Comment(c) => c.to_tokens(tokens),
            Node::Doctype(d) => d.to_tokens(tokens),
            Node::Fragment(f) => f.to_tokens(tokens),
            Node::Element(e) => e.to_tokens(tokens),
            Node::Text(t) => t.to_tokens(tokens),
        }
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
