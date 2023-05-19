use std::{convert::TryFrom, str::FromStr};

use eyre::Result;
use proc_macro2::TokenStream;
use quote::quote;
use rstml::{
    node::{KeyedAttribute, Node, NodeAttribute, NodeElement, NodeType},
    parse2, Parser, ParserConfig,
};
use syn::Block;

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
    assert_eq!(attribute.value_literal_string().expect("value"), "moo");

    Ok(())
}

#[test]
fn test_single_element_with_text() -> Result<()> {
    let tokens = quote! {
        <foo>"bar"</foo>
    };

    let nodes = parse2(tokens)?;
    let Node::Text(child) = get_element_child(&nodes, 0, 0) else { panic!("expected child") };

    assert_eq!(child.value.value(), "bar");

    Ok(())
}

#[test]
fn test_single_element_with_unquoted_text_simple() -> Result<()> {
    let tokens = quote! {
        // Note two spaces between bar and baz
        <foo> bar  baz </foo>
    };

    let nodes = parse2(tokens)?;
    let Node::RawText(child) = get_element_child(&nodes, 0, 0) else { panic!("expected child") };

    // We can't use source text if token stream was created with quote!.
    assert_eq!(child.to_token_stream_string(), "bar baz");
    assert_eq!(child.to_token_stream_string(), child.to_string_best());
    Ok(())
}

#[test]
fn test_single_element_with_unquoted_text_advance() -> Result<()> {
    let tokens = TokenStream::from_str(
        r#"
        <foo> bar  baz </foo>
        "#,
    )
    .unwrap();

    let nodes = parse2(tokens)?;
    let Node::RawText(child) = get_element_child(&nodes, 0, 0) else { panic!("expected child") };

    // source text should be available
    assert_eq!(child.to_source_text(true).unwrap(), " bar  baz ");
    assert_eq!(child.to_source_text(false).unwrap(), "bar  baz");
    assert_eq!(child.to_token_stream_string(), "bar baz");
    // When source is available, best should
    assert_eq!(child.to_string_best(), child.to_source_text(true).unwrap());
    Ok(())
}

macro_rules! test_unquoted {
    ($($name:ident => $constructor: expr => Node::$getter:ident($bind:ident) => $check:expr ;)* )  => {
        $(
            mod $name {
                use super::*;
                #[test]
                fn test_unquoted_text_mixed() -> Result<()> {
                    let tokens = TokenStream::from_str(
                        concat!("<foo> bar bar ", $constructor, " baz baz </foo>")
                    ).unwrap();

                    let nodes = parse2(tokens)?;
                    let Node::RawText(child1) = get_element_child(&nodes, 0, 0) else { panic!("expected unquoted child") };

                    let Node::$getter($bind) = get_element_child(&nodes, 0, 1) else { panic!("expected matcher child") };

                    let Node::RawText(child3) = get_element_child(&nodes, 0, 2) else { panic!("expected unquoted child") };

                    // source text should be available
                    assert_eq!(child1.to_source_text(true).unwrap(), " bar bar ");
                    assert_eq!(child1.to_source_text(false).unwrap(), "bar bar");
                    assert_eq!(child1.to_token_stream_string(), "bar bar");

                    // When source is available, best should
                    assert_eq!(child1.to_string_best(), child1.to_source_text(true).unwrap());

                    assert_eq!(child3.to_source_text(true).unwrap(), " baz baz ");
                    assert_eq!(child3.to_source_text(false).unwrap(), "baz baz");
                    assert_eq!(child3.to_token_stream_string(), "baz baz");
                    assert_eq!(child3.to_string_best(), child3.to_source_text(true).unwrap());
                    $check;
                    Ok(())
                }
            }
        )*
    }
}

