use std::{collections::HashSet, fmt::Debug, rc::Rc};

use proc_macro2_diagnostics::{Diagnostic, Level};
use syn::parse::{Parse, ParseStream};

use crate::config::TransformBlockFn;

#[derive(Default)]
pub struct RecoveryConfig {
    ///
    /// Try to parse invalid syn::Block as something.
    /// Usefull to make expressions more IDE-friendly.
    pub(crate) recover_block: bool,
    /// elements that has no child and is always self closed like <img> and <br>
    pub(crate) always_self_closed_elements: HashSet<&'static str>,
    /// Elements like <script> <style>, context of which is not a valid html,
    /// and should be provided as is.
    pub(crate) raw_text_elements: HashSet<&'static str>,
    pub(crate) transform_block: Option<Rc<TransformBlockFn>>,
}

impl Debug for RecoveryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveryConfig")
            .field("recover_block", &self.recover_block)
            .field(
                "always_self_closed_elements",
                &self.always_self_closed_elements,
            )
            .field("raw_text_elements", &self.raw_text_elements)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct RecoverableContext {
    pub(super) diagnostics: Vec<Diagnostic>,
    config: RecoveryConfig,
}
impl RecoverableContext {
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            diagnostics: vec![],
            config,
        }
    }
    pub fn parse_result<T>(self, val: Option<T>) -> ParsingResult<T> {
        ParsingResult::from_parts(val, self.diagnostics)
    }

    pub fn config(&self) -> &RecoveryConfig {
        &self.config
    }

    pub fn parse_simple<T: Parse>(&mut self, input: ParseStream) -> Option<T> {
        match input.parse() {
            Ok(v) => Some(v),
            Err(e) => {
                self.diagnostics.push(e.into());
                None
            }
        }
    }
    pub fn parse_recoverable<T: ParseRecoverable>(&mut self, input: ParseStream) -> Option<T> {
        T::parse_recoverable(self, input)
    }
    pub fn save_diagnostics<T>(&mut self, val: syn::Result<T>) -> Option<T> {
        match val {
            Ok(v) => Some(v),
            Err(e) => {
                self.diagnostics.push(e.into());
                None
            }
        }
    }

    pub fn push_diagnostic(&mut self, diagnostic: impl Into<Diagnostic>) {
        self.diagnostics.push(diagnostic.into());
    }
}

/// Result of parsing.
pub enum ParsingResult<T> {
    /// Fully valid ast that was parsed without errors.
    Ok(T),
    /// The ast contain invalid starting tokens, and cannot be parsed.
    Failed(Vec<Diagnostic>),
    /// The ast can be partially parsed,
    /// but some tokens was skipped during parsing, or their meaning was
    /// changed.
    Partial(T, Vec<Diagnostic>),
}

impl<T> ParsingResult<T> {
    /// Create new ParsingResult from optional value and accumulated errors.
    pub fn from_parts(value: Option<T>, errors: Vec<Diagnostic>) -> Self {
        if let Some(token) = value {
            if errors.is_empty() {
                Self::Ok(token)
            } else {
                Self::Partial(token, errors)
            }
        } else {
            Self::Failed(errors)
        }
    }

    ///
    /// Convert into syn::Result, with fail on first diagnostic message,
    /// Returns Error on ParsingResult::Failed, and ParsingResult::Partial.
    pub fn into_result(self) -> syn::Result<T> {
        match self {
            ParsingResult::Ok(r) => Ok(r),
            ParsingResult::Failed(errors) | ParsingResult::Partial(_, errors) => Err(errors
                .into_iter()
                .next()
                .unwrap_or_else(|| {
                    Diagnostic::new(
                        Level::Error,
                        "Object parsing failed, but no additional info was provided",
                    )
                })
                .into()),
        }
    }

    pub fn split(self) -> (Option<T>, Vec<Diagnostic>) {
        match self {
            Self::Ok(r) => (Some(r), vec![]),
            Self::Failed(errors) => (None, errors),
            Self::Partial(r, errors) => (Some(r), errors),
        }
    }
}

impl<T> ParsingResult<Vec<T>> {
    pub fn split_vec(self) -> (Vec<T>, Vec<Diagnostic>) {
        match self {
            Self::Ok(r) => (r, vec![]),
            Self::Failed(errors) => (vec![], errors),
            Self::Partial(r, errors) => (r, errors),
        }
    }
}

impl<T> From<syn::Result<T>> for ParsingResult<T> {
    ///
    /// Convert into syn::Result,
    /// Returns Error on ParsingResult::Failed, and ParsingResult::Partial.
    fn from(result: syn::Result<T>) -> ParsingResult<T> {
        match result {
            Result::Ok(r) => ParsingResult::Ok(r),
            Result::Err(e) => ParsingResult::Failed(vec![e.into()]),
        }
    }
}

///
/// Provide a `Parse` interface to `ParseRecoverable` types.
/// Returns error if any error was set in `RecoverableParser` during parsing.
/// Use Default implementation of RecoveryConfig.
pub struct Recoverable<T>(T);
impl<T> Recoverable<T> {
    pub fn inner(self) -> T {
        self.0
    }
}

impl<T: ParseRecoverable> Parse for Recoverable<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut empty_context = RecoverableContext::default();
        let parse = T::parse_recoverable(&mut empty_context, input);
        empty_context
            .parse_result(parse)
            .into_result()
            .map(Recoverable)
    }
}

///
/// Parsing interface for recoverable TokenStream parsing,
///     analog to syn::Parse but with ability to keep unexpected tokens, and
/// more diagnostic messages.
///
/// If input stream can be parsed to valid, or partially valid object
/// Option::Some should be returned. If object is parsed partially one can save
/// diagnostic message in `RecoverableParser`. If object is failed to parse
/// Option::None should be returned, and any message should be left in
/// `RecoverableParser`.
///
/// Instead of using content we can change result as follow:
/// ```no_build
///  pub trait ParseRecoverable: Sized
/// {
///     fn parse_recoverable(input: ParseStream) -> ParsingResult<Self>;
/// }
/// ```
/// And it would more type-safe, but because std::ops::Try is not stable,
/// writing implementation with this would end with a lot of boilerplate.
pub trait ParseRecoverable: Sized {
    fn parse_recoverable(parser: &mut RecoverableContext, input: ParseStream) -> Option<Self>;
}
