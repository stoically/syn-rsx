//!
//! Implementation of ToTokens and Spanned for node related structs

use std::convert::identity;

use proc_macro2::{extra::DelimSpan, Delimiter, Span, TokenStream, TokenTree};
use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::ToTokens;
use syn::{
    braced,
    parse::{discouraged::Speculative, Parse, ParseStream, Parser as _},
    spanned::Spanned,
    token::Brace,
    Block, Ident, LitStr, Token,
};

use super::{
    atoms::{
        token::{self, DocStart},
        CloseTag, FragmentClose, FragmentOpen, OpenTag,
    },
    raw_text::RawText,
    Node, NodeBlock, NodeDoctype, NodeFragment,
};
use crate::{
    config::TransformBlockFn,
    parser::recoverable::{ParseRecoverable, RecoverableContext},
    token::CloseTagStart,
    NodeAttribute, NodeElement,
};

impl ParseRecoverable for NodeBlock {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let fork = input.fork();

        let block = match parse_valid_block_expr(&fork) {
            Ok(value) => {
                input.advance_to(&fork);
                NodeBlock::ValidBlock(value.into())
            }
            Err(e) if parser.config().recover_block => {
                parser.push_diagnostic(e);
                let try_block = || {
                    let content;
                    Ok(NodeBlock::Invalid {
                        brace: braced!(content in input),
                        body: content.parse()?,
                    })
                };
                parser.save_diagnostics(try_block())?
            }
            Err(e) => {
                parser.push_diagnostic(e);
                return None;
            }
        };
        Some(block)
    }
}

impl ParseRecoverable for NodeFragment {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let tag_open: FragmentOpen = parser.parse_simple(input)?;

        let is_raw = |name| crate::context::with_config(|c| c.raw_text_elements.contains(name));

        let (mut children, tag_close) = if is_raw("") {
            let (child, closed_tag) =
                parser.parse_with_ending(input, |_, t| RawText::from(t), FragmentClose::parse);

            (vec![Node::RawText(child)], closed_tag)
        } else {
            parser.parse_tokens_until::<Node, _, _>(input, FragmentClose::parse)
        };
        let tag_close = tag_close?;
        let open_tag_end = tag_open.token_gt.span();
        let close_tag_start = tag_close.token_lt.span();
        let spans: Vec<Span> = Some(open_tag_end)
            .into_iter()
            .chain(children.iter().map(|n| n.span()))
            .chain(Some(close_tag_start))
            .collect();
        for (spans, children) in spans.windows(3).zip(&mut children) {
            match children {
                Node::RawText(t) => t.set_tag_spans(spans[0], spans[2]),
                _ => {}
            }
        }
        Some(NodeFragment {
            tag_open,
            children,
            tag_close,
        })
    }
}

impl ParseRecoverable for NodeDoctype {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let token_start = parser.parse_simple::<DocStart>(input)?;
        let doctype_keyword = parser.parse_simple::<Ident>(input)?;
        if doctype_keyword.to_string().to_lowercase() != "doctype" {
            parser.push_diagnostic(input.error("expected Doctype"));
            return None;
        }
        let (value, token_end) =
            parser.parse_with_ending(input, |_, t| RawText::from(t), <Token![>]>::parse);

        let token_end = token_end?;
        Some(Self {
            token_start,
            token_doctype: doctype_keyword,
            value,
            token_end,
        })
    }
}

impl ParseRecoverable for OpenTag {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let token_lt = parser.parse_simple::<Token![<]>(input)?;
        // Found closing tag when open tag was expected
        // keep parsing it as open tag.
        if input.peek(Token![/]) {
            let span = if let Ok(solidus) = input.parse::<Token![/]>() {
                solidus.span()
            } else {
                token_lt.span()
            };
            parser.push_diagnostic(Diagnostic::spanned(
                span,
                Level::Error,
                "close tag was parsed while waiting for open tag",
            ));
        }
        let name = parser.parse_simple(input)?;

        let (attributes, end_tag) =
            parser.parse_tokens_with_ending::<NodeAttribute, _, _>(input, token::OpenTagEnd::parse);

        if end_tag.is_none() {
            parser.push_diagnostic(Diagnostic::new(Level::Error, "expected end of tag '>'"));
        }
        end_tag.map(|end_tag| OpenTag {
            token_lt,
            name,
            attributes,
            end_tag,
        })
    }
}

impl ParseRecoverable for NodeElement {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let open_tag: OpenTag = parser.parse_recoverable(input)?;
        let is_known_self_closed =
            |name| crate::context::with_config(|c| c.always_self_closed_elements.contains(name));
        let is_raw = |name| crate::context::with_config(|c| c.raw_text_elements.contains(name));

        let tag_name_str = &*open_tag.name.to_string();
        if open_tag.is_self_closed() || is_known_self_closed(tag_name_str) {
            return Some(NodeElement {
                open_tag,
                children: vec![],
                close_tag: None,
            });
        }