// unified way to check that unquoted text will work within mixed content,
// For example unquoted text before quoted, before element, fragment, or block
test_unquoted!(
    text => "\"text\"" => Node::Text(v) => assert_eq!(v.value_string(), "text");

    empty_element => "<div/>" => Node::Element(v) => {
        assert!(v.close_tag.is_none());
        assert_eq!(v.open_tag.name.to_string(), "div");

        assert!(v.attributes().is_empty() );
        assert!(v.children.is_empty() );
    };
    block => "{ x + 1 }" => Node::Block(v) => {
        // check that block valid
        assert!(v.try_block().is_some());
    };

    few_elements => "<div> <basd/> </div>" => Node::Element(v) => {
        assert!(v.close_tag.is_some());
        assert_eq!(v.open_tag.name.to_string(), "div");

        assert!(v.attributes().is_empty() );
        assert!(v.children.len() == 1 );
        let Node::Element(child) = v.children[0].clone() else {
            panic!("Not a element")
        };
        assert!(child.attributes().is_empty() );
        assert!(child.children.is_empty() );
        assert_eq!(child.open_tag.name.to_string(), "basd");
        assert!(child.close_tag.is_none());
    };


);

#[test]
#[should_panic = "Parsing error: LexError"]
fn test_unqouted_unfinished_quote_failing() {
    // Quote should be finished.
    let _ = TokenStream::from_str(
        r#"
        <foo> bar\"  baz </foo>
        "#,
    )
    .expect("Parsing error");
}

#[test]
#[should_panic = "Parsing error: LexError"]
fn test_unqouted_unfinished_brace_failing() {
    // Brace should be finished.
    let _ = TokenStream::from_str(
        r#"
        <foo> bar{  baz </foo>
        "#,
    )
    .expect("Parsing error");
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
    let nodes = Parser::new(config).parse_simple(tokens)?;

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

// TODO: Is it really needed?
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

    let nodes = Parser::new(ParserConfig::new().number_of_top_level_nodes(2)).parse_simple(tokens);
    assert!(nodes.is_err());

    let tokens = quote! {
        <div>
            <div />
        </div>
        <div />
    };
    let nodes = Parser::new(ParserConfig::new().number_of_top_level_nodes(2).flat_tree())
        .parse_simple(tokens);
    assert!(nodes.is_ok());

    let tokens = quote! {
        <div />
    };
    let nodes = Parser::new(ParserConfig::new().number_of_top_level_nodes(2)).parse_simple(tokens);
    assert!(nodes.is_err());

    Ok(())
}

#[test]
fn test_type_of_top_level_nodes() -> Result<()> {
    let tokens = quote! {
        "foo"
    };
    let nodes = Parser::new(ParserConfig::new().type_of_top_level_nodes(NodeType::Element))
        .parse_simple(tokens);

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

    let nodes = Parser::new(config).parse_simple(tokens)?;
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
    let nodes = Parser::new(config).parse_simple(tokens);

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

    assert_eq!(doctype.value.to_token_stream_string(), "html");

    Ok(())
}

#[test]
fn test_doctype_empty() -> Result<()> {
    let tokens = quote! {
        <!DOCTYPE>
        <html>
        </html>
    };

    let nodes = parse2(tokens)?;
    let Some(Node::Doctype(doctype)) = nodes.get(0) else { panic!("expected doctype") };

    assert_eq!(doctype.value.to_token_stream_string(), "");

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

    assert_eq!(comment1.value.value(), "comment1");
    assert_eq!(comment2.value.value(), "comment2");

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

#[test]
fn test_single_element_with_different_attributes() -> Result<()> {
    let tokens = quote! {
        <foo bar="moo" baz=0x10 bax=true bay=0.1 foz='c' foy={x} fo1=b'c'></foo>
    };
    let nodes = parse2(tokens)?;

    let valid_values = vec![
        ("bar", "moo"),
        ("baz", "16"),
        ("bax", "true"),
        ("bay", "0.1"),
        ("foz", "c"),
    ];
    let valid_values_len = valid_values.len();
    for (ix, (name, value)) in valid_values.into_iter().enumerate() {
        let attribute = get_element_attribute(&nodes, 0, ix);

        assert_eq!(attribute.key.to_string(), name);
        assert_eq!(attribute.value_literal_string().expect("value"), value);
    }
    let values = vec!["foy", "fo1"];
    for (ix, name) in values.into_iter().enumerate() {
        let attribute = get_element_attribute(&nodes, 0, valid_values_len + ix);

        assert_eq!(attribute.key.to_string(), name);
        assert!(attribute.value_literal_string().is_none());
    }

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
