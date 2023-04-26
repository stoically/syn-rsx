//! RSX Parser

use std::vec;

use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse, ParseStream, Peek},
    punctuated::Punctuated,
    Ident, Result,
};

use crate::{config::EmitError, node::*, ParserConfig};

/// RSX Parser
pub struct Parser {
    config: ParserConfig,
}

impl Parser {
    /// Create a new parser with the given [`ParserConfig`].
    pub fn new(config: ParserConfig) -> Parser {
        Parser { config }
    }

    /// Parse a given [`ParseStream`].
    pub fn parse(&self, input: ParseStream) -> Result<Vec<Node>> {
        let mut nodes = vec![];
        let mut top_level_nodes = 0;

        let _context = crate::context::Context::new_from_config(self.config.clone());
        while !input.cursor().eof() {
            let parsed_node = input.parse::<Node>()?;

            if let Some(type_of_top_level_nodes) = &self.config.type_of_top_level_nodes {
                if &parsed_node.r#type() != type_of_top_level_nodes {
                    return Err(input.error(format!(
                        "top level nodes need to be of type {}",
                        type_of_top_level_nodes
                    )));
                }
            }

            top_level_nodes += 1;
            nodes.push(parsed_node)
        }

        if let Some(number_of_top_level_nodes) = &self.config.number_of_top_level_nodes {
            if &top_level_nodes != number_of_top_level_nodes {
                return Err(input.error(format!(
                    "saw {} top level nodes but exactly {} are required",
                    top_level_nodes, number_of_top_level_nodes
                )));
            }
        }

        match self.config.emit_errors {
            EmitError::All => {}
            EmitError::First => crate::context::get_first_error()?,
        }
        Ok(nodes)
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
