use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use rstml::{
    node::{Node, NodeAttribute, NodeName},
    Parser, ParserConfig,
};
use syn::spanned::Spanned;

#[derive(Default)]
struct WalkNodesOutput<'a> {
    static_format: String,
    // Use proc_macro2::TokenStream instead of syn::Expr
    // to provide more errors to the end user.
    values: Vec<proc_macro2::TokenStream>,
    // Collect elements to provide semantic highlight based on element tag.
    // No differences between open tag and closed tag.
    // Also multiple tags with same name can be present,
    // because we need to mark each of them.
    collected_elements: Vec<&'a NodeName>,
}
impl<'a> WalkNodesOutput<'a> {
    fn extend(&mut self, other: WalkNodesOutput<'a>) {
        self.static_format.push_str(&other.static_format);
        self.values.extend(other.values);
        self.collected_elements.extend(other.collected_elements);
    }
}

fn walk_nodes(nodes: &Vec<Node>) -> WalkNodesOutput<'_> {
    let mut out = WalkNodesOutput::default();

    for node in nodes {
        match node {
            Node::Doctype(doctype) => {
                let value = &doctype.value.to_token_stream_string();
                out.static_format.push_str(&format!("<!DOCTYPE {}>", value));
            }
            Node::Element(element) => {
                let name = element.name().to_string();
                out.static_format.push_str(&format!("<{}", name));
                out.collected_elements.push(&element.open_tag.name);
                if let Some(e) = &element.close_tag {
                    out.collected_elements.push(&e.name)
                }

                // attributes
                for attribute in element.attributes() {
                    match attribute {
                        NodeAttribute::Block(block) => {
                            // If the nodes parent is an attribute we prefix with whitespace
                            out.static_format.push(' ');
                            out.static_format.push_str("{}");
                            out.values.push(block.to_token_stream());
                        }
                        NodeAttribute::Attribute(attribute) => {
                            out.static_format.push_str(&format!(" {}", attribute.key));
                            if let Some(value) = attribute.value() {
                                out.static_format.push_str(r#"="{}""#);
                                out.values.push(value.to_token_stream());
                            }
                        }
                    }
                }
                // In HTML selfclosing slash is ignored but for embeded SVG or MathML it
                // meaningfull.
                if element.open_tag.is_self_closed() {
                    out.static_format.push('/');
                }
                out.static_format.push('>');

                // skip emiting childs if
                //  - no closing tag (self closed, or self closed without marker)
                //  - but element is valid and no childs left
                if element.close_tag.is_none() && element.children.is_empty() {
                    continue;
                }

                // children
                let other_output = walk_nodes(&element.children);
                out.extend(other_output);
                out.static_format.push_str(&format!("</{}>", name));
            }
            Node::Text(text) => {
                out.static_format.push_str(&text.value_string());
            }
            Node::RawText(text) => {
                out.static_format.push_str("{}");
                let tokens = text.to_string_best();
                let literal = Literal::string(&tokens);

                out.values.push(TokenTree::from(literal).into());
            }
            Node::Fragment(fragment) => {
                let other_output = walk_nodes(&fragment.children);
                out.extend(other_output)
            }
            Node::Comment(comment) => {
                out.static_format.push_str("<!-- {} -->");
                out.values.push(comment.value.to_token_stream());
            }
            Node::Block(block) => {
                out.static_format.push_str("{}");
                out.values.push(block.to_token_stream());
            }
        }
    }

    out
}

/// Converts HTML to `String`.
///
/// Values returned from braced blocks `{}` are expected to return something
/// that implements `Display`.
///
/// See [rstml docs](https://docs.rs/rstml/) for supported tags and syntax.
///
/// # Example
///
/// ```
/// use html_to_string_macro::html;
/// // using this macro, one should write docs module on top level of crate.
/// // Macro will link html tags to them.
/// pub mod docs {
///     /// Element has open and close tags, content and attributes.
///     pub fn element() {}
/// }
/// # fn main (){
///
/// let world = "planet";
/// assert_eq!(html!(<div>"hello "{world}</div>), "<div>hello planet</div>");
/// # }
/// ```
#[proc_macro]
pub fn html(tokens: TokenStream) -> TokenStream {
    let config = ParserConfig::new()
        .recover_block(true)
        // https://developer.mozilla.org/en-US/docs/Glossary/Empty_element
        .always_self_closed_elements(
            [
                "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                "param", "source", "track", "wbr",
            ]
            .into_iter()
            .collect(),
        )
        .raw_text_elements(["script", "style"].into_iter().collect());

    let parser = Parser::new(config);
    let (nodes, errors) = parser.parse_recoverable(tokens).split_vec();

    let WalkNodesOutput {
        static_format: html_string,
        values,
        collected_elements: elements,
    } = walk_nodes(&nodes);
    let docs = generate_tags_docs(elements);
    let errors = errors.into_iter().map(|e| e.emit_as_expr_tokens());
    quote! {
        {
            // Make sure that "compile_error!(..);"  can be used in this context.
            #(#errors;)*
            // Make sure that "enum x{};" and "let _x = crate::element;"  can be used in this context
            #(#docs;)*
            format!(#html_string, #(#values),*)
        }
    }
    .into()
}

fn generate_tags_docs(elements: Vec<&NodeName>) -> Vec<proc_macro2::TokenStream> {
    // Mark some of elements as type,
    // and other as elements as fn in crate::docs,
    // to give an example how to link tag with docs.
    let elements_as_type: HashSet<&'static str> = vec!["html", "head", "meta", "link", "body"]
        .into_iter()
        .collect();

    elements
        .into_iter()
        .map(|e| {
            if elements_as_type.contains(&*e.to_string()) {
                let element = quote_spanned!(e.span() => enum);
                quote!({#element x{}})
            } else {
                // let _ = crate::docs::element;
                let element = quote_spanned!(e.span() => element);
                quote!(let _ = crate::docs::#element)
            }
        })
        .collect()
}
