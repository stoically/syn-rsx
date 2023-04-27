//!
//!  Node value type
use std::{convert::TryFrom, ops::Deref};

use syn::{Expr, ExprBlock, ExprLit, Lit};

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
#[derive(Clone, Debug, syn_derive::ToTokens)]
pub struct NodeBlock {
    /// The block value..
    pub value: NodeValueExpr,
}

impl From<NodeBlock> for NodeValueExpr {
    fn from(v: NodeBlock) -> NodeValueExpr {
        v.value
    }
}
