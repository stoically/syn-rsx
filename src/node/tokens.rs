//!
//! Implementation of ToTokens and Spanned for node related structs

use proc_macro2::{TokenStream, Punct, TokenTree};
use quote::{quote_spanned, ToTokens};
use syn::{parse::{Parse, discouraged::Speculative, ParseStream, Parser as _}, Ident, ExprBlock, braced, Block, Token, Error, token::{Brace, PathSep, Colon}, spanned::Spanned, ExprPath, Path, PathSegment, punctuated::Punctuated, ext::IdentExt, ExprLit};

use crate::{NodeValueExpr, NodeElement, NodeAttribute, Parser, punctuation::Dash};

use super::{Node, NodeBlock, NodeComment, NodeDoctype, NodeFragment, NodeName, NodeText, attribute::{KeyedAttribute}, atoms::{FragmentOpen, FragmentClose, token::{self, DocStart, ComStart, ComEnd}, OpenTag, CloseTag}};


impl ToTokens for KeyedAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {

        let key = &self.key;
        let value = &self.value;

        // self closing
        if let Some(value) = value{
            tokens.extend(quote_spanned!(self.span => 
            #key = #value ))
        } else {
            tokens.extend(quote_spanned!(self.span => 
            #key ))
        }
    }
}

impl Parse for NodeText {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let value = input.parse::<ExprLit>()?.into();

        Ok(NodeText { value })
    }
}

impl Parse for NodeName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
         if input.peek2(PathSep) {
            Parser::node_name_punctuated_ident::<PathSep, fn(_) -> PathSep, PathSegment>(
                input, PathSep,
            )
            .map(|segments| {
                NodeName::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments,
                    },
                })
            })
        } else if input.peek2(Colon) || input.peek2(Dash) {
            Parser::node_name_punctuated_ident_with_alternate::<Punct, fn(_) -> Colon, fn(_) -> Dash, Ident>(
                input, Colon, Dash,
            )
            .map(NodeName::Punctuated)
        } else if input.peek(Brace) {
            let fork = &input.fork();
            let value = block_expr(fork)?;
            input.advance_to(fork);
            Ok(NodeName::Block(value.into()))
        } else if input.peek(Ident::peek_any) {
            let mut segments = Punctuated::new();
            let ident = Ident::parse_any(input)?;
            segments.push_value(PathSegment::from(ident));
            Ok(NodeName::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }))
        } else {
            Err(input.error("invalid tag name or attribute key"))
        }
    }
}

impl Parse for NodeBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(NodeBlock{
            value: block_expr(input)?.into()
        })
    }
}

impl Parse for KeyedAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key = NodeName::parse(input)?;
        let eq = input.parse::<Option<Token![=]>>()?;
        let value = if eq.is_some() {
            if input.is_empty() {
                return Err(Error::new(key.span(), "missing attribute value"));
            }

            if input.peek(Brace) {
                Some(NodeBlock::parse(input)?.into())
            } else {
                Some(NodeValueExpr::new(input.parse()?))
            }
        } else {
            None
        };

        let span = if let Some(ref val) = value {
            key.span().join(val.span()).unwrap_or(key.span())
        } else {
            key.span()
        };
        Ok(KeyedAttribute { key, value, span })
    }
}


impl Parse for NodeFragment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tag_open = FragmentOpen::parse(input)?;

        let (children, tag_close) = parse_tokens_until::<Node, _, _> (input, FragmentClose::parse)?;
        Ok(NodeFragment{
            tag_open,
            children,
            tag_close
        })
    }

}

impl Parse for NodeDoctype {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let token_start = DocStart::parse(input)?;
        let doctype_keyword = input.parse::<Ident>()?;
        if doctype_keyword.to_string().to_lowercase() != "doctype" {
            return Err(input.error("expected Doctype"));
        }
        let doctype = input.parse::<Ident>()?;
        let token_end = input.parse::<Token![>]>()?;

        let mut segments = Punctuated::new();
        segments.push_value(PathSegment::from(doctype));
        let value = NodeValueExpr::new(
            ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }
            .into(),
        );
        Ok(Self {
            token_start,
            token_doctype: doctype_keyword,
            value,
            token_end
        })
    }
}



impl Parse for NodeComment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let token_start = ComStart::parse(input)?;
        let value = NodeValueExpr::new(input.parse::<ExprLit>()?.into());
        let token_end = ComEnd::parse(input)?;
        
        Ok(NodeComment { 
            token_start,
            value,
            token_end,
        })
    }
}

impl Parse for OpenTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let token_lt = input.parse::<Token![<]>()?;
        let name = NodeName::parse(input)?;
        let (attributes, end_tag) = parse_tokens_with_separator::<NodeAttribute, _, _>(input, token::OpenTagEnd::parse)?;
        Ok(OpenTag {
            token_lt,
            name,
            attributes,
            end_tag,
        })
    }
}


