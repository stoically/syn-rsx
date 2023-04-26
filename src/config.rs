use std::rc::Rc;

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
    pub(crate) emit_errors: EmitError,
}

/// How parsing error should be emitted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmitError {
    /// Whenever first error is received.
    First,
    /// Try to recover after invalid parsing.
    /// The end user of library should then process them.
    All,
}

impl Default for EmitError {
    fn default() -> Self {
        EmitError::First
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

    /// Change behaviour of emitting errors
    pub fn emit_errors(mut self, emit_errors: EmitError) -> Self {
        self.emit_errors = emit_errors;
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
        self.transform_block = Some(Rc::new(callback));
        self
    }
}
