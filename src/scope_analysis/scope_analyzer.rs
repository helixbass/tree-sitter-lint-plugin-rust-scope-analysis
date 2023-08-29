use std::fmt;

use id_arena::Id;
use tracing::trace;
use tree_sitter_lint::{tree_sitter::Node, tree_sitter_grep::RopeOrSlice};

use crate::kind::SourceFile;

use super::{
    arenas::AllArenas,
    scope::{Scope, _Scope}, variable::{Variable, _Variable}, reference::{Reference, _Reference}, definition::{_Definition, Definition},
};

pub struct ScopeAnalyzer<'a> {
    file_contents: RopeOrSlice<'a>,
    pub scopes: Vec<Id<_Scope<'a>>>,
    arena: AllArenas<'a>,
}

impl<'a> ScopeAnalyzer<'a> {
    pub fn new(file_contents: impl Into<RopeOrSlice<'a>>) -> Self {
        let file_contents = file_contents.into();

        Self {
            file_contents,
            scopes: Default::default(),
            arena: Default::default(),
        }
    }

    pub fn visit(&mut self, node: Node<'a>) {
        trace!(?node, "visiting node");

        match node.kind() {
            SourceFile => {
                self.scopes.push(_Scope::new_root(&mut self.arena.scopes));
            }
            _ => self.visit_children(node),
        }
    }

    fn visit_children(&mut self, node: Node<'a>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.visit(child);
        }
    }

    pub(crate) fn borrow_scope<'b>(&'b self, scope: Id<_Scope<'a>>) -> Scope<'a, 'b> {
        Scope::new(&self.arena.scopes[scope], self)
    }

    pub(crate) fn borrow_variable<'b>(&'b self, variable: Id<_Variable<'a>>) -> Variable<'a, 'b> {
        Variable::new(&self.arena.variables[variable], self)
    }

    pub(crate) fn borrow_reference<'b>(&'b self, reference: Id<_Reference<'a>>) -> Reference<'a, 'b> {
        Reference::new(&self.arena.references[reference], self)
    }

    pub(crate) fn borrow_definition<'b>(&'b self, definition: Id<_Definition<'a>>) -> Definition<'a, 'b> {
        Definition::new(&self.arena.definitions[definition], self)
    }

    pub fn root_scope<'b>(&'b self) -> Scope<'a, 'b> {
        self.borrow_scope(self.scopes[0])
    }
}

impl<'a> fmt::Debug for ScopeAnalyzer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeAnalyzer")
            .field("file_contents", &self.file_contents)
            .field("scopes", &self.scopes)
            // .field("arena", &self.arena)
            .finish()
    }
}
