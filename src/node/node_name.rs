use std::{convert::TryFrom, fmt};

use proc_macro2::Punct;
use syn::{
    punctuated::{Pair, Punctuated},
    Expr, ExprBlock, ExprPath, Ident,
};

use super::path_to_string;
use crate::Error;

/// Name of the node.
#[derive(Debug, syn_derive::ToTokens)]
pub enum NodeName {
    /// A plain identifier like `div` is a path of length 1, e.g. `<div />`. Can
    /// be separated by double colons, e.g. `<foo::bar />`.
    Path(ExprPath),

    /// Name separated by punctuation, e.g. `<div data-foo="bar" />` or `<div
    /// data:foo="bar" />`.
    Punctuated(Punctuated<Ident, Punct>),

    /// Arbitrary rust code in braced `{}` blocks.
    Block(Expr),
}

impl TryFrom<&NodeName> for ExprBlock {
    type Error = Error;

    fn try_from(node: &NodeName) -> Result<Self, Self::Error> {
        match node {
            NodeName::Block(Expr::Block(expr)) => Ok(expr.to_owned()),
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
