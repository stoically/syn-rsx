//!
//! Tokens that is used as parts of nodes, to simplify parsing.
//! Example:
//! `<!--` `<>` `</>` `<!` `/>`
//!
//! Also contain some entities that split parsing into several small units,
//! like: `<open_tag attr />`
//! `</close_tag>`

use syn::Token;

use crate::{NodeAttribute, NodeName};

pub mod token {
    use syn::Token;

    /// Start part of doctype tag
    /// `<!`
    #[derive(Debug)]
    pub struct DocStart {
        pub token_lt: Token![<],
        pub token_not: Token![!],
    }

    /// Start part of comment tag
    /// `<!--`
    #[derive(Debug)]
    pub struct ComStart {
        pub token_lt: Token![<],
        pub token_not: Token![!],
        pub token_minus: [Token![-]; 2],
    }

    /// End part of comment tag
    /// `-->`
    #[derive(Debug)]
    pub struct ComEnd {
        pub token_minus: [Token![-]; 2],
        pub token_gt: Token![>],
    }

    /// End part of element's open tag
    /// `/>`
    #[derive(Debug)]
    pub struct OpenTagEnd {
        pub token_solidus: Option<Token![/]>,
        pub token_gt: Token![>],
    }
}

/// Fragment open part
/// `<>`
#[derive(Debug)]
pub struct FragmentOpen {
    pub token_lt: Token![<],
    pub token_gt: Token![>],
}

/// Fragment close part
/// `</>`
#[derive(Debug)]
pub struct FragmentClose {
    pub token_lt: Token![<],
    pub token_sol: Token![/],
    pub token_gt: Token![>],
}

/// Open tag for element, possibly self-closed.
/// <name attr=x, attr_flag>
#[derive(Debug)]
pub struct OpenTag {
    pub token_lt: Token![<],
    pub name: NodeName,
    pub attributes: Vec<NodeAttribute>,
    pub end_tag: token::OpenTagEnd,
}

impl OpenTag {
    pub fn is_self_closed(&self) -> bool {
        self.end_tag.token_solidus.is_some()
    }
}

/// Open tag for element, <name attr=x, attr_flag>
#[derive(Debug)]
pub struct CloseTag {
    pub token_lt: Token![<],
    pub token_solidus: Token![/],
    pub name: NodeName,
    pub token_gt: Token![>],
}
