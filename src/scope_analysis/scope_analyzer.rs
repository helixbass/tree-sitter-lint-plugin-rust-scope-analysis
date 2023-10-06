use std::{borrow::Cow, fmt, ops};

use id_arena::Id;
use tracing::trace;
use tree_sitter_lint::{
    better_any::tid,
    tree_sitter::Node,
    tree_sitter_grep::{RopeOrSlice, SupportedLanguage},
    FileRunContext, FromFileRunContext, NodeExt, SourceTextProvider,
};

use crate::{
    ast_helpers::{
        get_leading_name_node_of_scoped_identifier, is_enum_variant_name, is_simple_identifier,
        is_underscore, is_attribute_name, is_macro_name,
    },
    kind::{
        ConstItem, EnumItem, ExternCrateDeclaration, FunctionItem, Identifier, LetDeclaration,
        MacroDefinition, ModItem, ScopedIdentifier, ScopedTypeIdentifier, ScopedUseList,
        SourceFile, StaticItem, StructItem, TraitItem, TypeIdentifier, TypeItem, UnionItem,
        UseAsClause, UseDeclaration, UseList, UseWildcard, TypeArguments, Kind, Self_, ArrowSeparatedPair,
    },
    scope_analysis::definition::{DefinitionKind, Visibility},
};

use super::{
    arenas::AllArenas,
    definition::{Definition, _Definition},
    reference::{Reference, UsageKind, _Reference},
    scope::{Scope, _Scope},
    variable::{Variable, _Variable},
};

macro_rules! current_scope_mut {
    ($self:ident) => {
        &mut $self.arena.scopes[$self.current_scope.unwrap()]
    };
}

pub struct ScopeAnalyzer<'a> {
    file_contents: RopeOrSlice<'a>,
    pub scopes: Vec<Id<_Scope<'a>>>,
    arena: AllArenas<'a>,
    current_scope: Option<Id<_Scope<'a>>>,
    is_in_left_hand_side_of: Option<Node<'a>>,
    visiting_descendants_of_kind: Option<Kind>,
}

impl<'a> ScopeAnalyzer<'a> {
    pub fn new(file_contents: impl Into<RopeOrSlice<'a>>) -> Self {
        let file_contents = file_contents.into();

        Self {
            file_contents,
            scopes: Default::default(),
            arena: Default::default(),
            current_scope: Default::default(),
            is_in_left_hand_side_of: Default::default(),
            visiting_descendants_of_kind: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn current_scope_mut(&mut self) -> &mut _Scope<'a> {
        &mut self.arena.scopes[self.current_scope.unwrap()]
    }

    pub fn visit(&mut self, node: Node<'a>) {
        trace!(?node, "visiting node");

        let mut saved_visiting_descendants_of_kind: Option<Kind> = None;
        if let Some(visiting_descendants_of_kind) = self.visiting_descendants_of_kind {
            if node.kind() == visiting_descendants_of_kind {
                trace!(kind = ?visiting_descendants_of_kind, "descending into descendant of kind");

                saved_visiting_descendants_of_kind = Some(visiting_descendants_of_kind);
                self.visiting_descendants_of_kind = None;
            } else {
                trace!(kind = ?visiting_descendants_of_kind, "just visiting children because not of kind");

                self.visit_children(node);
                return;
            }
        }

        match node.kind() {
            SourceFile => {
                let scope = _Scope::new_root(&mut self.arena.scopes, node);
                self.enter_scope(scope);

                self.visit_children(node);

                self.close(scope);
            }
            StructItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Struct, visibility, node.field("name"), node);

                self.visit_children_except_name(node);
            }
            ModItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Module, visibility, node.field("name"), node);

