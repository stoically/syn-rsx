//!
//! Tokens that is used as parts of nodes, to simplify parsing.
//! Example:
//! `<!--` `<>` `</>` `<!` `/>`
//!
//! Also contain some entities that split parsing into several small units,
//! like: `<open_tag attr />`
//! `</close_tag>`

use proc_macro2::Ident;
use proc_macro2_diagnostics::{Diagnostic, Level};
use syn::{ext::IdentExt, Token};

use crate::{
    node::{parse, NodeAttribute, NodeName},
    parser::recoverable::RecoverableContext,
};

pub(crate) mod tokens {
    //! Custom syn punctuations
    use syn::{custom_punctuation, Token};

    use crate::node::parse;
    // Dash between node-name
    custom_punctuation!(Dash, -);

    // Later use custom punct, currently it is not compatible with quote;

    // // Start part of doctype tag
    //     // `<!`
    //     custom_punctuation!(DocStart, <!);

    //     // Start part of element's close tag.
    //     // Its commonly used as separator
    //     // `</`
    //     custom_punctuation!(CloseTagStart, </);

    //     custom_punctuation!(ComEnd, -->);

    //     //
    //     // Rest tokens is impossible to implement using custom_punctuation,
    //     // because they have Option fields, or more than 3 elems
    //     //
    /// Start part of doctype tag
    /// `<!`
    #[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
    pub struct DocStart {
        pub token_lt: Token![<],
        pub token_not: Token![!],
    }

    /// Start part of comment tag
    /// `<!--`
    #[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
    pub struct ComStart {
        pub token_lt: Token![<],
        pub token_not: Token![!],
        #[parse(parse::parse_array_of2_tokens)]
        #[to_tokens(parse::to_tokens_array)]
        pub token_minus: [Token![-]; 2],
    }

    /// End part of comment tag
    /// `-->`
    #[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
    pub struct ComEnd {
        #[parse(parse::parse_array_of2_tokens)]
        #[to_tokens(parse::to_tokens_array)]
        pub token_minus: [Token![-]; 2],
        pub token_gt: Token![>],
    }

    /// End part of element's open tag
    /// `/>` or `>`
    #[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
    pub struct OpenTagEnd {
        pub token_solidus: Option<Token![/]>,
        pub token_gt: Token![>],
    }

    /// Start part of element's close tag.
    /// Its commonly used as separator
    /// `</`
    #[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
    pub struct CloseTagStart {
        pub token_lt: Token![<],
        pub token_solidus: Token![/],
    }
}

pub use tokens::*;

/// Fragment open part
/// `<>`
#[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
pub struct FragmentOpen {
    pub token_lt: Token![<],
    pub token_gt: Token![>],
}

/// Fragment close part
/// `</>`
#[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
pub struct FragmentClose {
    pub start_tag: tokens::CloseTagStart,
    pub token_gt: Token![>],
}

impl FragmentClose {
    pub fn parse_with_start_tag(
        parser: &mut RecoverableContext,
        input: syn::parse::ParseStream,
        start_tag: Option<tokens::CloseTagStart>,
    ) -> Option<Self> {
        let start_tag = start_tag?;
        if input.peek(Ident::peek_any) {
            let ident_from_invalid_closing = Ident::parse_any(input).expect("parse after peek");
            parser.push_diagnostic(Diagnostic::spanned(
                ident_from_invalid_closing.span(),
                Level::Error,
                "expected fragment closing, found element closing tag",
            ));
        };
        Some(Self {
            start_tag,
            token_gt: parser.save_diagnostics(input.parse())?,
        })
    }
}

/// Open tag for element, possibly self-closed.
/// `<name attr=x, attr_flag>`
#[derive(Clone, Debug, syn_derive::ToTokens)]
pub struct OpenTag {
    pub token_lt: Token![<],
    pub name: NodeName,
    #[to_tokens(parse::to_tokens_array)]
    pub attributes: Vec<NodeAttribute>,
    pub end_tag: tokens::OpenTagEnd,
}

impl OpenTag {
    pub fn is_self_closed(&self) -> bool {
        self.end_tag.token_solidus.is_some()
    }
}

/// Open tag for element, `<name attr=x, attr_flag>`
#[derive(Clone, Debug, syn_derive::Parse, syn_derive::ToTokens)]
pub struct CloseTag {
    pub start_tag: tokens::CloseTagStart,
    pub name: NodeName,
    pub token_gt: Token![>],
}

impl CloseTag {
    pub fn parse_with_start_tag(
        parser: &mut RecoverableContext,
        input: syn::parse::ParseStream,
        start_tag: Option<tokens::CloseTagStart>,
    ) -> Option<Self> {
        Some(Self {
            start_tag: start_tag?,
            name: parser.save_diagnostics(input.parse())?,
            token_gt: parser.save_diagnostics(input.parse())?,
        })
    }
}

#[cfg(test)]
mod test {
    use syn::custom_punctuation;

    use super::*;

    macro_rules! parse_quote {
            ($mod_name:ident, $name: ident=> $($tts:tt)*) => {
                mod $mod_name {
                    use super::*;
                    // use super::tokens::*;
                    #[test]
                    fn parse_quote() {

                        let tts = quote::quote!{
                            $($tts)*
                        };
                        syn::parse2::<$name>(tts).unwrap();

                    }
                }
            }
        }

    parse_quote! {docstart, DocStart => <!}

    parse_quote! {comstart, ComStart => <!--}

    parse_quote! {comend, ComEnd => -->}

    parse_quote! {open_tag_end1, OpenTagEnd => >}

    parse_quote! {open_tag_end2, OpenTagEnd => />}

    parse_quote! {close_tag_start, CloseTagStart => </}

    /// Custom punctuation wasnt compatible with quote,
    /// check if it now compatible to replace simple parser with it  
    #[test]
    fn parse_quote_doc_comp_custom_punct() {
        custom_punctuation!(CloseTagStart2, </);
        let tts = quote::quote! {
            </
        };
        syn::parse2::<CloseTagStart2>(tts).unwrap_err();
    }
}
