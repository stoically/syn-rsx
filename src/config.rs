use proc_macro2::TokenStream;
use syn::{parse::ParseStream, Result};

use crate::NodeType;

type TransformBlockFn = dyn Fn(ParseStream) -> Result<Option<TokenStream>>;

/// Configures the `Parser` behavior
#[derive(Default)]
pub struct ParserConfig {
    pub(crate) flat_tree: bool,
    pub(crate) number_of_top_level_nodes: Option<usize>,
    pub(crate) type_of_top_level_nodes: Option<NodeType>,
    pub(crate) transform_block: Option<Box<TransformBlockFn>>,
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

    /// Transforms the `value` of all `NodeType::Block`s with the given closure
    /// callback. The provided `ParseStream` is the content of the block.
    ///
    /// When `Some(TokenStream)` is returned, the `TokenStream` is parsed as
    /// Rust block content. The `ParseStream` must be completely consumed in
    /// this case (no tokens left).
    ///
    /// If `None` is returned, the `ParseStream` is parsed as Rust block
    /// content. The `ParseStream` isn't forked, so partial parsing inside the
    /// transform callback will break this mechanism - fork if you want to avoid
    /// breaking.
    ///
    /// An example usage might be a custom syntax inside blocks which isn't
    /// valid Rust. The given example simply translates the `%` character into
    /// the string `percent`
    ///
    /// ```rust
    /// use quote::quote;
    /// use syn::Token;
    /// use syn_rsx::{parse2_with_config, ParserConfig};
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
        self.transform_block = Some(Box::new(callback));
        self
    }
}
