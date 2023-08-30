use id_arena::{Id, Arena};
use tree_sitter_lint::tree_sitter::Node;

use crate::ScopeAnalyzer;

use super::{scope::{_Scope, Scope}, variable::{_Variable, Variable}};

#[derive(Debug)]
pub struct _Reference<'a> {
    pub node: Node<'a>,
    pub scope: Id<_Scope<'a>>,
    pub resolved: Option<Id<_Variable<'a>>>,
    pub id: Id<Self>,
    pub usage_kind: UsageKind,
}

impl<'a> _Reference<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        arena: &mut Arena<Self>,
        usage_kind: UsageKind,
        node: Node<'a>,
        scope: Id<_Scope<'a>>,
    ) -> Id<Self> {
        arena.alloc_with_id(|id| Self {
            node,
            usage_kind,
            scope,
            resolved: Default::default(),
            id,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UsageKind {
    TypeReference,
    IdentifierReference,
}

#[derive(Debug)]
pub struct Reference<'a, 'b> {
    reference: &'b _Reference<'a>,
    scope_analyzer: &'b ScopeAnalyzer<'a>,
}

impl<'a, 'b> Reference<'a, 'b> {
    pub fn new(reference: &'b _Reference<'a>, scope_analyzer: &'b ScopeAnalyzer<'a>) -> Self {
        Self {
            reference,
            scope_analyzer,
        }
    }

    pub fn resolved(&self) -> Option<Variable<'a, 'b>> {
        self.reference
            .resolved
            .map(|resolved| self.scope_analyzer.borrow_variable(resolved))
    }

    pub fn node(&self) -> Node<'a> {
        self.reference.node
    }

    pub fn scope(&self) -> Scope<'a, 'b> {
        self.scope_analyzer.borrow_scope(self.reference.scope)
    }

    pub fn usage_kind(&self) -> UsageKind {
        self.reference.usage_kind
    }
}
