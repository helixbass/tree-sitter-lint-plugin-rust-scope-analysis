use std::marker::PhantomData;

use id_arena::{Id, Arena};

use super::definition::_Definition;

pub enum _Scope<'a> {
    Base(ScopeBase<'a>),
}

impl<'a> _Scope<'a> {
    pub fn new_root(
        arena: &mut Arena<Self>,
    ) -> Id<Self> {
        arena.alloc(Self::Base(ScopeBase {
            kind: ScopeKind::Root,
            definitions: Default::default(),
            _phantom_data: Default::default(),
        }))
    }
}

pub struct ScopeBase<'a> {
    kind: ScopeKind,
    definitions: Vec<Id<_Definition<'a>>>,
    _phantom_data: PhantomData<&'a ()>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Root,
}
