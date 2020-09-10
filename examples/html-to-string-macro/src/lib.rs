use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error};
use syn_rsx::{parse, Node, NodeType};

fn walk_nodes(nodes: Vec<Node>) -> Result<String, proc_macro2::TokenStream> {
    let mut out = String::new();
    for node in nodes {
        match node.node_type {
            NodeType::Element => {
                let name = node.name_as_string().unwrap();
                out.push_str(&format!("<{}", name));
                match walk_nodes(node.attributes) {
                    Ok(html_string) => out.push_str(&html_string),
                    Err(error) => return Err(error),
                }
                out.push_str(">");

                match walk_nodes(node.children) {
                    Ok(html_string) => out.push_str(&html_string),
                    Err(error) => return Err(error),
                };

                out.push_str(&format!("</{}>", name));
            }
            NodeType::Attribute => {
                out.push_str(&format!(" {}", node.name_as_string().unwrap()));
                if node.value.is_some() {
                    match node.value_as_string() {
                        Some(value) => out.push_str(&format!("=\"{}\"", &value)),
                        None => return Err(Error::new(
                            node.name_span().unwrap(),
                            "Only String literals as attribute value are supported in this example",
                        )
                        .to_compile_error()),
                    }
                }
            }
            NodeType::Text => out.push_str(&node.value_as_string().unwrap()),
            NodeType::Block => {
                return Err(Error::new(
                    node.value_as_block().unwrap().span(),
                    "NodeType::Block is not supported in this example",
                )
                .to_compile_error())
            }
        }
    }

    Ok(out)
}

#[proc_macro]
pub fn html_to_string(tokens: TokenStream) -> TokenStream {
    match parse(tokens) {
        Ok(nodes) => match walk_nodes(nodes) {
            Ok(html_string) => quote! { #html_string },
            Err(error) => error,
        },
        Err(error) => error.to_compile_error(),
    }
    .into()
}
