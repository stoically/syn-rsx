use std::{convert::TryFrom, str::FromStr};

use eyre::Result;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Block;
use syn_rsx::{Node, NodeAttribute, NodeBlock, Parser, ParserConfig};

#[test]
fn test_recover_incorrect_closing_tags() {
    let stream = quote!(<div><open></close><foo></foo></div>);

    let config = ParserConfig::new().recover_block(true);
    let parser = Parser::new(config);
    // by default parse return error
    assert!(parser.parse_simple(stream.clone()).is_err());

    let (nodes, _errors) = parser.parse_recoverable(stream).split_vec();
    assert_eq!(nodes.len(), 1);
    let Node::Element(e) = &nodes[0] else {
        panic!("Not element")
    };
    assert_eq!(e.children.len(), 2);
    let Node::Element(c) = &e.children[0] else {
        panic!("No child")
    };
    assert_eq!(c.open_tag.name.to_string(), "open");
    assert_eq!(c.close_tag.as_ref().unwrap().name.to_string(), "close");

    let Node::Element(c) = &e.children[1] else {
        panic!("No child")
    };
    assert_eq!(c.open_tag.name, c.close_tag.as_ref().unwrap().name);
    assert_eq!(c.open_tag.name.to_string(), "foo")
}

#[test]
fn test_parse_invalid_block() -> Result<()> {
    let tokens = TokenStream::from_str(
        "<foo>{x.}</foo>", // dot is not allowed
    )
    .unwrap();
    let config = ParserConfig::new().recover_block(true);
    let (nodes, errors) = Parser::new(config).parse_recoverable(tokens).split_vec();
    assert!(!errors.is_empty());

    let Node::Block(block) = &nodes[0].children().unwrap()[0] else { panic!("expected block") };

    assert!(block.try_block().is_none());

    assert!(Block::try_from(block.clone()).is_err());
    Ok(())
}

#[test]
fn test_parse_invalid_attr_block() -> Result<()> {
    let tokens = TokenStream::from_str(
        "<foo {x.} />", // dot is not allowed
    )
    .unwrap();
    let config = ParserConfig::new().recover_block(true);
    let (nodes, errors) = Parser::new(config).parse_recoverable(tokens).split_vec();

    assert!(!errors.is_empty());

    let Node::Element(f) = &nodes[0] else { panic!("expected element") };
    let NodeAttribute::Block(NodeBlock::Invalid { .. }) = f.attributes()[0] else {
        panic!("expected attribute")
    };
    Ok(())
}

// TODO: keyed attribute
