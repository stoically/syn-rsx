use std::convert::TryFrom;

use eyre::Result;
use quote::quote;
use syn::{Block, ExprBlock};
use syn_rsx::{
    parse2, parse2_with_config, KeyedAttribute, Node, NodeAttribute, NodeElement, NodeType,
    ParserConfig,
};

#[test]
fn test_single_empty_element() -> Result<()> {
    let tokens = quote! {
        <foo></foo>
    };
    let nodes = parse2(tokens)?;
    let element = get_element(&nodes, 0);

    assert_eq!(element.name().to_string(), "foo");

    Ok(())
}

#[test]
fn test_single_element_with_attributes() -> Result<()> {
    let tokens = quote! {
        <foo bar="moo" baz="42"></foo>
    };
    let nodes = parse2(tokens)?;

    let attribute = get_element_attribute(&nodes, 0, 0);

    assert_eq!(attribute.key.to_string(), "bar");
    assert_eq!(
        String::try_from(attribute.value.as_ref().expect("value"))?,
        "moo"
    );

    Ok(())
}

#[test]
fn test_single_element_with_text() -> Result<()> {
    let tokens = quote! {
        <foo>"bar"</foo>
    };

    let nodes = parse2(tokens)?;
    let Node::Text(child) = get_element_child(&nodes, 0, 0) else { panic!("expected child") };

    assert_eq!(String::try_from(&child.value)?, "bar");

    Ok(())
}

#[test]
fn test_reserved_keyword_attributes() -> Result<()> {
    let tokens = quote! {
        <input type="foo" />
    };
    let nodes = parse2(tokens)?;
    let element = get_element(&nodes, 0);
    let Some(NodeAttribute::Attribute(attribute)) = element.attributes().get(0) else { panic!("expected attribute") };

    assert_eq!(element.name().to_string(), "input");
    assert_eq!(attribute.key.to_string(), "type");

    Ok(())
}

#[test]
fn test_block_node() -> Result<()> {
    let tokens = quote! {
        <div>{hello}</div>
    };
    let nodes = parse2(tokens)?;
    let element = get_element(&nodes, 0);

    assert_eq!(element.children.len(), 1);

    Ok(())
}

#[test]
fn test_flat_tree() -> Result<()> {
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
    let nodes = parse2_with_config(tokens, config)?;

    assert_eq!(nodes.len(), 7);

    Ok(())
}

#[test]
fn test_path_as_tag_name() -> Result<()> {
    let tokens = quote! {
        <some::path />
    };

    let nodes = parse2(tokens)?;
    let element = get_element(&nodes, 0);

    assert_eq!(element.name().to_string(), "some::path");

    Ok(())
}

#[test]
fn test_block_as_tag_name() -> Result<()> {
    let tokens = quote! {
        <{some_logic(block)} />
    };

    let nodes = parse2(tokens)?;
    let element = get_element(&nodes, 0);

    assert!(Block::try_from(element.name()).is_ok());

    Ok(())
}

#[test]
fn test_block_as_tag_name_with_closing_tag() -> Result<()> {
    let tokens = quote! {
        <{some_logic(block)}>"Test"</{some_logic(block)}>
    };

    let nodes = parse2(tokens)?;
    let element = get_element(&nodes, 0);

    assert!(Block::try_from(element.name()).is_ok());

    Ok(())
}

#[test]
fn test_dashed_attribute_name() -> Result<()> {
    let tokens = quote! {
        <div data-foo="bar" />
    };

    let nodes = parse2(tokens)?;
    let attribute = get_element_attribute(&nodes, 0, 0);

    assert_eq!(attribute.key.to_string(), "data-foo");

    Ok(())
}

#[test]
fn test_coloned_attribute_name() -> Result<()> {
    let tokens = quote! {
        <div on:click={foo} />
    };

    let nodes = parse2(tokens)?;
    let attribute = get_element_attribute(&nodes, 0, 0);

    assert_eq!(attribute.key.to_string(), "on:click");

    Ok(())
}

#[test]
fn test_mixed_colon_and_dash_attribute_name() -> Result<()> {
    let tokens = quote! {
        <div on:ce-click={foo} />
    };

    let nodes = parse2(tokens)?;
    let attribute = get_element_attribute(&nodes, 0, 0);

    assert_eq!(attribute.key.to_string(), "on:ce-click");

    Ok(())
}

