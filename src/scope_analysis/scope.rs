use id_arena::{Arena, Id};
use tree_sitter_lint::{tree_sitter::Node, SourceTextProvider, NodeExt};

use crate::ScopeAnalyzer;

use super::{variable::{Variable, _Variable}, definition::{Visibility, _Definition, DefinitionKind}};

pub enum _Scope<'a> {
    Base(ScopeBase<'a>),
}

impl<'a> _Scope<'a> {
    pub fn new_root(arena: &mut Arena<Self>) -> Id<Self> {
        arena.alloc_with_id(|id| Self::Base(ScopeBase {
            kind: ScopeKind::Root,
            variables: Default::default(),
            id,
        }))
    }

    fn base(&self) -> &ScopeBase<'a> {
        match self {
            _Scope::Base(value) => value,
        }
    }

    fn base_mut(&mut self) -> &mut ScopeBase<'a> {
        match self {
            _Scope::Base(value) => value,
        }
    }

    pub fn id(&self) -> Id<Self> {
        self.base().id
    }

    pub fn variables_mut(&mut self) -> &mut Vec<Id<_Variable<'a>>> {
        &mut self.base_mut().variables
    }

    #[allow(clippy::too_many_arguments)]
    pub fn define(
        self_: Id<Self>,
        arena: &mut Arena<Self>,
        definition_arena: &mut Arena<_Definition<'a>>,
        variable_arena: &mut Arena<_Variable<'a>>,
        kind: DefinitionKind,
        visibility: Visibility<'a>,
        name: Node<'a>,
        node: Node<'a>,
        source_text_provider: &impl SourceTextProvider<'a>,
    ) {
        let definition = _Definition::new(
            definition_arena,
            kind,
            name,
            node,
            visibility,
        );
        let variable = _Variable::new(
            variable_arena,
            name.text(source_text_provider),
            definition,
            arena[self_].id(),
        );
        arena[self_].variables_mut().push(variable);
    }
}

pub struct Scope<'a, 'b> {
    scope_analyzer: &'b ScopeAnalyzer<'a>,
    scope: &'b _Scope<'a>,
}

impl<'a, 'b> Scope<'a, 'b> {
    pub fn new(scope: &'b _Scope<'a>, scope_analyzer: &'b ScopeAnalyzer<'a>) -> Self {
        Self {
            scope_analyzer,
            scope,
        }
    }

    pub fn variables(&self) -> impl Iterator<Item = Variable<'a, 'b>> + '_ {
        self.scope
            .base()
            .variables
            .iter()
            .map(|variable| self.scope_analyzer.borrow_variable(*variable))
    }
}

pub struct ScopeBase<'a> {
    kind: ScopeKind,
    variables: Vec<Id<_Variable<'a>>>,
    id: Id<_Scope<'a>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Root,
}