                self.visit_children_except_name(node);
            }
            FunctionItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(
                    DefinitionKind::Function,
                    visibility,
                    node.field("name"),
                    node,
                );

                let scope =
                    _Scope::new_function(&mut self.arena.scopes, node, self.current_scope.unwrap());
                self.enter_scope(scope);

                self.visit_children_except_name(node);

                self.close(scope);
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

                self.visit_children_except_name(node);
            }
            UnionItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Union, visibility, node.field("name"), node);

                self.visit_children_except_name(node);
            }
            EnumItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Enum, visibility, node.field("name"), node);

                self.visit_children_except_name(node);
            }
            ConstItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Const, visibility, node.field("name"), node);

                self.visit_children_except_name(node);
            }
            StaticItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Static, visibility, node.field("name"), node);

                self.visit_children_except_name(node);
            }
            TraitItem => {
                let visibility = Visibility::from_item(node, self);
                self.define(DefinitionKind::Trait, visibility, node.field("name"), node);

                self.visit_children_except_name(node);
            }
            MacroDefinition => {
                self.define(
                    DefinitionKind::Macro,
                    // TODO: should macros have their own "special" visibility?
                    Visibility::Pub,
                    node.field("name"),
                    node,
                );

                self.visit_children_except_name(node);
            }
            ScopedTypeIdentifier => {
                if let Some(leading_name_node) = get_leading_name_node_of_scoped_identifier(node) {
                    self.add_reference(get_usage_kind(node), leading_name_node);
                }
            }
            TypeIdentifier => {
                self.add_reference(get_usage_kind(node), node);
            }
            LetDeclaration => {
                self.is_in_left_hand_side_of = Some(node);
                self.visit(node.field("pattern"));
                self.is_in_left_hand_side_of = None;

                self.visit_children_except_field(node, "pattern");
            }
            ScopedIdentifier => {
                if let Some(leading_name_node) = get_leading_name_node_of_scoped_identifier(node) {
                    self.add_reference(get_usage_kind(node), leading_name_node);
                }

                self.visit_descendants_of_kind(node, TypeArguments);
            }
            Identifier => {
                if is_enum_variant_name(node) {
                    return;
                }

                #[allow(clippy::collapsible_else_if)]
                if let Some(is_in_left_hand_side_of) = self.is_in_left_hand_side_of {
                    self.define(
                        DefinitionKind::Variable,
                        Visibility::Private,
                        node,
                        is_in_left_hand_side_of,
                    );
                } else {
                    if is_simple_identifier(node) {
                        self.add_reference(get_usage_kind(node), node);
                    }
                }
            }
            ArrowSeparatedPair => {
                self.visit_children_except_field(node, "key");
            }
            _ => self.visit_children(node),
        }

        if let Some(saved_visiting_descendants_of_kind) = saved_visiting_descendants_of_kind {
            self.visiting_descendants_of_kind = Some(saved_visiting_descendants_of_kind);
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
            Self_ => {
                let parent = use_clause.parent().unwrap();
                if parent.kind() != UseList {
                    return;
                }
                let parent_parent = parent.parent().unwrap();
                if parent_parent.kind() != ScopedUseList {
                    return;
                }
                let Some(preceding_path) = parent_parent.child_by_field_name("path") else {
                    return;
                };

                let preceding_identifier = match preceding_path.kind() {
                    Identifier => preceding_path,
                    ScopedIdentifier => preceding_path.field("name"),
                    _ => unreachable!(),
                };
                self.define(DefinitionKind::Use, visibility, preceding_identifier, use_declaration);
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
            x => unreachable!("{x}"),
        }
    }

    fn visit_children(&mut self, node: Node<'a>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            self.visit(child);
        }
    }

    fn visit_children_except_name(&mut self, node: Node<'a>) {
        self.visit_children_except_field(node, "name");
    }

    fn visit_children_except_field(&mut self, node: Node<'a>, field: &str) {
        node.non_comment_named_children_and_field_names(SupportedLanguage::Rust)
            .filter(|(_, field_name)| *field_name != Some(field))
            .for_each(|(child, _)| {
                self.visit(child);
            });
    }

    fn visit_descendants_of_kind(&mut self, node: Node<'a>, kind: Kind) {
        self.visiting_descendants_of_kind = Some(kind);
        self.visit_children(node);
        self.visiting_descendants_of_kind = None;
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
            self.current_scope.unwrap(),
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

    fn add_reference(&mut self, usage_kind: UsageKind, node: Node<'a>) {
        current_scope_mut!(self).add_reference(
            &mut self.arena.references,
            usage_kind,
            node,
            &self.file_contents,
        );
    }

    fn close(&mut self, scope: Id<_Scope<'a>>) {
        loop {
            let current_scope = self.current_scope.unwrap();
            let upper = self.arena.scopes[current_scope].maybe_upper();

            trace!(scope = ?current_scope, "closing scope");

            _Scope::close(
                current_scope,
                &mut self.arena.scopes,
                &mut self.arena.references,
                &self.arena.definitions,
                &mut self.arena.variables,
                &self.file_contents,
            );
            self.current_scope = upper;
            if scope == current_scope {
                return;
            }
        }
    }

    fn enter_scope(&mut self, scope: Id<_Scope<'a>>) {
        trace!(?scope, kind = ?self.arena.scopes[scope].kind(), "entering scope");

        self.scopes.push(scope);
        self.current_scope = Some(scope);
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

    pub fn scopes<'b>(&'b self) -> impl Iterator<Item = Scope<'a, 'b>> {
        self.scopes.iter().map(|scope| self.borrow_scope(*scope))
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

impl<'a> FromFileRunContext<'a> for ScopeAnalyzer<'a> {
    fn from_file_run_context(file_run_context: FileRunContext<'a, '_>) -> Self {
        let mut scope_analyzer = Self::new(file_run_context.file_contents);

        scope_analyzer.visit(file_run_context.tree.root_node());

        scope_analyzer
    }
}

tid! { impl<'a> TidAble<'a> for ScopeAnalyzer<'a> }

impl<'a> fmt::Debug for ScopeAnalyzer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeAnalyzer")
            .field("file_contents", &self.file_contents)
            .field("scopes", &self.scopes)
            // .field("arena", &self.arena)
            .finish()
    }
}

fn get_usage_kind(node: Node) -> UsageKind {
    match node.kind() {
        TypeIdentifier | ScopedTypeIdentifier | Identifier | ScopedIdentifier => {
            if is_attribute_name(node) {
                UsageKind::AttributeName
            } else if is_macro_name(node) {
                UsageKind::Macro
            } else {
                UsageKind::IdentifierReference
            }
        }
        _ => unreachable!(),
    }
}