impl Parse for NodeElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let open_tag = OpenTag::parse(input)?;
        let (children, close_tag_token) = if !open_tag.is_self_closed() {
            let (children, close_tag_token) = parse_tokens_until::<Node, _, _> (input, token::CloseTagStart::parse)?;
            (children, Some(close_tag_token))
        } else {
            (vec![], None)
        };
        let close_tag = close_tag_token.map(|t|CloseTag::parse_with_start_tag(input, t)).transpose()?;
       
        if let Some(close_tag) = &close_tag {
            if close_tag.name != open_tag.name {
                return Err(Error::new(close_tag.span(), "close tag has no coresponding open tag"));
            }
        }
        Ok(NodeElement {
            open_tag,
            children,
            close_tag
        })
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![<]) {
            if input.peek2(Token![!]) {
                if input.peek3(Ident) {
                    Node::Doctype(NodeDoctype::parse(input)?)
                } else {
                    Node::Comment(NodeComment::parse(input)?)
                }
            } else if input.peek2(Token![>]) {
                Node::Fragment(NodeFragment::parse(input)?)
            } else {
                Node::Element(NodeElement::parse(input)?)
            }
        } else if input.peek(Brace) {
            Node::Block(NodeBlock::parse(input)?)
        } else {
            Node::Text(NodeText::parse(input)?)
        })

        // if self.config.flat_tree {
        //     let mut children = node
        //         .children_mut()
        //         .map(|children| children.drain(..))
        //         .into_iter()
        //         .flatten()
        //         .collect::<Vec<_>>();

        //     let mut nodes = vec![node];
        //     nodes.append(&mut children);
        //     Ok(nodes)
        // } else {
        //     Ok(vec![node])
        // }
    }
}



/// Parse array of toknes that is seperated by spaces(tabs, or new lines).
/// Stop parsing array when other branch could parse anything.
/// 
/// Example:
/// ```
/// let tokens = quote::quote!(few idents seperated by spaces and then minus sign - that will stop parsing);
/// let concat_idents_without_minus = |input: ParseStream| -> syn::Result<String> {
///     let (idents, _minus) = parse_tokens_until<syn::Ident, _,_ >(input, |i|
///         i.parse::<Token![-]()
///     );
///     let mut new_str = String::new();
///     for ident in idents {
///         new_str.push_str(&ident.to_string()) 
///     }
///     Ok(new_str)
/// }
/// let concated = syn::parse_macro_input!(tokens in concat_idents);
/// assert_eq!(concated, "fewidentsseperatedbyspacesandthenminussign")
/// 
/// ```
/// 
/// 
fn parse_tokens_until<T, F, U>(input: ParseStream, stop: F) -> syn::Result<(Vec<T>, U)>
where T:Parse + std::fmt::Debug,
F: Fn(ParseStream) -> syn::Result<U>
{
    let mut collection = vec![];
    let res = loop {
        let fork = input.fork();
        if let Ok(res) = stop(&fork) {
            input.advance_to(&fork);
            break res;
        }
        let v = input.parse::<T>();
        collection.push(v?)
    };
    Ok((collection, res))
}

/// Two-phase parsing, firstly find separator, and then parse array of tokens before separator.
/// For simple inputs method work like `parse_tokens_until`,
/// but it creates intermediate TokenStream and copy of all tokens until separator token is found.
/// It is usefull when separator (or it's part) can be treated as part of token T.
///
/// 
/// Example:
/// ```
/// let tokens = quote!(some_expr_seperated + with - lt_gt * tokens <> other part);
/// ```
/// In this example "<" can can be parsed as part of expression, but we want to split tokens after "<>" was found.
/// So instead of parsing all input as expression, firstly we need to seperate it into two chunks.
///
fn parse_tokens_with_separator<T, F, U>(input: ParseStream, separator: F) -> syn::Result<(Vec<T>, U)>
where T:Parse + std::fmt::Debug,
F: Fn(ParseStream) -> syn::Result<U>
{
    let mut tokens = TokenStream::new();
    let res = loop {
        // we still use fork there, to allow parsing expressions in attributes, like foo=x/y
        let fork = input.fork();
        if let Ok(end) = separator(&fork) {
            
            input.advance_to(&fork);
            break end;
        }

        if input.is_empty() {
            return Err(input.error("expected closing caret >")); // TODO: fix text
        }

        let next: TokenTree = input.parse()?;
        tokens.extend([next]);
    };

    let parser = |input: ParseStream|{
        let mut collection = vec![];
        while !input.is_empty() {

            collection.push(input.parse::<T>()?);
        }

        Ok(collection)
    };
    let collection = parser.parse2(tokens)?;
    Ok((collection, res))
}


// This method could be const generic until https://github.com/rust-lang/rust/issues/63569
/// Parse array of tokens with 
pub(super) fn parse_array_of2_tokens<T:Parse>(input: ParseStream) -> syn::Result<[T; 2]> {
    Ok([input.parse()?, input.parse()?])
}


pub(super) fn to_tokens_array<I>(input: &mut TokenStream, iter: I)
where
    I: IntoIterator,
    I::Item: ToTokens
{
        use quote::TokenStreamExt;
        input.append_all(iter)
}

fn block_expr(input: syn::parse::ParseStream) -> syn::Result<ExprBlock> {
    let content;
    let brace_token = braced!(content in input);
    Ok(ExprBlock {
        attrs: vec![],
        label: None,
        block: Block {
            brace_token,
            stmts: Block::parse_within(&content)?,
        },
    })
}
