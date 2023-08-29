#![allow(non_upper_case_globals)]

mod kind;
mod path;
mod scope_analysis;
#[cfg(test)]
mod tests;

pub use scope_analysis::ScopeAnalyzer;
