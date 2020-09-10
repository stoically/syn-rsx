# syn-rsx

[![crates.io page](https://img.shields.io/crates/v/syn-rsx.svg)](https://crates.io/crates/syn-rsx)
[![docs.rs page](https://docs.rs/syn-rsx/badge.svg)](https://docs.rs/syn-rsx/)
![build](https://github.com/stoically/syn-rsx/workflows/build/badge.svg)
![license: MIT](https://img.shields.io/crates/l/syn-rsx.svg)

[syn](https://github.com/dtolnay/syn)-powered parser for JSX-like [TokenStreams](https://doc.rust-lang.org/proc_macro/struct.TokenStream.html), aka RSX. The parsed result is a nested `Node` structure, similar to the browser DOM, where node name and value are syn expressions to support building proc macros. 

```rust
use quote::quote;
use syn_rsx::parse2;

let tokens = quote! { <hello world>"hi"</hello> };

let nodes = parse2(tokens).unwrap();
assert_eq!(nodes[0].name_as_string().unwrap(), "hello");
assert_eq!(nodes[0].attributes[0].name_as_string().unwrap(), "world");
assert_eq!(nodes[0].children[0].value_as_string().unwrap(), "hi");
```

## Features


- **Not opinionated**

  Every tag or attribute name is valid

  ```html
  <hello world />
  ```

- **Text nodes**

  Support for [unquoted text is planned] (currently requires Rust nightly)

  ```html
  <div>"String literal"</div>
  ```

- **Node names separated by dash or colon**

  ```html
  <tag-name attribute-key="value" />
  <tag:name attribute:key="value" />
  ```

- **Node names as [mod style path]**

  ```html
  <tag::path attribute::path="value" />
  ```

- **Node names with reserved keywords**

  ```html
  <input type="submit" />
  ```

- **Attribute values can be any valid syn expression without requiring braces**

  ```html
  <div key=some::value() />
  ```

- **Braced blocks are parsed as arbitrary Rust code**

  ```html
  <div>{ let block = "in node position"; }</div>
  <div { let block = "in attribute position"; } />
  <div key={ let block = "in attribute value position"; } />
  ```

- **Helpful error reporting out of the box**

  ```rust,no-run
  error: open tag has no corresponding close tag and is not self-closing
  --> examples/html-to-string-macro/tests/lib.rs:5:24
    |
  5 |     html_to_string! { <div> };
    |                        ^^^
  ```

- **Customization**

  A `ParserConfig` to customize parsing behavior is available, so if you have
  slightly different requirements for parsing and it's not yet customizable
  feel free to open an issue or pull request to extend the configuration.


[`syn`]: /syn
[`TokenStream`]: https://doc.rust-lang.org/proc_macro/struct.TokenStream.html
[mod style path]: https://docs.rs/syn/1.0.40/syn/struct.Path.html#method.parse_mod_style
[unquoted text is planned]: https://github.com/stoically/syn-rsx/issues/2
