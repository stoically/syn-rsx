//! [`syn`]-powered parser for JSX-like [`TokenStream`]s. The parsed result is a
//! nested [`Node`] structure, similar to the browser DOM. The `node_value` is
//! an [`syn::Expr`].
//!
//! [`syn`]: /syn
//! [`TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
//! [`Node`]: struct.Node.html
//! [`syn::Expr`]: https://docs.rs/syn/latest/syn/enum.Expr.html
//!
//! ```
//! use syn_rsx::parse2;
//! use quote::quote;
//!
//! let tokens = quote! {
//!     <div>
//!         <div>"hello"</div>
//!         <world />
//!     </div>
//! };
//!
//! let nodes = parse2(tokens, None).unwrap();
//! assert_eq!(nodes[0].childs.len(), 2);
//! assert_eq!(nodes[0].childs[1].name_as_string().unwrap(), "world");
//! ```

extern crate proc_macro;

use syn::{
    parse::{ParseStream, Parser as _},
    Result,
};

mod node;
mod parser;

pub use node::{Node, NodeType};
pub use parser::{Parser, ParserConfig};

/// Parse the given `proc-macro::TokenStream` into a `Node` tree
pub fn parse(tokens: proc_macro::TokenStream, config: Option<ParserConfig>) -> Result<Vec<Node>> {
    let parser = move |input: ParseStream| {
        let config = config.unwrap_or_else(ParserConfig::default);
        Parser::new(config).parse(input)
    };

    parser.parse(tokens)
}

/// Parse the given `proc-macro2::TokenStream` into a `Node` tree
pub fn parse2(tokens: proc_macro2::TokenStream, config: Option<ParserConfig>) -> Result<Vec<Node>> {
    let parser = move |input: ParseStream| {
        let config = config.unwrap_or_else(ParserConfig::default);
        Parser::new(config).parse(input)
    };

    parser.parse2(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Expr, Lit};

    #[test]
    fn test_single_empty_element() {
        let tokens = quote::quote! {
            <foo></foo>
        };
        let nodes = parse2(tokens, None).unwrap();
        assert_eq!(nodes[0].name_as_string().unwrap(), "foo");
    }

    #[test]
    fn test_single_element_with_attributes() {
        let tokens = quote::quote! {
            <foo bar="moo" baz="42"></foo>
        };
        let nodes = parse2(tokens, None).unwrap();

        let attribute = &nodes[0].attributes[0];
        let attribute_value = match attribute.value.as_ref().unwrap() {
            Expr::Lit(expr) => match &expr.lit {
                Lit::Str(lit_str) => Some(lit_str.value()),
                _ => None,
            },
            _ => None,
        }
        .unwrap();

        assert_eq!(attribute.name_as_string().unwrap(), "bar");
        assert_eq!(attribute_value, "moo");
    }

    #[test]
    fn test_single_element_with_text() {
        let tokens = quote::quote! {
            <foo>"bar"</foo>
        };
        let nodes = parse2(tokens, None).unwrap();

        let node_value = match nodes[0].childs[0].value.as_ref().unwrap() {
            Expr::Lit(expr) => match &expr.lit {
                Lit::Str(lit_str) => Some(lit_str.value()),
                _ => None,
            },
            _ => None,
        }
        .unwrap();

        assert_eq!(node_value, "bar");
    }

    #[test]
    fn test_reserved_keyword_attributes() {
        let tokens = quote::quote! {
            <input type="foo" />
        };
        let nodes = parse2(tokens, None).unwrap();

        assert_eq!(nodes[0].name_as_string().unwrap(), "input");
        assert_eq!(nodes[0].attributes[0].name_as_string().unwrap(), "type");
    }

    #[test]
    fn test_block_node() {
        let tokens = quote::quote! {
            <div>{hello}</div>
        };
        let nodes = parse2(tokens, None).unwrap();

        assert_eq!(nodes[0].childs.len(), 1);
    }

    #[test]
    fn test_flat_tree() {
        let config = ParserConfig { flatten: true };

        let tokens = quote::quote! {
            <div>
                <div>
                    <div>{hello}</div>
                    <div>"world"</div>
                </div>
            </div>
            <div />
        };

        let nodes = parse2(tokens, Some(config)).unwrap();
        assert_eq!(nodes.len(), 7);
    }

    #[test]
    fn test_path_as_tag_name() {
        let tokens = quote::quote! {
            <some::path />
        };

        let nodes = parse2(tokens, None).unwrap();
        assert_eq!(nodes[0].name_as_string().unwrap(), "some::path");
    }
}
