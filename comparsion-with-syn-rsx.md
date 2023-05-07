This fork has a lot of refactoring and features in it, to summarize differences this file was created:


## Syn_v2 released

In [March syn v2 was released]( https://github.com/dtolnay/syn/releases/tag/2.0.0). 
Rust language evolves and parsing library should adopt to this changes. 
And updating dependencies generally sounds like a good idea.

## Lossless parser v1

One of the ideas behind this fork was to implement all types in [lossless form](https://github.com/stoically/syn-rsx/issues/53),
and to provide single `syn::Parse` implementation for them.
In short example it looks like Parse implementation on types:

```rust
pub struct OpenTag {
    pub less_sign: Token![<],
    pub tag_name: NodeName,
    pub attributes: Vec<Attribute>,
    pub solidus: Option<Token![/]>,
    pub great_sign: Token![>],
}
pub struct CloseTag {
    pub less_sign: Token![<],
    pub solidus: Token![/],
    pub tag_name: NodeName,
    pub great_sign: Token![>],
}

pub struct NodeElement {
     pub open_tag: OpenTag,
     pub close_tag: Option<CloseTag>,
}
```

This was done in: https://github.com/vldm/syn-rsx/commit/ff2fb40692e149a2f6112d82bed31965119b6305
And after it refactored to use syn_derive instead of handwritten implementation in: https://github.com/vldm/syn-rsx/commit/6806a71997dc3fcce09069d597da7cadce92896a


Since that, everybody can reuse this parse to write write custom routine and parse mixed content, something like this, that is hard to achieve with custom entrypoint,:
`parse_expr_and_few_nodes!(x+1 => <div>$val</div>, x+2 <div>$val2</div>)`

As one of benefit of lossless representation - is ability to compute full `Span` even if `Span::join` is not available.
Also now all types implement ToTokens, so it's now easier to implement macro that will process some input, modify it and save it for future use in inner macro. All this with saving original spans.


## Unquoted text

One of the most demanded feature in syn-rsx was ["unquoted text"](https://github.com/stoically/syn-rsx/issues/2).
Its the ability to write text without quotes inside html tags.

```
<div> Some text inside div </div>

<style>
    div {
      background-color: red;
      color: white;
    }
</style>
<script>
var x = 12;
console.log(x);
if (x>1) {
  console.log(y);
}
</script>
```

This feature is crucial for parsing `<script>` and `<style>` tags.

Currently it is implemented with few drawbacks, see `node::RawText` for more details.


## Better diagnostic and recoverable parser
One of aim of this fork was to provide better IDE support. 
But originally syn-rsx uses fail-fast method to handle errors during parsing.
This design will not spam errors if you have broken syntax, and fail on first unexpected token.
It is enforced by `syn::Parse` and `syn::Parser`, and easy to implement - just propagate result and you got it.

This design is acceptable if used in compiler context but it has weak support in IDE.
Internally when you write anything inside a macro, IDE (rust-analyzer, InteliJ) will try to evaluate macro
to provide you completion and syntax highlighting.
If macro failed to evaluate - it has nothing to check.
Even worse, if you change macro from "working" to "broken" state by typing,
it also internally produces a big diff and IDE can work slowly.
There few [practicies](https://github.com/rust-lang/rust-analyzer/issues/11014) to made macro more IDE friendly.
One of them is make macro recover after invalid token.

This is done by introducing `ParseRecoverable` trait which is `syn::Parse` alternative but with the ability to save more then one error.
Also changing trait from `syn::Parse` to `ParseRecoverable` allows to switch from `syn::Error` to `Diagnostic`, which can provide not only compile_errors, but also Notes, Help and Warning information.
Currently this crate uses `Diagnostic` to provide context about unclosed tags, or closed tags that wasn't expected.

### Example
```rust,no_run
    error: wrong close tag found
    --> examples/html-to-string-macro/tests/tests.rs:23:33
    |
    23 |                         <div>"1"</xad>
    |                                 ^^^^^^
    |
    help: open tag that should be closed started there
    --> examples/html-to-string-macro/tests/tests.rs:23:25
    |
    23 |                         <div>"1"</xad>
    |                         ^^^^^
    error: could not compile `html-to-string-macro` (test "tests") due to previous error
```


### NodeBlock rework

One of the common problem in making syn-rsx more ide friendly, was parsing of `syn::Expr` especially one in braces {} (`syn::Block`).
By default `syn` parse only valid syntax, but when you write a code, you makes it temporary invalid, the most common example is a dot `{x.}`.
After which `syn` expect method call, or field access, so if you tries parse expr like `{x.}` in syn you will get an error.
In fail-fast design emiting error equals to no output from macro that equals to no IDE support.
To make `NodeBlock` more ide friendly `NodeBlock::Invalid` was implemented - it allows yo parse any tokens inside braces as `NodeBlock`, to emit them in future, even if there was invalid `syn::Expr`.
To enable it `ParserConfig::recover_block` should be set to true.

as a result:
![Completion](/doc_imgs/completion.png)

Completion work even if syntax is broken.

### Example to sumarize recoverable parsing:
in code like this, where few errors exist (not valid expr, invlid closing, non)
```rust 
html! {
    <!DOCTYPE html>
    <html>
        <head>
            <title>"Example"</title>
        </head>
        <body>
            <!-- "comment" -->
                <div x=x a=2 x=3 >
                    <div>"1" { world.} "2" </div>
                </div>
                    <div x=a some text as attribute flags and unclosed tag  "2" </div>
                <div x =a > some unquoted text with quotes "3". <div/></div>
                <div {"some-attribute-from-rust-block"}/>
                <div {"some-attribute-from-rust-blocks22".}>"3"</div>

            <div hello=world x=x >
            </br>
            </x>
        </body>
    </html>

}
```

<div style="width: 100%;">
  <img src="/doc_imgs/output.svg" style="width: 100%;" alt="Click to see the source">
</div>

# Minor improvements

- "examples/html-to-string-macro" now contain compiletests 
- Can now parse rust-lang.org html as input (with slightly modifications), and added bench to check speed of real world html.
- Only string literal is now allowed inside `Node::Text` (no more int literal, or float)
- One can now define tags like `<script>` or `<style>` (the list is defined by user) as raw. 
    In that case only one `RawText` child is allowed for this elements.
- One can now set list of elements that is guaranteed to be self closed even if slash is missing. It's important feature for parsing html tags like `<br>`,`<link>` or `<meta>`