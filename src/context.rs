//!
//! Context storage for changing parsing behaviour.
//!
//! Current syn::Parse implementation is working without context,
//! but sometimes you need to change parsing behaviour based on config,
//! or you could try to provide more information to user, by trying to parse
//! code that has invalid syntax.
use std::cell::RefCell;

use proc_macro2::TokenStream;

use crate::ParserConfig;

thread_local! {
    static ERRORS_STACK: RefCell<Vec<syn::Error>> = RefCell::new(Vec::new());
    static CONFIG: RefCell<Option<ParserConfig>> = RefCell::new(None);
}

pub fn push_error(error: syn::Error) {
    ERRORS_STACK.with(|errors| errors.borrow_mut().push(error))
}

pub fn get_first_error() -> Result<(), syn::Error> {
    if let Some(e) = ERRORS_STACK.with(|errors| errors.borrow().get(0).cloned()) {
        Err(e)
    } else {
        Ok(())
    }
}

pub fn try_emit_errors(token_stream: TokenStream) -> TokenStream {
    let errors = ERRORS_STACK.with(|errors| std::mem::take(&mut *errors.borrow_mut()));

    let errors = errors.into_iter().map(|e| e.into_compile_error());
    // println!("{:?}", error);
    let token_stream = quote::quote!( {#(#errors)* #token_stream });
    token_stream
}

pub fn with_config<F, U>(func: F) -> U
where
    F: FnOnce(&ParserConfig) -> U,
{
    CONFIG.with(move |cfg| {
        func(
            cfg.borrow()
                .as_ref()
                .expect("Config should be set before requesting it"),
        )
    })
}

pub struct Context {
    _v: (),
}

impl Context {
    pub fn new_from_config(config: ParserConfig) -> Self {
        if let Some(_) = CONFIG.with(|cfg| cfg.replace(Some(config))) {
            panic!("Config already set")
        }
        Context { _v: () }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        CONFIG
            .with(|old_dummy| old_dummy.replace(None))
            .expect("Config to be set");
    }
}
