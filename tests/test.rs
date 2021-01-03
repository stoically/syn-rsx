use quote::quote;
use syn_rsx::{parse2, parse2_with_config, NodeType, ParserConfig};

#[test]
fn test_single_empty_element() {
    let tokens = quote! {
        <foo></foo>
    };
    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].name_as_string().unwrap(), "foo");
}

#[test]
fn test_single_element_with_attributes() {
    let tokens = quote! {
        <foo bar="moo" baz="42"></foo>
    };
    let nodes = parse2(tokens).unwrap();

    let attribute = &nodes[0].attributes[0];
    assert_eq!(attribute.name_as_string().unwrap(), "bar");
    assert_eq!(attribute.value_as_string().unwrap(), "moo");
}

#[test]
fn test_single_element_with_text() {
    let tokens = quote! {
        <foo>"bar"</foo>
    };

    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].children[0].value_as_string().unwrap(), "bar");
}

#[test]
fn test_reserved_keyword_attributes() {
    let tokens = quote! {
        <input type="foo" />
    };
    let nodes = parse2(tokens).unwrap();

    assert_eq!(nodes[0].name_as_string().unwrap(), "input");
    assert_eq!(nodes[0].attributes[0].name_as_string().unwrap(), "type");
}

#[test]
fn test_block_node() {
    let tokens = quote! {
        <div>{hello}</div>
    };
    let nodes = parse2(tokens).unwrap();

    assert_eq!(nodes[0].children.len(), 1);
}

#[test]
fn test_flat_tree() {
    let config = ParserConfig::new().flat_tree();

    let tokens = quote! {
        <div>
            <div>
                <div>{hello}</div>
                <div>"world"</div>
            </div>
        </div>
        <div />
    };

    let nodes = parse2_with_config(tokens, config).unwrap();
    assert_eq!(nodes.len(), 7);
}

#[test]
fn test_path_as_tag_name() {
    let tokens = quote! {
        <some::path />
    };

    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].name_as_string().unwrap(), "some::path");
}

#[test]
fn test_block_as_tag_name() {
    let tokens = quote! {
        <{some_logic(block)} />
    };

    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].name_as_block().is_some(), true);
}

#[test]
fn test_block_as_tag_name_with_closing_tag() {
    let tokens = quote! {
        <{some_logic(block)}>"Test"</{some_logic(block)}>
    };

    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].name_as_block().is_some(), true);
}

#[test]
fn test_dashed_attribute_name() {
    let tokens = quote! {
        <div data-foo="bar" />
    };

    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].attributes[0].name_as_string().unwrap(), "data-foo");
}

#[test]
fn test_coloned_attribute_name() {
    let tokens = quote! {
        <div on:click={foo} />
    };

    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].attributes[0].name_as_string().unwrap(), "on:click");
}

#[test]
fn test_block_as_attribute() {
    let tokens = quote! {
        <div {attribute} />
    };

    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].attributes.len(), 1);
}

#[test]
fn test_number_of_top_level_nodes() {
    let tokens = quote! {
        <div />
        <div />
        <div />
    };
    let nodes = parse2_with_config(tokens, ParserConfig::new().number_of_top_level_nodes(2));
    assert!(nodes.is_err());

    let tokens = quote! {
        <div>
            <div />
        </div>
        <div />
    };
    let nodes = parse2_with_config(
        tokens,
        ParserConfig::new().number_of_top_level_nodes(2).flat_tree(),
    );
    assert!(nodes.is_ok());

    let tokens = quote! {
        <div />
    };
    let nodes = parse2_with_config(tokens, ParserConfig::new().number_of_top_level_nodes(2));
    assert!(nodes.is_err());
}

#[test]
fn test_type_of_top_level_nodes() {
    let tokens = quote! {
        "foo"
    };

    let config = ParserConfig::new().type_of_top_level_nodes(NodeType::Element);
    let nodes = parse2_with_config(tokens, config);

    assert!(nodes.is_err())
}

#[test]
fn test_transform_block_some() {
    use syn::{Expr, Lit, Stmt, Token};

    let tokens = quote! {
        <div>{%}</div>
    };

    let config = ParserConfig::new().transform_block(|input| {
        input.parse::<Token![%]>()?;
        Ok(Some(quote! { "percent" }))
    });

    let nodes = parse2_with_config(tokens, config).unwrap();

    assert_eq!(
        match &nodes[0].children[0].value {
            Some(Expr::Block(expr)) => {
                match &expr.block.stmts[0] {
                    Stmt::Expr(Expr::Lit(expr)) => match &expr.lit {
                        Lit::Str(lit_str) => Some(lit_str.value()),
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        },
        Some("percent".to_owned())
    )
}

#[test]
fn test_transform_block_none() {
    let tokens = quote! {
        <div>{"foo"}</div>
    };

    let config = ParserConfig::new().transform_block(|_| Ok(None));
    let nodes = parse2_with_config(tokens, config);

    assert!(nodes.is_ok())
}

#[test]
fn test_doctype() {
    let tokens = quote! {
        <!DOCTYPE html>
        <html>
        </html>
    };

    let nodes = parse2(tokens).unwrap();

    assert_eq!(nodes[0].node_type, NodeType::Doctype);
    assert_eq!(nodes[0].name_as_string(), Some("html".to_owned()));
}

#[test]
fn test_comment() {
    let tokens = quote! {
        <!-- "comment1" -->
        <div>
            <!-- "comment2" -->
            <div />
        </div>
    };

    let nodes = parse2(tokens).unwrap();
    assert_eq!(nodes[0].value_as_string(), Some("comment1".to_owned()));
    assert_eq!(
        nodes[1].children[0].value_as_string(),
        Some("comment2".to_owned())
    );
}

#[test]
fn test_fragment() {
    let tokens = quote! {
        <>
            <div />
        </>
    };

    let nodes = parse2(tokens);

    assert!(nodes.is_ok());
}
