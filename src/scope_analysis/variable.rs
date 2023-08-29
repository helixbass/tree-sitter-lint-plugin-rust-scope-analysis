use std::borrow::Cow;

use id_arena::{Id, Arena};

use crate::ScopeAnalyzer;

use super::{definition::{_Definition, Definition}, scope::{_Scope, Scope}, reference::{_Reference, Reference}};

pub struct _Variable<'a> {
    pub name: Cow<'a, str>,
    pub definition: Id<_Definition<'a>>,
    pub scope: Id<_Scope<'a>>,
    pub references: Vec<Id<_Reference<'a>>>,
    id: Id<Self>,
}

impl<'a> _Variable<'a> {
    pub fn new(
        arena: &mut Arena<Self>,
        name: Cow<'a, str>,
        definition: Id<_Definition<'a>>,
        scope: Id<_Scope<'a>>,
    ) -> Id<Self> {
        arena.alloc_with_id(|id| Self {
            name,
            definition,
            scope,
            id,
            references: Default::default(),
        })
    }
}

pub struct Variable<'a, 'b> {
    variable: &'b _Variable<'a>,
    scope_analyzer: &'b ScopeAnalyzer<'a>,
}

impl<'a, 'b> Variable<'a, 'b> {
    pub fn new(variable: &'b _Variable<'a>, scope_analyzer: &'b ScopeAnalyzer<'a>) -> Self {
        Self {
            variable,
            scope_analyzer,
        }
    }

    pub fn name(&self) -> &str {
        &self.variable.name
    }

    pub fn scope(&self) -> Scope<'a, 'b> {
        self.scope_analyzer.borrow_scope(self.variable.scope)
    }

    pub fn references(&self) -> impl Iterator<Item = Reference<'a, 'b>> + '_ {
        self.variable.references.iter().map(|&reference| self.scope_analyzer.borrow_reference(reference))
    }

    pub fn definition(&self) -> Definition<'a, 'b> {
        self.scope_analyzer.borrow_definition(self.variable.definition)
    }
}
