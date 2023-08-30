#![allow(non_upper_case_globals, clippy::into_iter_on_ref)]

mod ast_helpers;
mod kind;
mod path;
mod scope_analysis;
#[cfg(test)]
mod tests;

pub use scope_analysis::*;
