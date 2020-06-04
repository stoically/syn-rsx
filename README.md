# syn-rsx

[![crates.io page](https://img.shields.io/crates/v/syn-rsx.svg)](https://crates.io/crates/syn-rsx)
[![docs.rs page](https://docs.rs/syn-rsx/badge.svg)](https://docs.rs/syn-rsx/)
![build](https://github.com/stoically/syn-rsx/workflows/build/badge.svg)
![license: MIT](https://img.shields.io/crates/l/syn-rsx.svg)

[syn](https://github.com/dtolnay/syn)-powered parser for JSX-like [TokenStreams](https://doc.rust-lang.org/proc_macro/struct.TokenStream.html). The parsed result is a nested `Node` structure, similar to the browser DOM, where node name and value are syn expressions to support building proc macros. A `ParserConfig` to customize parsing behavior is available, so if you have slightly different requirements for parsing and it's not yet customizable feel free to open an issue or pull request to extend the configuration.

```rust
use quote::quote;
use syn_rsx::parse2;

let tokens = quote! {
    <div foo={bar}>
        <div>"hello"</div>
        <world />
    </div>
};

let nodes = parse2(tokens, None).unwrap();

let node = &nodes[0];
assert_eq!(node.attributes[0].name_as_string().unwrap(), "foo");

let children = &node.children;
assert_eq!(children.len(), 2);
assert_eq!(children[0].children[0].value_as_string().unwrap(), "hello");
assert_eq!(children[1].name_as_string().unwrap(), "world");
```
