use std::marker::PhantomData;

use id_arena::{Arena, Id};

use crate::ScopeAnalyzer;

use super::variable::{Variable, _Variable};

pub enum _Scope<'a> {
    Base(ScopeBase<'a>),
}

impl<'a> _Scope<'a> {
    pub fn new_root(arena: &mut Arena<Self>) -> Id<Self> {
        arena.alloc(Self::Base(ScopeBase {
            kind: ScopeKind::Root,
            variables: Default::default(),
            _phantom_data: Default::default(),
        }))
    }

    fn base(&self) -> &ScopeBase<'a> {
        match self {
            _Scope::Base(value) => value,
        }
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
    _phantom_data: PhantomData<&'a ()>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Root,
}
