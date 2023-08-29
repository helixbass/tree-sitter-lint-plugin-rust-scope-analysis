use std::{borrow::Cow, fmt, ops};

use id_arena::Id;
use tracing::trace;
use tree_sitter_lint::{
    squalid::EverythingExt,
    tree_sitter::Node,
    tree_sitter_grep::{RopeOrSlice, SupportedLanguage},
    NodeExt, SourceTextProvider,
};

use crate::{
    kind::{SourceFile, StructItem, VisibilityModifier, ModItem, FunctionItem},
    scope_analysis::definition::{DefinitionKind, Visibility},
};

use super::{
    arenas::AllArenas,
    definition::{Definition, _Definition},
    reference::{Reference, _Reference},
    scope::{Scope, _Scope},
    variable::{Variable, _Variable},
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

    fn current_scope_id(&self) -> Id<_Scope<'a>> {
        *self.scopes.last().unwrap()
    }

    fn current_scope_mut(&mut self) -> &mut _Scope<'a> {
        &mut self.arena.scopes[*self.scopes.last().unwrap()]
    }

    pub fn visit(&mut self, node: Node<'a>) {
        trace!(?node, "visiting node");

        match node.kind() {
            SourceFile => {
                self.scopes.push(_Scope::new_root(&mut self.arena.scopes));

                self.visit_children(node);
            }
            StructItem => {
                let visibility = node
                    .first_non_comment_named_child(SupportedLanguage::Rust)
                    .when(|node| node.kind() == VisibilityModifier)
                    .map(|node| Visibility::from_visibility_modifier(node, self))
                    .unwrap_or_default();
                _Scope::define(
                    self.current_scope_id(),
                    &mut self.arena.scopes,
                    &mut self.arena.definitions,
                    &mut self.arena.variables,
                    DefinitionKind::Struct,
                    visibility,
                    node.field("name"),
                    node,
                    &self.file_contents,
                );

                self.visit_children(node);
            }
            ModItem => {
                let visibility = node
                    .first_non_comment_named_child(SupportedLanguage::Rust)
                    .when(|node| node.kind() == VisibilityModifier)
                    .map(|node| Visibility::from_visibility_modifier(node, self))
                    .unwrap_or_default();
                _Scope::define(
                    self.current_scope_id(),
                    &mut self.arena.scopes,
                    &mut self.arena.definitions,
                    &mut self.arena.variables,
                    DefinitionKind::Module,
                    visibility,
                    node.field("name"),
                    node,
                    &self.file_contents,
                );

                self.visit_children(node);
            }
            FunctionItem => {
                let visibility = node
                    .first_non_comment_named_child(SupportedLanguage::Rust)
                    .when(|node| node.kind() == VisibilityModifier)
                    .map(|node| Visibility::from_visibility_modifier(node, self))
                    .unwrap_or_default();
                _Scope::define(
                    self.current_scope_id(),
                    &mut self.arena.scopes,
                    &mut self.arena.definitions,
                    &mut self.arena.variables,
                    DefinitionKind::Function,
                    visibility,
                    node.field("name"),
                    node,
                    &self.file_contents,
                );

                self.visit_children(node);
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

    pub(crate) fn borrow_reference<'b>(
        &'b self,
        reference: Id<_Reference<'a>>,
    ) -> Reference<'a, 'b> {
        Reference::new(&self.arena.references[reference], self)
    }

    pub(crate) fn borrow_definition<'b>(
        &'b self,
        definition: Id<_Definition<'a>>,
    ) -> Definition<'a, 'b> {
        Definition::new(&self.arena.definitions[definition], self)
    }

    pub fn root_scope<'b>(&'b self) -> Scope<'a, 'b> {
        self.borrow_scope(self.scopes[0])
    }
}

impl<'a> SourceTextProvider<'a> for ScopeAnalyzer<'a> {
    fn node_text(&self, node: Node) -> Cow<'a, str> {
        self.file_contents.node_text(node)
    }

    fn slice(&self, range: ops::Range<usize>) -> Cow<'a, str> {
        self.file_contents.slice(range)
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
