//! [`syn`]-powered parser for JSX-like [`TokenStream`]s. The parsed result is a
//! nested [`Node`] structure, similar to the browser DOM, where node name and
//! value are syn expressions to support building proc macros. A [`ParserConfig`]
//! to customize parsing behavior is available, so if you have slightly
//! different requirements for parsing and it's not yet customizable feel free
//! to open an issue or pull request to extend the configuration.
//!
//! [`syn`]: /syn
//! [`TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
//! [`Node`]: struct.Node.html
//! [`ParserConfig`]: struct.ParserConfig.html
//!
//! ```
//! use quote::quote;
//! use syn_rsx::parse2;
//!
//! let tokens = quote! {
//!     <div foo={bar}>
//!         <div>"hello"</div>
//!         <world />
//!     </div>
//! };
//!
//! let nodes = parse2(tokens, None).unwrap();
//!
//! let node = &nodes[0];
//! assert_eq!(node.attributes[0].name_as_string().unwrap(), "foo");
//!
//! let children = &node.children;
//! assert_eq!(children.len(), 2);
//! assert_eq!(children[0].children[0].value_as_string().unwrap(), "hello");
//! assert_eq!(children[1].name_as_string().unwrap(), "world");
//! ```

extern crate proc_macro;

use syn::{
    parse::{ParseStream, Parser as _},
    Result,
};

mod node;
mod parser;

pub mod punctuation {
    //! Custom syn punctuations
    use syn::custom_punctuation;

    custom_punctuation!(Dash, -);
}

pub use node::{Node, NodeName, NodeType};
pub use parser::{Parser, ParserConfig};

/// Parse the given [`proc-macro::TokenStream`] into a [`Node`] tree
///
/// [`proc-macro::TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
/// [`Node`]: struct.Node.html
pub fn parse(tokens: proc_macro::TokenStream, config: Option<ParserConfig>) -> Result<Vec<Node>> {
    let parser = move |input: ParseStream| {
        let config = config.unwrap_or_else(ParserConfig::default);
        Parser::new(config).parse(input)
    };

    parser.parse(tokens)
}

/// Parse the given [`proc-macro2::TokenStream`] into a [`Node`] tree
///
/// [`proc-macro2::TokenStream`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.TokenStream.html
/// [`Node`]: struct.Node.html
pub fn parse2(tokens: proc_macro2::TokenStream, config: Option<ParserConfig>) -> Result<Vec<Node>> {
    let parser = move |input: ParseStream| {
        let config = config.unwrap_or_else(ParserConfig::default);
        Parser::new(config).parse(input)
    };

    parser.parse2(tokens)
}
