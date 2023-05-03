use proc_macro2::{Span, TokenStream};
use syn::{parse::ParseStream, spanned::Spanned, token::Brace, Expr, Lit};

use crate::{NodeBlock, NodeName};

///
/// Element attribute with fixed key.
///
/// Example:
/// key=value // attribute with ident as value
/// key // attribute without value
#[derive(Clone, Debug)]
pub struct KeyedAttribute {
    /// Key of the element attribute.
    pub key: NodeName,
    /// Value of the element attribute.
    pub value: Option<Expr>,
    /// Source span of the attribute for error reporting.
    ///
    /// Note: This should cover the entire node in nightly, but is a "close
    /// enough" approximation in stable until [Span::join] is stabilized.
    pub span: Span,
}
impl KeyedAttribute {
    ///
    /// Returns string representation of inner value,
    /// if value expression contain something that can be treated as displayable
    /// literal.
    ///
    /// Example of displayable literals:
    /// `"string"`      // string
    /// `'c'`           // char
    /// `0x12`, `1231`  // integer - converted to decimal form
    /// `0.12`          // float point value - converted to decimal form
    /// `true`, `false` // booleans
    ///
    /// Examples of literals that also will be non-displayable:
    /// `b'a'`     // byte
    /// `b"asdad"` // byte-string
    ///
    /// Examples of non-static non-displayable expressions:
    /// `{ x + 1}`     // block of code
    /// `y`            // usage of variable
    /// `|v| v + 1`    // closure is valid expression too
    /// `[1, 2, 3]`    // arrays,
    /// `for/while/if` // any controll flow
    /// .. and this list can be extended
    ///
    /// Adapted from leptos
    pub fn value_literal_string(&self) -> Option<String> {
        self.value.as_ref().and_then(|v| match v {
            Expr::Lit(l) => match &l.lit {
                Lit::Str(s) => Some(s.value()),
                Lit::Char(c) => Some(c.value().to_string()),
                Lit::Int(i) => Some(i.base10_digits().to_string()),
                Lit::Float(f) => Some(f.base10_digits().to_string()),
                Lit::Bool(b) => Some(b.value.to_string()),
                _ => None,
            },
            _ => None,
        })
    }

    // Checks if error is about eof.
    // This error is known to report Span::call_site.
    // Correct them to point to ParseStream
    pub(crate) fn correct_expr_error_span(error: syn::Error, input: ParseStream) -> syn::Error {
        let error_str = error.to_string();
        if error_str.starts_with("unexpected end of input") {
            let stream = input
                .parse::<TokenStream>()
                .expect("BUG: Token stream should always be parsable");
            return syn::Error::new(
                stream.span(),
                format!("failed to parse expression: {}", error),
            );
        }
        error
    }
}

/// Sum type for Dyn and Keyed attributes.
///
/// Attributes is stored in opening tags.
#[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
pub enum NodeAttribute {
    ///
    /// Element attribute that is computed from rust code block.
    ///
    /// Example:
    /// {"some-fixed-key"} // attribute without value that is computed from
    /// string
    #[parse(peek = Brace)]
    Block(NodeBlock),
    Attribute(KeyedAttribute),
}
