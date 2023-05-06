use std::{convert::TryFrom, fmt};

use proc_macro2::Punct;
use syn::{
    ext::IdentExt,
    parse::{discouraged::Speculative, Parse},
    punctuated::{Pair, Punctuated},
    token::{Brace, Colon, PathSep},
    Block, ExprPath, Ident, Path, PathSegment,
};

use super::path_to_string;
use crate::{punctuation::Dash, tokens::block_expr, Error, Parser};

/// Name of the node.
#[derive(Clone, Debug, syn_derive::ToTokens)]
pub enum NodeName {
    /// A plain identifier like `div` is a path of length 1, e.g. `<div />`. Can
    /// be separated by double colons, e.g. `<foo::bar />`.
    Path(ExprPath),

    /// Name separated by punctuation, e.g. `<div data-foo="bar" />` or `<div
    /// data:foo="bar" />`.
    Punctuated(Punctuated<Ident, Punct>),

    /// Arbitrary rust code in braced `{}` blocks.
    Block(Block),
}

impl TryFrom<&NodeName> for Block {
    type Error = Error;

    fn try_from(node: &NodeName) -> Result<Self, Self::Error> {
        match node {
            NodeName::Block(b) => Ok(b.to_owned()),
            _ => Err(Error::TryFrom(
                "NodeName does not match NodeName::Block(Expr::Block(_))".into(),
            )),
        }
    }
}

impl PartialEq for NodeName {
    fn eq(&self, other: &NodeName) -> bool {
        match self {
            Self::Path(this) => match other {
                Self::Path(other) => this == other,
                _ => false,
            },
            // can't be derived automatically because `Punct` doesn't impl `PartialEq`
            Self::Punctuated(this) => match other {
                Self::Punctuated(other) => {
                    this.pairs()
                        .zip(other.pairs())
                        .all(|(this, other)| match (this, other) {
                            (
                                Pair::Punctuated(this_ident, this_punct),
                                Pair::Punctuated(other_ident, other_punct),
                            ) => {
                                this_ident == other_ident
                                    && this_punct.as_char() == other_punct.as_char()
                            }
                            (Pair::End(this), Pair::End(other)) => this == other,
                            _ => false,
                        })
                }
                _ => false,
            },
            Self::Block(this) => match other {
                Self::Block(other) => this == other,
                _ => false,
            },
        }
    }
}

impl fmt::Display for NodeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NodeName::Path(expr) => path_to_string(expr),
                NodeName::Punctuated(name) => {
                    name.pairs()
                        .flat_map(|pair| match pair {
                            Pair::Punctuated(ident, punct) => {
                                [ident.to_string(), punct.to_string()]
                            }
                            Pair::End(ident) => [ident.to_string(), "".to_string()],
                        })
                        .collect::<String>()
                }
                NodeName::Block(_) => String::from("{}"),
            }
        )
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
            Parser::node_name_punctuated_ident_with_alternate::<
                Punct,
                fn(_) -> Colon,
                fn(_) -> Dash,
                Ident,
            >(input, Colon, Dash)
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
