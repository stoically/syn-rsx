# rstml

[![crates.io page](https://img.shields.io/crates/v/rstml.svg)](https://crates.io/crates/rstml)
[![docs.rs page](https://docs.rs/rstml/badge.svg)](https://docs.rs/rstml/)
[![codecov](https://codecov.io/gh/rs-tml/rstml/branch/main/graph/badge.svg?token=2LMJ8YEV92)](https://codecov.io/gh/rs-tml/rs-tml)
![build](https://github.com/rs-tml/rstml/workflows/ci/badge.svg)
![license: MIT](https://img.shields.io/crates/l/rstml.svg)
Rust templating for XML-based formats (HTML, SVG, MathML) implemented on top of [`proc-macro::TokenStream`]s.
Similar to JSX but for Rust (commonly named RSX).The parsed result is a nested `Node` structure, similar to the browser DOM, where node name and value are syn expressions to support building proc macros.

The fork of original [syn-rsx](https://github.com/stoically/syn-rsx) repo.
It was created because of various reasons:
- The author of original `syn-rsx` retired, end left new features without attention https://github.com/stoically/syn-rsx/issues/49
- Syn v2 was released and `syn-rsx` need to be mooved to new version.
- The idea of [lossless parsing](https://github.com/stoically/syn-rsx/issues/53) was left unattended.
- [Unquoted text](https://github.com/stoically/syn-rsx/issues/2) feature should advance.
- Interest in recoverable parsing and better IDE support.


See [comparsion](/comparsion-with-syn-rsx.md) for more detail.

```rust
use std::convert::TryFrom;

use eyre::bail;
use quote::quote;
use rstml::{parse2, Node, NodeAttribute, NodeElement, NodeText};

// Create HTML `TokenStream`.
let tokens = quote! { <hello world>"hi"</hello> };

// Parse the tokens into a tree of `Node`s.
let nodes = parse2(tokens)?;

// Extract some specific nodes from the tree.
let Node::Element(element) = &nodes[0] else { bail!("element") };
let Node::Attribute(attribute) = &element.attributes[0] else { bail!("attribute") };
let Node::Text(text) = &element.children[0] else { bail!("text") };

// Work with the nodes.
assert_eq!(element.name.to_string(), "hello");
assert_eq!(attribute.key.to_string(), "world");
assert_eq!(String::try_from(&text.value)?, "hi");
```

You might want to check out the [html-to-string-macro example] as well.

## Features

- **Not opinionated**

  Every tag or attribute name is valid

  ```html
  <hello world />
  ```

- **Text nodes**


  ```html
  <div>"String literal"</div>
  ```

- **Unquoted text nodes**

  Unquoted text is supported with few limitations:
  - Only valid Rust TokenStream can be unquoted text (no single quote text is supported, no unclosed braces, etc.)
  - Unquoted text not always can save spaces. It uses [`Span::source_text`] and [`Span::join`] to retrive info about spaces, and it is not always available.
  - Quoted text near unquoted treated as diferent Node, end library user should decide whenever to preserve quotation.

  ```html
  <div> Some string that is valid rust token stream </div>
  ```

- **Node names separated by dash, colon or double colon**

  ```html
  <tag-name some:attribute-key="value" />
  <tag::name attribute::key="value" />
  ```

- **Node names with reserved keywords**

  ```html
  <input type="submit" />
  ```

- **Doctypes, Comments and Fragments**

  ```html
  <!DOCTYPE html>
  <!-- "comment" -->
  <></>
  ```

- **Braced blocks are parsed as arbitrary Rust code**

  ```html
  <{ let block = "in node name position"; } />
  <div>{ let block = "in node position"; }</div>
  <div { let block = "in attribute position"; } />
  <div key={ let block = "in attribute value position"; } />
  ```

- **Attribute values can be any valid syn expression without requiring braces**

  ```html
  <div key=some::value() />
  ```

- **Helpful error reporting out of the box**

  ```rust,no-run
  error: open tag has no corresponding close tag and is not self-closing
  --> examples/html-to-string-macro/tests/lib.rs:5:24
    |
  5 |     html_to_string! { <div> };
    |                        ^^^
  ```

- **Possibility to get the span for a whole node**

  This can be used to improve error reporting, e.g.

  ```rust,no-run
  error: Invalid element
  --> examples/src/main.rs:14:13
     |
  14 | /             <div>
  15 | |                 "invalid node for some consumer specific reason"
  16 | |             </div>
     | |__________________^
     
  ```

- **Recoverable parser**

  Can parse html with multiple mistakes.
  As result library user get array of errors that can be reported, and tree of nodes that was parsed.

  ```html
   <div hello={world.} /> <!-- dot after world is invalid syn expression -->
    <>
        <div>"1"</x> <!-- incorrect closed tag -->
        <div>"2"</div>
        <div>"3"</div>
        <div {"some-attribute-from-rust-block"}/>
    </>
  ```

  Using this feature one can write macro in IDE friendly way.
  This macro will work faster (because on invalid syntax it change output slightly, instead of removing it completely, so IDE can check diff quicly). And give completion (goto definition, and other semantic related feature) more often.

- **Customization**

  A `ParserConfig` to customize parsing behavior is available, so if you have
  slightly different requirements for parsing and it's not yet customizable
  feel free to open an issue or pull request to extend the configuration.

  One highlight with regards to customization is the `transform_block`
  configuration, which takes a closure that receives raw block content as
  `ParseStream` and lets you optionally convert it to a `TokenStream`. That makes it
  possible to have custom syntax in blocks. More details in [#9]

[`syn`]: https://github.com/dtolnay/syn
[`Span::join`]: https://doc.rust-lang.org/proc_macro/struct.Span.html#method.join
[`Span::source_text`]: https://doc.rust-lang.org/proc_macro/struct.Span.html#method.source_text
[`tokenstream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
[html-to-string-macro example]: https://github.com/stoically/syn-rsx/tree/main/examples/html-to-string-macro
[#9]: https://github.com/stoically/syn-rsx/issues/9
