# syn-rsx

[![crates.io page](https://img.shields.io/crates/v/syn-rsx.svg)](https://crates.io/crates/syn-rsx)
[![docs.rs page](https://docs.rs/syn-rsx/badge.svg)](https://docs.rs/syn-rsx/)
![build](https://github.com/stoically/syn-rsx/workflows/build/badge.svg)
![license: MIT](https://img.shields.io/crates/l/syn-rsx.svg)

[syn](https://github.com/dtolnay/syn)-powered parser for JSX-like [TokenStreams](https://doc.rust-lang.org/proc_macro/struct.TokenStream.html). The parsed result is a nested `Node` structure, similar to the browser DOM. The `node_value` is an [`syn::Expr`](https://docs.rs/syn/latest/syn/enum.Expr.html).

```rust
use syn_rsx::parse2;
use quote::quote;

let tokens = quote! {
    <div>
        <div>"hello"</div>
        <world />
    </div>
};

let nodes = parse2(tokens, None).unwrap();
assert_eq!(nodes[0].child_nodes.len(), 2);
assert_eq!(nodes[0].child_nodes[1].node_name, "world");
```