        let (children, close_tag) = if is_raw(tag_name_str) {
            let (child, closed_tag) =
                parser.parse_with_ending(input, |_, t| RawText::from(t), CloseTag::parse);
            // don't keep empty RawText
            let children = if !child.is_empty() {
                vec![Node::RawText(child)]
            } else {
                vec![]
            };
            (children, closed_tag)
        } else {
            // If node is not raw use any closing tag as separator, to early report about
            // invalid closing tags.
            let (children, close_tag) =
                parser.parse_tokens_until::<Node, _, _>(input, CloseTagStart::parse);

            let close_tag = close_tag
                .map(|close_tag| CloseTag::parse_with_start_tag(input, close_tag))
                .transpose();
            let close_tag = parser.save_diagnostics(close_tag).and_then(identity);

            (children, close_tag)
        };

        let open_tag_end = open_tag.end_tag.token_gt.span();
        let close_tag_start = close_tag.as_ref().map(|c| c.start_tag.token_lt.span());
        let children = RawText::vec_set_context(open_tag_end, close_tag_start, children);

        let Some(close_tag) = close_tag else {
            let mut diagnostic = Diagnostic::spanned(open_tag.span(), Level::Error, "open tag has no coresponding close tag");
            if !children.is_empty() {
                let mut note_span = TokenStream::new();
                children.iter().for_each(|v|v.to_tokens(&mut note_span));
                diagnostic = diagnostic
                                .span_note(note_span.span(), "treating all inputs after open tag as it content");
            }

            parser.push_diagnostic(diagnostic);
            return Some(NodeElement {
                open_tag,
                children: children,
                close_tag: None,
            });
        };

        if close_tag.name != open_tag.name {
            let diagnostic =
                Diagnostic::spanned(close_tag.span(), Level::Error, "wrong close tag found")
                    .spanned_child(
                        open_tag.span(),
                        Level::Help,
                        "open tag that should be closed started there",
                    );

            parser.push_diagnostic(diagnostic)
        }
        let element = NodeElement {
            open_tag,
            children,
            close_tag: Some(close_tag),
        };
        Some(element)
    }
}

impl ParseRecoverable for Node {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self> {
        let node = if input.peek(Token![<]) {
            if input.peek2(Token![!]) {
                if input.peek3(Ident) {
                    Node::Doctype(parser.parse_recoverable(input)?)
                } else {
                    Node::Comment(parser.parse_simple(input)?)
                }
            } else if input.peek2(Token![>]) {
                Node::Fragment(parser.parse_recoverable(input)?)
            } else {
                Node::Element(parser.parse_recoverable(input)?)
            }
        } else if input.peek(Brace) {
            Node::Block(parser.parse_recoverable(input)?)
        } else if input.peek(LitStr) {
            Node::Text(parser.parse_simple(input)?)
        } else if !input.is_empty() {
            // Parse any input except of any other Node starting
            Node::RawText(parser.parse_simple(input)?)
        } else {
            return None;
        };
        Some(node)
    }
}

impl RecoverableContext {
    /// Parse array of toknes that is seperated by spaces(tabs, or new lines).
    /// Stop parsing array when other branch could parse anything.
    ///
    /// Example:
    /// ```no_build
    /// # use syn::{parse::{Parser, ParseStream}, Ident, Result, parse_macro_input, Token};
    /// # use syn_rsx::{parse_tokens_until};
    /// # fn main() -> syn::Result<()>{
    /// let tokens:proc_macro2::TokenStream = quote::quote!(few idents seperated by spaces and then minus sign - that will stop parsing).into();
    /// let concat_idents_without_minus = |input: ParseStream| -> Result<String> {
    ///     let (idents, _minus) = parser.parse_tokens_until::<Ident, _, _>(input, |i|
    ///         i.parse::<Token![-]>()
    ///     )?;
    ///     let mut new_str = String::new();
    ///     for ident in idents {
    ///         new_str.push_str(&ident.to_string())
    ///     }
    ///     // .. skip rest idents in input
    /// #    while !input.is_empty() {
    /// #        input.parse::<Ident>()?;
    /// #    }
    ///     Ok(new_str)
    /// };
    /// let concated = concat_idents_without_minus.parse2(tokens)?;
    /// assert_eq!(concated, "fewidentsseperatedbyspacesandthenminussign");
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_tokens_until<T, F, U>(
        &mut self,
        input: ParseStream,
        stop: F,
    ) -> (Vec<T>, Option<U>)
    where
        T: ParseRecoverable + std::fmt::Debug + Spanned,
        F: Fn(ParseStream) -> syn::Result<U>,
    {
        let mut collection = vec![];
        let res = loop {
            let old_cursor = input.cursor();
            let fork = input.fork();
            if let Ok(res) = stop(&fork) {
                input.advance_to(&fork);
                break Some(res);
            }
            if let Some(o) = self.parse_recoverable(input) {
                collection.push(o)
            }

            if old_cursor == input.cursor() {
                break None;
            }
        };
        (collection, res)
    }
    /// Two-phase parsing, firstly find separator, and then parse array of
    /// tokens before separator. For simple inputs method work like
    /// `parse_tokens_until`, but it creates intermediate TokenStream and
    /// copy of all tokens until separator token is found. It is usefull
    /// when separator (or it's part) can be treated as part of token T.
    ///
    ///
    /// Example:
    /// ```no_build
    /// let tokens = quote!(some_expr_seperated + with - lt_gt * tokens <> other part);
    /// ```
    /// In this example "<" can can be parsed as part of expression, but we want
    /// to split tokens after "<>" was found. So instead of parsing all
    /// input as expression, firstly we need to seperate it into two chunks.
    pub fn parse_tokens_with_ending<T, F, U>(
        &mut self,
        input: ParseStream,
        separator: F,
    ) -> (Vec<T>, Option<U>)
    where
        T: ParseRecoverable + std::fmt::Debug,
        F: Fn(ParseStream) -> syn::Result<U>,
    {
        let parser = |parser: &mut Self, tokens: TokenStream| {
            let parse = |input: ParseStream| {
                let mut collection = vec![];

                while !input.is_empty() {
                    let old_cursor = input.cursor();
                    if let Some(o) = parser.parse_recoverable(input) {
                        collection.push(o)
                    }
                    if old_cursor == input.cursor() {
                        break;
                    }
                }
                let eated_tokens = input.parse::<TokenStream>()?;
                Ok((collection, eated_tokens))
            };
            let (collection, eaten_tokens) = parse.parse2(tokens).expect("No errors allowed");
            if !eaten_tokens.is_empty() {
                parser.push_diagnostic(Diagnostic::spanned(
                    eaten_tokens.span(),
                    Level::Error,
                    "tokens was ignored during parsing",
                ))
            }
            collection
        };
        self.parse_with_ending(input, parser, separator)
    }

