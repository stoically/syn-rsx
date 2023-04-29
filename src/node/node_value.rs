//!
//!  Node value type
use std::{convert::TryFrom, ops::Deref};

use proc_macro2::TokenStream;
use syn::{token::Brace, Block, Expr, ExprBlock, ExprLit, Lit};

use super::path_to_string;
use crate::Error;

/// Smart pointer to `syn::Expr`.
#[derive(Clone, Debug, syn_derive::ToTokens)]
pub struct NodeValueExpr {
    expr: Expr,
}

impl NodeValueExpr {
    /// Create a `NodeValueExpr`.
    pub fn new(expr: Expr) -> Self {
        Self { expr }
    }
}

impl AsRef<Expr> for NodeValueExpr {
    fn as_ref(&self) -> &Expr {
        &self.expr
    }
}

impl Deref for NodeValueExpr {
    type Target = Expr;

    fn deref(&self) -> &Self::Target {
        &self.expr
    }
}

impl From<Expr> for NodeValueExpr {
    fn from(expr: Expr) -> Self {
        Self { expr }
    }
}

impl From<ExprLit> for NodeValueExpr {
    fn from(expr: ExprLit) -> Self {
        Self { expr: expr.into() }
    }
}

impl From<ExprBlock> for NodeValueExpr {
    fn from(expr: ExprBlock) -> Self {
        Self { expr: expr.into() }
    }
}

impl From<NodeValueExpr> for Expr {
    fn from(value: NodeValueExpr) -> Self {
        value.expr
    }
}

impl<'a> From<&'a NodeValueExpr> for &'a Expr {
    fn from(value: &'a NodeValueExpr) -> Self {
        &value.expr
    }
}

impl TryFrom<NodeValueExpr> for ExprBlock {
    type Error = Error;

    fn try_from(value: NodeValueExpr) -> Result<Self, Self::Error> {
        if let Expr::Block(block) = value.expr {
            Ok(block)
        } else {
            Err(Error::TryFrom(
                "NodeValueExpr does not match Expr::Block(_)".into(),
            ))
        }
    }
}

impl TryFrom<NodeValueExpr> for ExprLit {
    type Error = Error;

    fn try_from(value: NodeValueExpr) -> Result<Self, Self::Error> {
        if let Expr::Lit(lit) = value.expr {
            Ok(lit)
        } else {
            Err(Error::TryFrom(
                "NodeValueExpr does not match Expr::Lit(_)".into(),
            ))
        }
    }
}

impl TryFrom<&NodeValueExpr> for String {
    type Error = Error;

    fn try_from(value: &NodeValueExpr) -> Result<Self, Self::Error> {
        match &value.expr {
            Expr::Lit(expr) => match &expr.lit {
                Lit::Str(lit_str) => Some(lit_str.value()),
                _ => None,
            },
            Expr::Path(expr) => Some(path_to_string(&expr)),
            _ => None,
        }
        .ok_or_else(|| {
            Error::TryFrom(
                "NodeValueExpr does not match Expr::Lit(Lit::Str(_)) or Expr::Path(_)".into(),
            )
        })
    }
}

/// Block node.
///
/// Arbitrary rust code in braced `{}` blocks.
#[derive(Clone, Debug)]
pub enum NodeBlock {
    /// The block value..
    ValidBlock(Block),

    Invalid {
        brace: Brace,
        body: TokenStream,
    },
}

impl NodeBlock {
    ///
    /// Returns syntactically valid `syn::Block` of Rust code.
    ///
    /// Usually to make macro expansion IDE friendly, its better to use
    /// `ToTokens` instead. Because it also emit blocks that is invalid for
    /// syn, but valid for rust and rust analyzer. But if you need early
    /// checks that this block is valid  - use this method.
    ///
    /// Example of blocks that will or will not parse:
    /// ```no_compile
    /// {x.} // Rust will parse this syntax, but for syn this is invalid Block, because after dot ident is expected.
    ///      // Emiting code like this for rust analyzer allows it to find completion.
    ///      // This block is parsed as NodeBlock::Invalid
    /// {]}   // this is invalid syntax for rust compiler and rust analyzer so it will not be parsed at all.
    /// {x + y} // Valid syn Block, parsed as NodeBlock::Valid
    /// ```
    pub fn try_block(&self) -> Option<&Block> {
        match self {
            Self::ValidBlock(b) => Some(b),
            Self::Invalid { .. } => None,
        }
    }
}

impl TryFrom<NodeBlock> for Block {
    type Error = syn::Error;
    fn try_from(v: NodeBlock) -> Result<Block, Self::Error> {
        match v {
            NodeBlock::ValidBlock(v) => Ok(v),
            NodeBlock::Invalid { .. } => Err(syn::Error::new_spanned(
                v,
                "Cant parse expression as block.",
            )),
        }
    }
}

impl TryFrom<NodeBlock> for NodeValueExpr {
    type Error = syn::Error;
    fn try_from(v: NodeBlock) -> Result<NodeValueExpr, Self::Error> {
        match v {
            NodeBlock::ValidBlock(v) => {
                let expr = ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: v,
                };
                Ok(expr.into())
            }
            NodeBlock::Invalid { .. } => Err(syn::Error::new_spanned(
                v,
                "Cant parse expression as block.",
            )),
        }
    }
}
