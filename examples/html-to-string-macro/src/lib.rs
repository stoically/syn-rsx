use proc_macro::TokenStream;
use quote::quote;
use syn::Expr;
use syn_rsx::{parse, Node, NodeType};

fn walk_nodes(nodes: Vec<Node>) -> Result<(String, Vec<Expr>), proc_macro2::TokenStream> {
    let mut out = String::new();
    let mut values = vec![];
    for node in nodes {
        match node.node_type {
            NodeType::Element => {
                let name = node.name_as_string().unwrap();
                out.push_str(&format!("<{}", name));
                match walk_nodes(node.attributes) {
                    Ok((html_string, attribute_values)) => {
                        out.push_str(&html_string);
                        values.extend(attribute_values);
                    }
                    Err(error) => return Err(error),
                }
                out.push_str(">");

                // https://developer.mozilla.org/en-US/docs/Glossary/Empty_element
                match name.as_str() {
                    "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link"
                    | "meta" | "param" | "source" | "track" | "wbr" => continue,
                    _ => (),
                }

                match walk_nodes(node.children) {
                    Ok((html_string, children_values)) => {
                        out.push_str(&html_string);
                        values.extend(children_values);
                    }
                    Err(error) => return Err(error),
                };

                out.push_str(&format!("</{}>", name));
            }
            NodeType::Attribute => {
                out.push_str(&format!(" {}", node.name_as_string().unwrap()));
                if node.value.is_some() {
                    out.push_str(r#"="{}""#);
                    values.push(node.value.unwrap());
                }
            }
            NodeType::Text | NodeType::Block => {
                out.push_str("{}");
                values.push(node.value.unwrap());
            }
            _ => (),
        }
    }

    Ok((out, values))
}

#[proc_macro]
pub fn html_to_string(tokens: TokenStream) -> TokenStream {
    match parse(tokens) {
        Ok(nodes) => match walk_nodes(nodes) {
            Ok((html_string, values)) => {
                quote! { format!(#html_string, #(#values),*) }
            }
            Err(error) => error,
        },
        Err(error) => error.to_compile_error(),
    }
    .into()
}