    pub fn parse_with_ending<F, CNV, V, U>(
        &mut self,
        input: ParseStream,
        parser: CNV,
        ending: F,
    ) -> (V, Option<U>)
    where
        F: Fn(ParseStream) -> syn::Result<U>,
        CNV: Fn(&mut Self, TokenStream) -> V,
    {
        let mut tokens = TokenStream::new();
        let res = loop {
            // Use fork, because we can't limit separator to be only Peekable for custom
            // tokens but we also need to parse complex expressions like
            // "foo=x/y" or "/>"
            let fork = input.fork();
            if let Ok(end) = ending(&fork) {
                input.advance_to(&fork);
                break Some(end);
            }

            if input.is_empty() {
                break None;
            }

            let next: TokenTree = self
                .parse_simple(input)
                .expect("TokenTree should always be parsable");
            tokens.extend([next]);
        };
        (parser(self, tokens), res)
    }
}

// This method couldn't be const generic until https://github.com/rust-lang/rust/issues/63569
/// Parse array of tokens with
pub(super) fn parse_array_of2_tokens<T: Parse>(input: ParseStream) -> syn::Result<[T; 2]> {
    Ok([input.parse()?, input.parse()?])
}

pub(super) fn to_tokens_array<I>(input: &mut TokenStream, iter: I)
where
    I: IntoIterator,
    I::Item: ToTokens,
{
    use quote::TokenStreamExt;
    input.append_all(iter)
}

/// Replace the next [`TokenTree::Group`] in the given parse stream with a
/// token stream returned by a user callback, or parse as original block if
/// no token stream is returned.
fn block_transform(input: ParseStream, transform_fn: &TransformBlockFn) -> syn::Result<Block> {
    input.step(|cursor| {
        let (block_group, block_span, next) = cursor
            .group(Delimiter::Brace)
            .ok_or_else(|| cursor.error("unexpected: no Group found"))?;
        let parser = move |block_content: ParseStream| {
            let forked_block_content = block_content.fork();

            match transform_fn(&forked_block_content) {
                Ok(transformed_tokens) => match transformed_tokens {
                    Some(tokens) => {
                        let parser = move |input: ParseStream| {
                            Ok(block_expr_with_extern_span(input, block_span))
                        };
                        let transformed_content = parser.parse2(tokens)?;
                        block_content.advance_to(&forked_block_content);
                        transformed_content
                    }
                    None => block_expr_with_extern_span(block_content, block_span),
                },
                Err(error) => Err(error),
            }
        };

        Ok((parser.parse2(block_group.token_stream())?, next))
    })
}

fn parse_valid_block_expr(input: syn::parse::ParseStream) -> syn::Result<Block> {
    let transform_block = crate::context::with_config(|c| c.transform_block.clone());
    let value = if let Some(transform_fn) = transform_block {
        block_transform(input, &*transform_fn)?
    } else {
        block_expr(input)?
    };
    Ok(value)
}
/// Parse the given stream and span as [`Expr::Block`].
fn block_expr_with_extern_span(input: ParseStream, span: DelimSpan) -> syn::Result<Block> {
    Ok(Block {
        brace_token: Brace { span },
        stmts: Block::parse_within(input)?,
    })
}

/// Parse the given stream as [`Expr::Block`].
pub(crate) fn block_expr(input: syn::parse::ParseStream) -> syn::Result<Block> {
    let content;
    let brace_token = braced!(content in input);
    Ok(Block {
        brace_token,
        stmts: Block::parse_within(&content)?,
    })
}
