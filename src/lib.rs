//! [`syn`]-powered parser for JSX-like [`TokenStream`]s, aka RSX. The parsed
//! result is a nested [`Node`] structure, similar to the browser DOM, where
//! node name and value are syn expressions to support building proc macros.
//!
//! ```rust
//! # fn main() -> eyre::Result<()> {
//! use std::convert::TryFrom;
//!
//! use eyre::bail;
//! use quote::quote;
//! use rstml::{
//!     node::{Node, NodeAttribute, NodeElement, NodeText},
//!     parse2,
//! };
//!
//! // Create HTML `TokenStream`.
//! let tokens = quote! { <hello world>"hi"</hello> };
//!
//! // Parse the tokens into a tree of `Node`s.
//! let nodes = parse2(tokens)?;
//!
//! // Extract some specific nodes from the tree.
//! let Node::Element(element) = &nodes[0] else { bail!("element") };
//! let NodeAttribute::Attribute(attribute) = &element.attributes()[0] else { bail!("attribute") };
//! let Node::Text(text) = &element.children[0] else { bail!("text") };
//!
//! // Work with the nodes.
//! assert_eq!(element.name().to_string(), "hello");
//! assert_eq!(attribute.key.to_string(), "world");
//! assert_eq!(text.value_string(), "hi");
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
//!   # use rstml::parse2;
//!   # parse2(quote! {
//!   <hello world />
//!   # }).unwrap();
//!   ```
//!
//! - **Text nodes**
//!
//!
//!   ```rust
//!   # use quote::quote;
//!   # use rstml::parse2;
//!   # parse2(quote! {
//!   <div>"String literal"</div>
//!   # }).unwrap();
//!   ```
//!
//!
//! - **Unquoted text nodes**
//!
//! Unquoted text is supported with few limitations:
//! - Only valid Rust TokenStream can be unquoted text (no single quote text is
//!   supported, no unclosed braces, etc.)
//! - Unquoted text not always can save spaces. It uses [`Span::source_text`]
//!   and [`Span::join`] to retrive info about spaces, and it is not always
//!   available.
//! - Quoted text near unquoted treated as diferent Node, end library user
//!   should decide whenever to preserve quotation.
//!
//! ```rust
//! 
//!   # use quote::quote;
//!   # use rstml::parse2;
//!   # parse2(quote! {
//!    <div> Some string that is valid rust token stream </div>
//!   # }).unwrap();
//! ```
//!
//! - **Node names separated by dash, colon or double colon**
//!
//!   ```rust
//!   # use quote::quote;
//!   # use rstml::parse2;
//!   # parse2(quote! {
//!   <tag-name some:attribute-key="value" />
//!   <tag::name attribute::key="value" />
//!   # }).unwrap();
//!   ```
//!
//! - **Node names with reserved keywords**
//!
//!   ```rust
//!   # use quote::quote;
//!   # use rstml::parse2;
//!   # parse2(quote! {
//!   <input type="submit" />
//!   # }).unwrap();
//!   ```
//!
//! - **Doctypes, Comments and Fragments**
//!
//!   ```rust
//!   # use quote::quote;
//!   # use rstml::parse2;
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
//!   # use rstml::parse2;
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
//!   # use rstml::parse2;
//!   # parse2(quote! {
//!   <div key=some::value() />
//!   # }).unwrap();
//!   ```
//!
//! - **Helpful error reporting out of the box**
//!
//!   ```no_build
//!   error: open tag has no corresponding close tag and is not self-closing
//!   --> examples/html-to-string-macro/tests/lib.rs:5:24
//!     |
//!   5 |     html_to_string! { <div> };
//!     |                        ^^^
//!   ```
//!
//! - **Possibility to get the span for a whole node**
//!
//!  This can be used to improve error reporting, e.g.
//!
//!  ```no_build
//!  error: Invalid element
//!  --> examples/src/main.rs:14:13
//!     |
//!  14 | /             <div>
//!  15 | |                 "invalid node for some consumer specific reason"
//!  16 | |             </div>
//!     | |__________________^
//!  ```
//!
//! - **Recoverable parser**
//!
//! Can parse html with multiple mistakes.
//! As result library user get array of errors that can be reported, and tree of
//! nodes that was parsed.
//!
//! ```rust
//!   # use quote::quote;
//!   # use rstml::{Parser, ParserConfig};
//!   # Parser::new(ParserConfig::default()).parse_recoverable(quote! {
//!  <div hello={world.} /> <!-- dot after world is invalid syn expression -->
//!   <>
//!       <div>"1"</x> <!-- incorrect closed tag -->
//!       <div>"2"</div>
//!       <div>"3"</div>
//!       <div {"some-attribute-from-rust-block"}/>
//!   </>
//!   #});
//! ```
//!
//! Using this feature one can write macro in IDE friendly way.
//! This macro will work faster (because on invalid syntax it change output
//! slightly, instead of removing it completely, so IDE can check diff quicly).
//! And give completion (goto definition, and other semantic related feature)
//! more often.
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
//! [`Span::join`]: https://doc.rust-lang.org/proc_macro/struct.Span.html#method.join
//! [`Span::source_text`]: https://doc.rust-lang.org/proc_macro/struct.Span.html#method.source_text
//! [`ParserConfig`]: struct.ParserConfig.html
//! [mod style path]: https://docs.rs/syn/1.0.40/syn/struct.Path.html#method.parse_mod_style
//! [unquoted text is planned]: https://github.com/stoically/syn-rsx/issues/2
//! [`transform_block`]: struct.ParserConfig.html#method.transform_block
//! [#9]: https://github.com/stoically/syn-rsx/issues/9
//! [html-to-string-macro example]: https://github.com/stoically/syn-rsx/tree/main/examples/html-to-string-macro

extern crate proc_macro;

use syn::Result;

mod config;
mod error;
pub mod node;
mod parser;
pub use config::ParserConfig;
pub use error::Error;
pub use node::atoms;
use node::Node;
// pub use node::*;
pub use parser::{recoverable, recoverable::ParsingResult, Parser};

/// Parse the given [`proc-macro::TokenStream`] into a [`Node`] tree.
///
/// [`proc-macro::TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
/// [`Node`]: struct.Node.html
pub fn parse(tokens: proc_macro::TokenStream) -> Result<Vec<Node>> {
    Parser::new(ParserConfig::default()).parse_simple(tokens)
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
    Parser::new(config).parse_simple(tokens)
}
/// Parse the given [`proc-macro2::TokenStream`] into a [`Node`] tree.
///
/// [`proc-macro2::TokenStream`]: https://docs.rs/proc-macro2/latest/proc_macro2/struct.TokenStream.html
/// [`Node`]: struct.Node.html
pub fn parse2(tokens: proc_macro2::TokenStream) -> Result<Vec<Node>> {
    Parser::new(ParserConfig::default()).parse_simple(tokens)
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
    Parser::new(config).parse_simple(tokens)
}
