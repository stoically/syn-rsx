# syn-rsx

[syn](https://github.com/dtolnay/syn)-powered parser for JSX-like [TokenStreams](https://doc.rust-lang.org/proc_macro/struct.TokenStream.html). The parsed result is a nested `Node` structure modelled after the browser DOM.

```rust
use syn_rsx::parse2;
use quote::quote;

let tokens = quote! {
    <div>
        <div>"hello"</div>
        <div>{world}</div>
    </div>
};

let nodes = parse2(tokens, None).unwrap();
assert_eq!(nodes.get(0).unwrap().child_nodes.len(), 2);
```
