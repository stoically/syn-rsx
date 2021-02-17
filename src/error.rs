use proc_macro2::Span;
use thiserror::Error;

use crate::NodeType;

#[derive(Debug, Error)]
pub enum Error {
    #[error("top level nodes need to be of type {expected:?}, found {found:?}")]
    InvalidTopLevelNode {
        expected: NodeType,
        found: NodeType,
        span: Span,
    },

    #[error("saw {found:?} top level nodes but exactly {expected:?} are required")]
    InvalidNumberOfTopLevelNodes {
        expected: usize,
        found: usize,
        span: Span,
    },
}

impl Error {
    pub fn to_compile_error(&self) -> Error {}
}
