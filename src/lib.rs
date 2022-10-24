//! [`syn`]-powered parser for JSX-like [`TokenStream`]s, aka RSX. The parsed
//! result is a nested [`Node`] structure, similar to the browser DOM, where
//! node name and value are syn expressions to support building proc macros.
//!
//! ```rust
//! # fn main() -> eyre::Result<()> {
//! use std::convert::TryFrom;
//!
//! use extrude::extrude;
//! use quote::quote;
//! use syn_rsx::{parse2, Node, NodeAttribute, NodeElement, NodeText};
//!
//! // Create HTML `TokenStream`.
//! let tokens = quote! { <hello world>"hi"</hello> };
//!
//! // Parse the tokens into a tree of `Node`s.
//! let nodes = parse2(tokens)?;
//!
//! // Extract some specific nodes from the tree.
//! let element = extrude!(&nodes[0], Node::Element(element)).unwrap();
//! let attribute = extrude!(&element.attributes[0], Node::Attribute(attribute)).unwrap();
//! let text = extrude!(&element.children[0], Node::Text(text)).unwrap();
//!
//! // Work with the nodes.
//! assert_eq!(element.name.to_string(), "hello");
//! assert_eq!(attribute.key.to_string(), "world");
//! assert_eq!(String::try_from(&text.value)?, "hi");
//! # Ok(())
//! # }
//! ```
//!
//! You might want to check out the [html-to-string-macro example] as well.
//!
//! ## Features
//!
//! - **Not opinionated**
//!
//!   Every tag or attribute name is valid
//!
//!   ```rust
//!   # use quote::quote;
//!   # use syn_rsx::parse2;
//!   # parse2(quote! {
//!   <hello world />
//!   # }).unwrap();
//!   ```
//!
//! - **Text nodes**
//!
//!   Support for [unquoted text is planned].
//!
//!   ```rust
//!   # use quote::quote;
//!   # use syn_rsx::parse2;
//!   # parse2(quote! {
//!   <div>"String literal"</div>
//!   # }).unwrap();
//!   ```
//!
//! - **Node names separated by dash, colon or double colon**
//!
//!   ```rust
//!   # use quote::quote;
//!   # use syn_rsx::parse2;
//!   # parse2(quote! {
//!   <tag-name attribute-key="value" />
//!   <tag:name attribute:key="value" />
//!   <tag::name attribute::key="value" />
//!   # }).unwrap();
//!   ```
//!
//! - **Node names with reserved keywords**
//!
//!   ```rust
//!   # use quote::quote;
//!   # use syn_rsx::parse2;
//!   # parse2(quote! {
//!   <input type="submit" />
//!   # }).unwrap();
//!   ```
//!
//! - **Doctypes, Comments and Fragments**
//!
//!   ```rust
//!   # use quote::quote;
//!   # use syn_rsx::parse2;
//!   # parse2(quote! {
//!   <!DOCTYPE html>
//!   <!-- "comment" -->
//!   <></>
//!   # }).unwrap();
//!   ```
//!
//! - **Braced blocks are parsed as arbitrary Rust code**
//!
//!   ```rust
//!   # use quote::quote;
//!   # use syn_rsx::parse2;
//!   # parse2(quote! {
//!   <{ let block = "in node name position"; } />
//!   <div>{ let block = "in node position"; }</div>
//!   <div { let block = "in attribute position"; } />
//!   <div key={ let block = "in attribute value position"; } />
//!   # }).unwrap();
//!   ```
//!
//! - **Attribute values can be any valid syn expression without requiring
//!   braces**
//!
//!   ```rust
//!   # use quote::quote;
//!   # use syn_rsx::parse2;
//!   # parse2(quote! {
//!   <div key=some::value() />
//!   # }).unwrap();
//!   ```
//!
//! - **Helpful error reporting out of the box**
//!
//!   ```ignore
//!   error: open tag has no corresponding close tag and is not self-closing
//!   --> examples/html-to-string-macro/tests/lib.rs:5:24
//!     |
//!   5 |     html_to_string! { <div> };
//!     |                        ^^^
//!   ```
//!
//! - **Customization**
//!
//!   A [`ParserConfig`] to customize parsing behavior is available, so if you
//! have   slightly different requirements for parsing and it's not yet
//! customizable   feel free to open an issue or pull request to extend the
//! configuration.
//!
//!   One highlight with regards to customization is the [`transform_block`]
//!   configuration, which takes a closure that receives raw block content as
//!   `ParseStream` and lets you optionally convert it to a `TokenStream`. That
//! makes it   possible to have custom syntax in blocks. More details in [#9]
//!
//!
//! [`syn`]: /syn
//! [`TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
//! [`Node`]: enum.Node.html
//! [`ParserConfig`]: struct.ParserConfig.html
//! [mod style path]: https://docs.rs/syn/1.0.40/syn/struct.Path.html#method.parse_mod_style
//! [unquoted text is planned]: https://github.com/stoically/syn-rsx/issues/2
//! [`transform_block`]: struct.ParserConfig.html#method.transform_block
//! [#9]: https://github.com/stoically/syn-rsx/issues/9
//! [html-to-string-macro example]: https://github.com/stoically/syn-rsx/tree/main/examples/html-to-string-macro

extern crate proc_macro;

use syn::{
    parse::{ParseStream, Parser as _},
    Result,
};

mod config;
mod error;
mod node;
mod parser;

pub mod punctuation {
    //! Custom syn punctuations
    use syn::custom_punctuation;

    custom_punctuation!(Dash, -);
}

pub use config::ParserConfig;
pub use error::Error;
pub use node::*;
pub use parser::Parser;

/// Parse the given [`proc-macro::TokenStream`] into a [`Node`] tree.
///
/// [`proc-macro::TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
/// [`Node`]: struct.Node.html
pub fn parse(tokens: proc_macro::TokenStream) -> Result<Vec<Node>> {
    let parser = move |input: ParseStream| Parser::new(ParserConfig::default()).parse(input);

    parser.parse(tokens)
}

/// Parse the given [`proc-macro::TokenStream`] into a [`Node`] tree with custom
/// [`ParserConfig`].
///
/// [`proc-macro::TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
/// [`Node`]: struct.Node.html
/// [`ParserConfig`]: struct.ParserConfig.html
pub fn parse_with_config(
    tokens: proc_macro::TokenStream,
    config: ParserConfig,
) -> Result<Vec<Node>> {
    let parser = move |input: ParseStream| Parser::new(config).parse(input);

    parser.parse(tokens)
}

/// Parse the given [`proc-macro2::TokenStream`] into a [`Node`] tree.
///
/// [`proc-macro2::TokenStream`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.TokenStream.html
/// [`Node`]: struct.Node.html
pub fn parse2(tokens: proc_macro2::TokenStream) -> Result<Vec<Node>> {
    let parser = move |input: ParseStream| Parser::new(ParserConfig::default()).parse(input);

    parser.parse2(tokens)
}

/// Parse the given [`proc-macro2::TokenStream`] into a [`Node`] tree with
/// custom [`ParserConfig`].
///
/// [`proc-macro2::TokenStream`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.TokenStream.html
/// [`Node`]: struct.Node.html
/// [`ParserConfig`]: struct.ParserConfig.html
pub fn parse2_with_config(
    tokens: proc_macro2::TokenStream,
    config: ParserConfig,
) -> Result<Vec<Node>> {
    let parser = move |input: ParseStream| Parser::new(config).parse(input);

    parser.parse2(tokens)
}
