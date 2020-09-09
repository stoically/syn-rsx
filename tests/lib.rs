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
    let config = ParserConfig::new().number_of_top_level_nodes(2);

    let tokens = quote! {
        <div />
        <div />
        <div />
    };
    let nodes = parse2_with_config(tokens, config.clone());
    assert!(nodes.is_err());

    let tokens = quote! {
        <div />
        <div />
    };
    let nodes = parse2_with_config(tokens, config.clone());
    assert!(nodes.is_ok());

    let tokens = quote! {
        <div />
    };
    let nodes = parse2_with_config(tokens, config);
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
