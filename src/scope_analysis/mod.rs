mod arenas;
mod definition;
mod reference;
mod scope;
mod scope_analyzer;
mod variable;

pub use definition::{Definition, DefinitionKind};
pub use reference::{Reference, UsageKind};
pub use scope_analyzer::ScopeAnalyzer;
pub use scope::{Scope, ScopeKind};
