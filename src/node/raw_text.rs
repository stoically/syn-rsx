use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    LitStr, Token,
};

use crate::Node;

/// Raw unquoted text
///
/// Use source_text to retrieve spaces from text. (Cant be used in `quote!`
/// context) If source text is not available uses `Display` implementation of
/// `TokenStream`. If source_text is available and macro from single file, it
/// also can retrieve whitespaces.
///
/// Internally it is valid `TokenStream` (stream of rust code tokens).
/// So, it has few limitations:
/// 1. It cant contain any unclosed branches, braces or parens.
/// 2. Some tokens like ' ` can be treated as invalid, because in rust it only
/// allowed in certain contexts.
#[derive(Clone, Debug, Default)]
pub struct RawText {
    token_stream: TokenStream,
    // Span that started before previous token, and after next.
    context_span: Option<(Span, Span)>,
}
impl RawText {
    pub(crate) fn set_tag_spans(&mut self, before: impl Spanned, after: impl Spanned) {
        // todo: use span.after/before when it will be available in proc_macro2
        // for now just join full span an remove tokens from it.
        self.context_span = Some((before.span(), after.span()));
    }

    /// Convert to string using Display implementation of inner token stream.
    pub fn to_token_stream_string(&self) -> String {
        self.token_stream.to_string()
    }

    /// Try to get source text of the token stream.
    /// Optionally including witespaces.
    pub fn to_source_text(&self, with_witespaces: bool) -> Option<String> {
        if with_witespaces {
            let (start, end) = self.context_span?;
            let full = start.join(end)?;
            let full_text = full.source_text()?;
            let start_text = start.source_text()?;
            let end_text = end.source_text()?;
            debug_assert!(full_text.ends_with(&end_text));
            debug_assert!(full_text.starts_with(&start_text));
            Some(full_text[start_text.len()..(full_text.len() - end_text.len())].to_string())
        } else {
            self.token_stream.span().source_text()
        }
    }
    pub fn is_empty(&self) -> bool {
        self.token_stream.is_empty()
    }

    pub fn vec_set_context(
        open_tag_end: Span,
        close_tag_start: Option<Span>,
        mut children: Vec<Node>,
    ) -> Vec<Node> {
        let spans: Vec<Span> = Some(open_tag_end)
            .into_iter()
            .chain(children.iter().map(|n| n.span()))
            .chain(close_tag_start)
            .collect();

        for (spans, children) in spans.windows(3).zip(&mut children) {
            match children {
                Node::RawText(t) => t.set_tag_spans(spans[0], spans[2]),
                _ => {}
            }
        }
        children
    }

    /// Trying to return best string representation available:
    /// 1. calls `to_source_text(true)`
    /// 2. calls `to_source_text(false)`
    /// 3. as fallback calls `to_token_stream_string()`
    pub fn to_string_best(&self) -> String {
        self.to_source_text(true)
            .or_else(|| self.to_source_text(false))
            .unwrap_or_else(|| self.to_token_stream_string())
    }
}

impl Parse for RawText {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut token_stream = TokenStream::new();
        let any_node =
            |input: ParseStream| input.peek(Token![<]) || input.peek(Brace) || input.peek(LitStr);
        // Parse any input until catching any node.
        // Fail only on eof.
        while !any_node(input) && !input.is_empty() {
            token_stream.extend([input.parse::<TokenTree>()?])
        }
        Ok(Self {
            token_stream,
            context_span: None,
        })
    }
}

impl ToTokens for RawText {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_stream.to_tokens(tokens)
    }
}

impl From<TokenStream> for RawText {
    fn from(token_stream: TokenStream) -> Self {
        Self {
            token_stream,
            context_span: None,
        }
    }
}
