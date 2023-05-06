//!
//! Context storage for changing parsing behaviour.
//!
//! Current syn::Parse implementation is working without context,
//! but sometimes you need to change parsing behaviour based on config,
//! or you could try to provide more information to user, by trying to parse
//! code that has invalid syntax.
use std::cell::RefCell;

use crate::ParserConfig;

thread_local! {
    static CONFIG: RefCell<Option<ParserConfig>> = RefCell::new(None);
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
