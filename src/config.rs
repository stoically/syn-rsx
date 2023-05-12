use std::{collections::HashSet, fmt::Debug, rc::Rc};

use proc_macro2::TokenStream;
use syn::{parse::ParseStream, Result};

use crate::NodeType;

pub type TransformBlockFn = dyn Fn(ParseStream) -> Result<Option<TokenStream>>;

/// Configures the `Parser` behavior
#[derive(Default, Clone)]
pub struct ParserConfig {
    pub(crate) flat_tree: bool,
    pub(crate) number_of_top_level_nodes: Option<usize>,
    pub(crate) type_of_top_level_nodes: Option<NodeType>,
    pub(crate) transform_block: Option<Rc<TransformBlockFn>>,
    pub(crate) recover_block: bool,
    pub(crate) always_self_closed_elements: HashSet<&'static str>,
    pub(crate) raw_text_elements: HashSet<&'static str>,
}

impl Debug for ParserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserConfig")
            .field("flat_tree", &self.flat_tree)
            .field("number_of_top_level_nodes", &self.number_of_top_level_nodes)
            .field("type_of_top_level_nodes", &self.type_of_top_level_nodes)
            .field("recover_block", &self.recover_block)
            .field(
                "always_self_closed_elements",
                &self.always_self_closed_elements,
            )
            .field("raw_text_elements", &self.raw_text_elements)
            .finish()
    }
}

impl ParserConfig {
    /// Create new `ParserConfig` with default config
    pub fn new() -> ParserConfig {
        ParserConfig::default()
    }

    /// Return flat tree instead of nested tree
    pub fn flat_tree(mut self) -> Self {
        self.flat_tree = true;
        self
    }

    /// Exact number of required top level nodes
    pub fn number_of_top_level_nodes(mut self, number: usize) -> Self {
        self.number_of_top_level_nodes = Some(number);
        self
    }

    /// Enforce the `NodeType` of top level nodes
    pub fn type_of_top_level_nodes(mut self, node_type: NodeType) -> Self {
        self.type_of_top_level_nodes = Some(node_type);
        self
    }

    /// Try to parse invalid `syn::Block`.
    /// If set tot true, `NodeBlock` can return `Invalid` variant.
    ///
    /// If `NodeBlock` is failed to parse as `syn::Block`
    /// it still usefull to emit it as expression.
    /// It will enhance IDE compatibility, and provide completion in cases of
    /// invalid blocks, for example `{x.}` is invalid expression, because
    /// after dot token `}` is unexpected. But for ide it is a marker that
    /// quick completion should be provided.
    pub fn recover_block(mut self, recover_block: bool) -> Self {
        self.recover_block = recover_block;
        self
    }

    /// Set array of nodes that is known to be self closed,
    /// it also known as void element.
    /// Void elements has no child and must not have closing tag.
    /// Parser will not search for it closing tag,
    /// even if no slash at end of it open part was found.
    ///
    /// Because we work in proc-macro context, we expect it as 'static refs.
    ///
    /// Examples:
    /// <br> <link> <img>
    pub fn always_self_closed_elements(mut self, elements: HashSet<&'static str>) -> Self {
        self.always_self_closed_elements = elements;
        self
    }

    /// Set array of nodes that is known to be parsed in two-phases,
    /// Parser will skip parsing of children nodes.
    /// and provide one child with RawText instead.
    ///
    /// This is usefull when parsing <script> or <style> tags elements.
    ///
    /// If you need fragment to be used in this context, empty string("") should
    /// be inserted.
    ///
    /// Raw texts has few limitations, check out `RawText` documentation.
    pub fn raw_text_elements(mut self, elements: HashSet<&'static str>) -> Self {
        self.raw_text_elements = elements;
        self
    }

    /// Transforms the `value` of all `NodeType::Block`s with the given closure
    /// callback. The provided `ParseStream` is the content of the block.
    ///
    /// When `Some(TokenStream)` is returned, the `TokenStream` is parsed as
    /// Rust block content. The `ParseStream` must be completely consumed in
    /// this case, meaning no tokens can be left in the stream.
    ///
    /// If `None` is returned, parsing happens with the original `ParseStream`,
    /// since the tokens that are passend into the transform callback are a
    /// fork, which gets only advanced if `Some` is returned.
    ///
    /// An example usage might be a custom syntax inside blocks which isn't
    /// valid Rust. The given example simply translates the `%` character into
    /// the string `percent`
    ///
    /// ```rust
    /// use quote::quote;
    /// use syn::Token;
    /// use rstml::{parse2_with_config, ParserConfig};
    ///
    /// let tokens = quote! {
    ///     <div>{%}</div>
    /// };
    ///
    /// let config = ParserConfig::new().transform_block(|input| {
    ///     input.parse::<Token![%]>()?;
    ///     Ok(Some(quote! { "percent" }))
    /// });
    ///
    /// parse2_with_config(tokens, config).unwrap();
    /// ```
    pub fn transform_block<F>(mut self, callback: F) -> Self
    where
        F: Fn(ParseStream) -> Result<Option<TokenStream>> + 'static,
    {
        self.transform_block = Some(Rc::new(callback));
        self
    }
}
