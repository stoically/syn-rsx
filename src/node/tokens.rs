//!
//! Implementation of ToTokens and Spanned for node related structs

use proc_macro2::{TokenStream, Punct, TokenTree};
use quote::{quote_spanned, ToTokens};
use syn::{parse::{Parse, discouraged::Speculative, ParseStream, Parser as _}, Ident, ExprBlock, braced, Block, Token, Error, token::{Brace, PathSep, Colon}, spanned::Spanned, ExprPath, Path, PathSegment, punctuated::Punctuated, ext::IdentExt, ExprLit};

use crate::{NodeValueExpr, NodeElement, NodeAttribute, Parser, punctuation::Dash};

use super::{Node, NodeBlock, NodeComment, NodeDoctype, NodeFragment, NodeName, NodeText, attribute::{KeyedAttribute, DynAttribute}, atoms::{FragmentOpen, FragmentClose, token::{self, DocStart, ComStart, ComEnd}, OpenTag, CloseTag}};

impl ToTokens for NodeValueExpr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let obj = self.as_ref();
        obj.to_tokens(tokens)
    }
}

impl ToTokens for NodeElement {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.open_tag.to_tokens(tokens);
        for child in &self.children {
            child.to_tokens(tokens)
        }
        if let Some(close_tag) = &self.close_tag{
            close_tag.to_tokens(tokens)
        }
    }
}

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

impl ToTokens for DynAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.block.to_tokens(tokens)
    }
}

impl ToTokens for NodeAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NodeAttribute::Attribute(a) => a.to_tokens(tokens),
            NodeAttribute::Block(b) => b.to_tokens(tokens),
        }
       
    }
}

impl ToTokens for NodeBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens)
    }
}

impl ToTokens for NodeComment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_start.to_tokens(tokens);
        self.value.to_tokens(tokens);
        self.token_end.to_tokens(tokens);
        
    }
}

impl ToTokens for token::DocStart {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_lt.to_tokens(tokens);
        self.token_not.to_tokens(tokens);
    }
}

impl ToTokens for NodeDoctype {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_start.to_tokens(tokens);
        self.token_doctype.to_tokens(tokens);
        self.value.to_tokens(tokens);
        self.token_end.to_tokens(tokens);
    }
}

impl ToTokens for FragmentOpen {

    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_lt.to_tokens(tokens);
        self.token_gt.to_tokens(tokens)
    }
}

impl ToTokens for FragmentClose {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_lt.to_tokens(tokens);
        self.token_sol.to_tokens(tokens);
        self.token_gt.to_tokens(tokens);
    }
}

impl ToTokens for NodeFragment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tag_open.to_tokens(tokens);
        for children in &self.children {
            children.to_tokens(tokens)
        }
        self.tag_close.to_tokens(tokens);
    }
}

impl ToTokens for NodeText {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens);
    }
}

impl ToTokens for Node {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Node::Block(b) => b.to_tokens(tokens),
            Node::Comment(c) => c.to_tokens(tokens),
            Node::Doctype(d) => d.to_tokens(tokens),
            Node::Fragment(f) => f.to_tokens(tokens),
            Node::Element(e) => e.to_tokens(tokens),
            Node::Text(t) => t.to_tokens(tokens),
        }
    }
}

impl ToTokens for NodeName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NodeName::Path(name) => name.to_tokens(tokens),
            NodeName::Punctuated(name) => name.to_tokens(tokens),
            NodeName::Block(name) => name.to_tokens(tokens),
        }
    }
}

impl ToTokens for OpenTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_lt.to_tokens(tokens);
        for attribute in &self.attributes {
            attribute.to_tokens(tokens);
        }
        self.end_tag.to_tokens(tokens);
    }
}

impl ToTokens for token::OpenTagEnd {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_solidus.to_tokens(tokens);
        self.token_gt.to_tokens(tokens);
    }
}

impl ToTokens for CloseTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_lt.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.token_solidus.to_tokens(tokens);
        self.token_gt.to_tokens(tokens);
    }
}

impl ToTokens for token::ComStart {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_lt.to_tokens(tokens);
        self.token_not.to_tokens(tokens);
        self.token_minus[0].to_tokens(tokens);
        self.token_minus[1].to_tokens(tokens);
    }
}

impl ToTokens for token::ComEnd {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.token_minus[0].to_tokens(tokens);
        self.token_minus[1].to_tokens(tokens);
        self.token_gt.to_tokens(tokens);
    }
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

impl Parse for DynAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let value = NodeBlock::parse(input)?.into();

        Ok(DynAttribute {
            block: NodeBlock { value },
        })
    }
}

impl Parse for NodeAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Brace) {
            Ok(NodeAttribute::Block(DynAttribute::parse(input)?))
        } else {
            Ok(NodeAttribute::Attribute(KeyedAttribute::parse(input)?))
        }
    }
}


impl Parse for FragmentOpen {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(FragmentOpen {
            token_lt: input.parse::<Token![<]>()?,
            token_gt: input.parse::<Token![>]>()?,
        })
    }
}

impl Parse for FragmentClose {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(FragmentClose {
            token_lt: input.parse::<Token![<]>()?,
            token_sol: input.parse::<Token![/]>()?,
            token_gt: input.parse::<Token![>]>()?,
        })
    }
}

impl Parse for token::DocStart {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(token::DocStart {
            token_lt: input.parse::<Token![<]>()?,
            token_not: input.parse::<Token![!]>()?,
        })
    }
}

impl Parse for token::ComStart {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(token::ComStart {
            token_lt: input.parse::<Token![<]>()?,
            token_not: input.parse::<Token![!]>()?,
            token_minus: [input.parse::<Token![-]>()?, input.parse::<Token![-]>()?],
        })
    }
}

impl Parse for token::ComEnd {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(token::ComEnd {
            token_minus: [input.parse::<Token![-]>()?, input.parse::<Token![-]>()?],
            token_gt: input.parse::<Token![>]>()?,
        })
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

impl Parse for token::OpenTagEnd {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let token_solidus = input.parse::<Option<Token![/]>>()?;
        let token_gt = input.parse::<Token![>]>()?;
        Ok(Self {
            token_solidus,
            token_gt
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

impl Parse for CloseTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let token_lt = input.parse::<Token![<]>()?;
        let token_solidus = input.parse::<Token![/]>()?;
        let name = NodeName::parse(input)?;
        let token_gt = input.parse::<Token![>]>()?;
        Ok(CloseTag {
            token_lt,
            token_solidus,
            name,
            token_gt,
        })
    }
}

impl Parse for NodeElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let open_tag = OpenTag::parse(input)?;
        let (children, close_tag) = if !open_tag.is_self_closed() {
            let (children, close_tag) = parse_tokens_until::<Node, _, _> (input, CloseTag::parse)?;
            (children, Some(close_tag))
        } else {
            (vec![], None)
        };
       
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
///     let (idents, _minus) = parse_vec_until<syn::Ident, _,_ >(input, |i|
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
