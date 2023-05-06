//! RSX Parser

use std::vec;

use proc_macro2::TokenStream;
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse, ParseStream, Peek},
    punctuated::Punctuated,
    Ident, Result,
};

pub mod recoverable;

use self::recoverable::{ParseRecoverable, ParsingResult, RecoverableContext, RecoveryConfig};
use crate::{node::*, ParserConfig};

/// RSX Parser
pub struct Parser {
    config: ParserConfig,
}

impl Parser {
    /// Create a new parser with the given [`ParserConfig`].
    pub fn new(config: ParserConfig) -> Parser {
        Parser { config }
    }

    /// Parse the given [`proc-macro2::TokenStream`] or
    /// [`proc-macro::TokenStream`] into a [`Node`] tree.
    ///
    /// [`proc-macro2::TokenStream`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.TokenStream.html
    /// [`proc-macro::TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
    /// [`Node`]: struct.Node.html
    pub fn parse_simple(&self, v: impl Into<TokenStream>) -> Result<Vec<Node>> {
        self.parse_recoverable(v).into_result()
    }

    /// Advance version of `parse_simple` that returns array of errors in case
    /// of partial parsing.
    pub fn parse_recoverable(&self, v: impl Into<TokenStream>) -> ParsingResult<Vec<Node>> {
        use syn::parse::Parser as _;
        let parser = move |input: ParseStream| Ok(self.parse_syn_stream(input));
        let res = parser.parse2(v.into());
        res.expect("No errors from parser")
    }

    /// Parse a given [`ParseStream`].
    pub fn parse_syn_stream(&self, input: ParseStream) -> ParsingResult<Vec<Node>> {
        let mut nodes = vec![];
        let mut top_level_nodes = 0;

        let mut parser = RecoverableContext::new(RecoveryConfig {
            recover_block: self.config.recover_block,
            raw_text_elements: self.config.raw_text_elements.clone(),
            always_self_closed_elements: self.config.always_self_closed_elements.clone(),
            transform_block: self.config.transform_block.clone(),
        });
        while !input.cursor().eof() {
            let Some(parsed_node) = Node::parse_recoverable(&mut parser, input) else {
                parser.push_diagnostic(input.error(format!(
                    "BUG: Node parse failed"
                )));
                break;
            };

            if let Some(type_of_top_level_nodes) = &self.config.type_of_top_level_nodes {
                if &parsed_node.r#type() != type_of_top_level_nodes {
                    parser.push_diagnostic(input.error(format!(
                        "top level nodes need to be of type {}",
                        type_of_top_level_nodes
                    )));
                    break;
                }
            }

            top_level_nodes += 1;
            nodes.push(parsed_node)
        }

        if let Some(number_of_top_level_nodes) = &self.config.number_of_top_level_nodes {
            if &top_level_nodes != number_of_top_level_nodes {
                parser.push_diagnostic(input.error(format!(
                    "saw {} top level nodes but exactly {} are required",
                    top_level_nodes, number_of_top_level_nodes
                )))
            }
        }

        let nodes = if self.config.flat_tree {
            nodes.into_iter().map(Node::flatten).flatten().collect()
        } else {
            nodes
        };

        let errors = parser.diagnostics;

        let nodes = if nodes.is_empty() { None } else { Some(nodes) };
        ParsingResult::from_parts(nodes, errors)
    }

    /// Parse the stream as punctuated idents.
    ///
    /// We can't replace this with [`Punctuated::parse_separated_nonempty`]
    /// since that doesn't support reserved keywords. Might be worth to
    /// consider a PR upstream.
    ///
    /// [`Punctuated::parse_separated_nonempty`]: https://docs.rs/syn/1.0.58/syn/punctuated/struct.Punctuated.html#method.parse_separated_nonempty
    pub fn node_name_punctuated_ident<T: Parse, F: Peek, X: From<Ident>>(
        input: ParseStream,
        punct: F,
    ) -> Result<Punctuated<X, T>> {
        let fork = &input.fork();
        let mut segments = Punctuated::<X, T>::new();

        while !fork.is_empty() && fork.peek(Ident::peek_any) {
            let ident = Ident::parse_any(fork)?;
            segments.push_value(ident.clone().into());

            if fork.peek(punct) {
                segments.push_punct(fork.parse()?);
            } else {
                break;
            }
        }

        if segments.len() > 1 {
            input.advance_to(fork);
            Ok(segments)
        } else {
            Err(fork.error("expected punctuated node name"))
        }
    }

    /// Parse the stream as punctuated idents, with two possible punctuations
    /// available
    pub fn node_name_punctuated_ident_with_alternate<T: Parse, F: Peek, G: Peek, X: From<Ident>>(
        input: ParseStream,
        punct: F,
        alternate_punct: G,
    ) -> Result<Punctuated<X, T>> {
        let fork = &input.fork();
        let mut segments = Punctuated::<X, T>::new();

        while !fork.is_empty() && fork.peek(Ident::peek_any) {
            let ident = Ident::parse_any(fork)?;
            segments.push_value(ident.clone().into());

            if fork.peek(punct) || fork.peek(alternate_punct) {
                segments.push_punct(fork.parse()?);
            } else {
                break;
            }
        }

        if segments.len() > 1 {
            input.advance_to(fork);
            Ok(segments)
        } else {
            Err(fork.error("expected punctuated node name"))
        }
    }
}