#[test]
fn test_block_as_attribute() -> Result<()> {
    let tokens = quote! {
        <div {attribute} />
    };

    let nodes = parse2(tokens)?;
    let element = get_element(&nodes, 0);

    assert_eq!(element.attributes().len(), 1);

    Ok(())
}

#[test]
fn test_number_of_top_level_nodes() -> Result<()> {
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

    Ok(())
}

#[test]
fn test_type_of_top_level_nodes() -> Result<()> {
    let tokens = quote! {
        "foo"
    };

    let config = ParserConfig::new().type_of_top_level_nodes(NodeType::Element);
    let nodes = parse2_with_config(tokens, config);

    assert!(nodes.is_err());

    Ok(())
}

#[test]
fn test_transform_block_some() -> Result<()> {
    use syn::{Expr, Lit, Stmt, Token};

    let tokens = quote! {
        <div>{%}</div>
    };

    let config = ParserConfig::new().transform_block(|input| {
        input.parse::<Token![%]>()?;
        Ok(Some(quote! { "percent" }))
    });

    let nodes = parse2_with_config(tokens, config)?;
    let Node::Block(block) = get_element_child(&nodes, 0, 0) else { panic!("expected block") };

    assert_eq!(
        match block.try_block().as_ref() {
            Some(block) => {
                match &block.stmts[0] {
                    Stmt::Expr(Expr::Lit(expr), None) => match &expr.lit {
                        Lit::Str(lit_str) => Some(lit_str.value()),
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        },
        Some("percent".to_owned())
    );

    Ok(())
}

#[test]
fn test_transform_block_none() -> Result<()> {
    let tokens = quote! {
        <div>{"foo"}</div>
    };

    let config = ParserConfig::new().transform_block(|_| Ok(None));
    let nodes = parse2_with_config(tokens, config);

    assert!(nodes.is_ok());

    Ok(())
}

#[test]
fn test_doctype() -> Result<()> {
    let tokens = quote! {
        <!DOCTYPE html>
        <html>
        </html>
    };

    let nodes = parse2(tokens)?;
    let Some(Node::Doctype(doctype)) = nodes.get(0) else { panic!("expected doctype") };

    assert_eq!(String::try_from(&doctype.value)?, "html");

    Ok(())
}

#[test]
fn test_comment() -> Result<()> {
    let tokens = quote! {
        <!-- "comment1" -->
        <div>
            <!-- "comment2" -->
            <div />
        </div>
    };

    let nodes = parse2(tokens)?;
    let Some(Node::Comment(comment1)) = nodes.get(0) else { panic!("expected comment") };
    let Node::Comment(comment2) =
        get_element_child(&nodes, 1, 0) else { panic!("expected comment") };

    assert_eq!(String::try_from(&comment1.value)?, "comment1");
    assert_eq!(String::try_from(&comment2.value)?, "comment2");

    Ok(())
}

#[test]
fn test_fragment() -> Result<()> {
    let tokens = quote! {
        <>
            <div />
        </>
    };

    let nodes = parse2(tokens)?;
    let Some(Node::Fragment(fragment)) = nodes.get(0) else { panic!("expected fragment") };

    assert_eq!(fragment.children.len(), 1);

    Ok(())
}

#[test]
fn test_reserved_keywords() -> Result<()> {
    let tokens = quote! {
        <tag::type attribute::type />
        <tag:type attribute:type />
        <tag-type attribute-type />
    };

    let nodes = parse2(tokens)?;

    assert_eq!(nodes.len(), 3);

    Ok(())
}

fn get_element(nodes: &[Node], element_index: usize) -> &NodeElement {
    let Some(Node::Element(element)) = nodes.get(element_index) else { panic!("expected element") };
    element
}

fn get_element_attribute(
    nodes: &[Node],
    element_index: usize,
    attribute_index: usize,
) -> &KeyedAttribute {
    let Some(Node::Element(element)) =
        nodes.get(element_index) else { panic!("expected element") };
    let Some(NodeAttribute::Attribute(attribute)) =
        element.attributes().get(attribute_index) else { panic!("expected attribute") };

    attribute
}

fn get_element_child(nodes: &[Node], element_index: usize, child_index: usize) -> &Node {
    let Some(Node::Element(element)) = nodes.get(element_index) else { panic!("expected element") };
    element.children.get(child_index).expect("child")
}
