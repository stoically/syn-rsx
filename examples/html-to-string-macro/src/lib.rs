use std::convert::TryFrom;

use proc_macro::TokenStream;
use quote::quote;
use syn::Expr;
use syn_rsx::{parse_with_config, DynAttribute, Node, NodeAttribute, ParserConfig};

fn walk_nodes<'a>(nodes: &'a Vec<Node>) -> (String, Vec<&'a Expr>) {
    let mut out = String::new();
    let mut values: Vec<&Expr> = vec![];

    for node in nodes {
        match node {
            Node::Doctype(doctype) => {
                let value = String::try_from(&doctype.value)
                    .expect("could not convert node value to string");
                out.push_str(&format!("<!DOCTYPE {}>", value));
            }
            Node::Element(element) => {
                let name = element.name().to_string();
                out.push_str(&format!("<{}", name));

                for attribute in element.attributes() {
                    match attribute {
                        NodeAttribute::Block(DynAttribute { block }) => {
                            // If the nodes parent is an attribute we prefix with whitespace
                            out.push(' ');
                            out.push_str("{}");
                            values.push(&block.value);
                        }
                        NodeAttribute::Attribute(attribute) => {
                            out.push_str(&format!(" {}", attribute.key.to_string()));
                            if let Some(value) = &attribute.value {
                                out.push_str(r#"="{}""#);
                                values.push(&*value);
                            }
                        }
                    }
                }
                // attributes
                out.push('>');

                // https://developer.mozilla.org/en-US/docs/Glossary/Empty_element
                match name.as_str() {
                    "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link"
                    | "meta" | "param" | "source" | "track" | "wbr" => continue,
                    _ => (),
                }

                // children
                let (html_string, children_values) = walk_nodes(&element.children);
                out.push_str(&html_string);
                values.extend(children_values);

                out.push_str(&format!("</{}>", name));
            }
            Node::Text(text) => {
                out.push_str("{}");
                values.push(&text.value);
            }
            Node::Fragment(fragment) => {
                let (html_string, children_values) = walk_nodes(&fragment.children);
                out.push_str(&html_string);
                values.extend(children_values);
            }
            Node::Comment(comment) => {
                out.push_str("<!-- {} -->");
                values.push(&comment.value);
            }
            Node::Block(block) => {
                out.push_str("{}");
                values.push(&block.value);
            }
        }
    }

    (out, values)
}

/// Converts HTML to `String`.
///
/// Values returned from braced blocks `{}` are expected to return something
/// that implements `Display`.
///
/// See [syn-rsx docs](https://docs.rs/syn-rsx/) for supported tags and syntax.
///
/// # Example
///
/// ```
/// use html_to_string_macro::html;
///
/// let world = "planet";
/// assert_eq!(html!(<div>"hello "{world}</div>), "<div>hello planet</div>");
/// ```
#[proc_macro]
pub fn html(tokens: TokenStream) -> TokenStream {
    let config = ParserConfig::new().emit_errors(syn_rsx::EmitError::All);
    let token_stream: TokenStream = match parse_with_config(tokens, config) {
        Ok(nodes) => {
            let (html_string, values) = walk_nodes(&nodes);
            quote! { format!(#html_string, #(#values),*) }
        }
        Err(error) => error.to_compile_error(),
    }
    .into();
    syn_rsx::try_emit_errors(token_stream.into()).into()
}
