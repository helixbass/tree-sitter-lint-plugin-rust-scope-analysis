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
    ast_helpers::is_underscore,
    kind::{
        ConstItem, EnumItem, ExternCrateDeclaration, FunctionItem, Identifier, ModItem,
        ScopedIdentifier, ScopedUseList, SourceFile, StaticItem, StructItem, TraitItem, TypeItem,
        UnionItem, UseAsClause, UseDeclaration, UseList, UseWildcard, VisibilityModifier, MacroDefinition,
    },
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
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Struct, visibility, node.field("name"), node);

                self.visit_children(node);
            }
            ModItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Module, visibility, node.field("name"), node);

                self.visit_children(node);
            }
            FunctionItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(
                    DefinitionKind::Function,
                    visibility,
                    node.field("name"),
                    node,
                );

                self.visit_children(node);
            }
            ExternCrateDeclaration => {
                let visibility = Visibility::from_item(node, self);
                self.define(
                    DefinitionKind::ExternCrateDeclaration,
                    visibility,
                    node.child_by_field_name("alias")
                        .unwrap_or_else(|| node.field("name")),
                    node,
                );

                self.visit_children(node);
            }
            UseDeclaration => {
                self.visit_use_declaration(node);
            }
            TypeItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(
                    DefinitionKind::TypeAlias,
                    visibility,
                    node.field("name"),
                    node,
                );

                self.visit_children(node);
            }
            UnionItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Union, visibility, node.field("name"), node);

                self.visit_children(node);
            }
            EnumItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Enum, visibility, node.field("name"), node);

                self.visit_children(node);
            }
            ConstItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Const, visibility, node.field("name"), node);

                self.visit_children(node);
            }
            StaticItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Static, visibility, node.field("name"), node);

                self.visit_children(node);
            }
            TraitItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Trait, visibility, node.field("name"), node);

                self.visit_children(node);
            }
            MacroDefinition => {
                self.define(
                    DefinitionKind::Macro,
                    // TODO: should macros have their own "special" visibility?
                    Visibility::Pub,
                    node.field("name"),
                    node,
                );

                self.visit_children(node);
            }
            _ => self.visit_children(node),
        }
    }

    fn visit_use_declaration(&mut self, use_declaration: Node<'a>) {
        let visibility = Visibility::from_item(use_declaration, self);
        self.visit_use_clause(
            visibility,
            use_declaration,
            use_declaration.field("argument"),
        );
    }

    fn visit_use_clause(
        &mut self,
        visibility: Visibility<'a>,
        use_declaration: Node<'a>,
        use_clause: Node<'a>,
    ) {
        match use_clause.kind() {
            Identifier => {
                self.define(DefinitionKind::Use, visibility, use_clause, use_declaration);
            }
            ScopedIdentifier => {
                self.define(
                    DefinitionKind::Use,
                    visibility,
                    use_clause.field("name"),
                    use_declaration,
                );
            }
            UseAsClause => {
                self.define(
                    DefinitionKind::Use,
                    visibility,
                    use_clause.field("alias"),
                    use_declaration,
                );
            }
            UseList => {
                use_clause
                    .non_comment_named_children(SupportedLanguage::Rust)
                    .for_each(|use_clause| {
                        self.visit_use_clause(visibility.clone(), use_declaration, use_clause);
                    });
            }
            ScopedUseList => {
                use_clause
                    .field("list")
                    .non_comment_named_children(SupportedLanguage::Rust)
                    .for_each(|use_clause| {
                        self.visit_use_clause(visibility.clone(), use_declaration, use_clause);
                    });
            }
            UseWildcard => (),
            _ => unreachable!(),
        }
    }

    fn visit_children(&mut self, node: Node<'a>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.visit(child);
        }
    }

    fn define(
        &mut self,
        kind: DefinitionKind,
        visibility: Visibility<'a>,
        name: Node<'a>,
        node: Node<'a>,
    ) {
        if is_underscore(name, self) {
            return;
        }
        _Scope::define(
            self.current_scope_id(),
            &mut self.arena.scopes,
            &mut self.arena.definitions,
            &mut self.arena.variables,
            kind,
            visibility,
            name,
            node,
            &self.file_contents,
        );
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
