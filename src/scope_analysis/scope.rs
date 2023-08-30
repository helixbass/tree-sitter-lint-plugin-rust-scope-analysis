use std::{borrow::Cow, collections::HashMap};

use id_arena::{Arena, Id};
use tracing::trace;
use tree_sitter_lint::{tree_sitter::Node, NodeExt, SourceTextProvider};

use crate::ScopeAnalyzer;

use super::{
    definition::{DefinitionKind, Visibility, _Definition},
    reference::{Reference, UsageKind, _Reference},
    variable::{Variable, _Variable},
};

type NameMap<'a> = HashMap<Cow<'a, str>, Vec<Id<_Variable<'a>>>>;

pub enum _Scope<'a> {
    Base(ScopeBase<'a>),
}

impl<'a> _Scope<'a> {
    fn _new(
        arena: &mut Arena<Self>,
        kind: ScopeKind,
        upper: Option<Id<Self>>,
        node: Node<'a>,
        create_from_base: impl Fn(ScopeBase<'a>) -> Self,
    ) -> Id<Self> {
        let id = arena.alloc_with_id(|id| {
            create_from_base(ScopeBase {
                kind,
                variables: Default::default(),
                references: Default::default(),
                pre_resolved: Some(Default::default()),
                upper,
                id,
                name_map: Default::default(),
                through: Default::default(),
                node,
                child_scopes: Default::default(),
            })
        });

        if let Some(upper) = upper {
            arena[upper].child_scopes_mut().push(id);
        }

        id
    }

    fn new_base(
        arena: &mut Arena<Self>,
        kind: ScopeKind,
        upper: Option<Id<Self>>,
        node: Node<'a>,
    ) -> Id<Self> {
        Self::_new(arena, kind, upper, node, Self::Base)
    }

    pub fn new_root(arena: &mut Arena<Self>, node: Node<'a>) -> Id<Self> {
        Self::new_base(arena, ScopeKind::Root, Default::default(), node)
    }

    pub fn new_function(arena: &mut Arena<Self>, node: Node<'a>, upper: Id<Self>) -> Id<Self> {
        Self::new_base(arena, ScopeKind::Function, Some(upper), node)
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

    pub fn references(&self) -> &[Id<_Reference<'a>>] {
        &self.base().references
    }

    pub fn references_mut(&mut self) -> &mut Vec<Id<_Reference<'a>>> {
        &mut self.base_mut().references
    }

    pub fn pre_resolved(&self) -> &Vec<Id<_Reference<'a>>> {
        self.base().pre_resolved.as_ref().unwrap()
    }

    pub fn pre_resolved_mut(&mut self) -> &mut Vec<Id<_Reference<'a>>> {
        self.base_mut().pre_resolved.as_mut().unwrap()
    }

    pub fn name_map(&self) -> &NameMap<'a> {
        &self.base().name_map
    }

    pub fn name_map_mut(&mut self) -> &mut NameMap<'a> {
        &mut self.base_mut().name_map
    }

    pub fn maybe_upper(&self) -> Option<Id<Self>> {
        self.base().upper
    }

    pub fn through_mut(&mut self) -> &mut Vec<Id<_Reference<'a>>> {
        &mut self.base_mut().through
    }

    pub fn child_scopes_mut(&mut self) -> &mut Vec<Id<Self>> {
        &mut self.base_mut().child_scopes
    }

    pub fn kind(&self) -> ScopeKind {
        self.base().kind
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
        let name_text = name.text(source_text_provider);

        let definition = _Definition::new(definition_arena, kind, name, node, visibility.clone());
        let variable = _Variable::new(
            variable_arena,
            name_text.clone(),
            definition,
            arena[self_].id(),
        );

        trace!(
            name = ?name_text,
            variable_id = ?variable_arena[variable].id,
            definition_id = ?definition_arena[definition].id,
            name_node = ?name,
            ?node,
            ?kind,
            ?visibility,
            "defining"
        );

        arena[self_].variables_mut().push(variable);
        arena[self_]
            .name_map_mut()
            .entry(name_text)
            .or_default()
            .push(variable);
    }

    pub fn add_reference(
        &mut self,
        reference_arena: &mut Arena<_Reference<'a>>,
        usage_kind: UsageKind,
        node: Node<'a>,
        source_text_provider: &impl SourceTextProvider<'a>,
    ) {
        let reference = _Reference::new(reference_arena, usage_kind, node, self.id());

        trace!(
            name = ?node.text(source_text_provider),
            id = ?reference_arena[reference].id,
            ?node,
            ?usage_kind,
            "adding reference"
        );

        self.references_mut().push(reference);
        self.pre_resolved_mut().push(reference);
    }

    pub fn close(
        self_: Id<Self>,
        arena: &mut Arena<Self>,
        reference_arena: &mut Arena<_Reference<'a>>,
        definition_arena: &Arena<_Definition<'a>>,
        variable_arena: &mut Arena<_Variable<'a>>,
        source_text_provider: &impl SourceTextProvider<'a>,
    ) {
        #[allow(clippy::unnecessary_to_owned)]
        for reference in arena[self_].pre_resolved().to_owned() {
            Self::close_reference(
                self_,
                arena,
                reference_arena,
                definition_arena,
                variable_arena,
                source_text_provider,
                reference,
            );
        }
        *arena[self_].pre_resolved_mut() = Default::default();
    }

    pub fn close_reference(
        self_: Id<Self>,
        arena: &mut Arena<Self>,
        reference_arena: &mut Arena<_Reference<'a>>,
        definition_arena: &Arena<_Definition<'a>>,
        variable_arena: &mut Arena<_Variable<'a>>,
        source_text_provider: &impl SourceTextProvider<'a>,
        reference: Id<_Reference<'a>>,
    ) {
        if !arena[self_].resolve(
            reference_arena,
            definition_arena,
            variable_arena,
            reference,
            source_text_provider,
        ) {
            trace!(?reference, "didn't resolve");

            Self::delegate_to_upper_scope(self_, arena, reference);
        }
    }

    pub fn resolve(
        &mut self,
        reference_arena: &mut Arena<_Reference<'a>>,
        definition_arena: &Arena<_Definition<'a>>,
        variable_arena: &mut Arena<_Variable<'a>>,
        reference: Id<_Reference<'a>>,
        source_text_provider: &impl SourceTextProvider<'a>,
    ) -> bool {
        let name = reference_arena[reference].node.text(source_text_provider);

        let Some(variables) = self.name_map().get(&name) else {
            return false;
        };
        let Some(variable) = find_resolution(
            variable_arena,
            definition_arena,
            self.id(),
            variables,
            &reference_arena[reference],
        ) else {
            return false;
        };

        trace!(?reference, ?variable, "resolved");

        variable_arena[variable].references.push(reference);
        reference_arena[reference].resolved = Some(variable);
        true
    }

    fn delegate_to_upper_scope(
        self_: Id<Self>,
        arena: &mut Arena<Self>,
        reference: Id<_Reference<'a>>,
    ) {
        if let Some(upper) = arena[self_].maybe_upper() {
            arena[upper].pre_resolved_mut().push(reference);
        }

        arena[self_].through_mut().push(reference);
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

    pub fn references(&self) -> impl Iterator<Item = Reference<'a, 'b>> + '_ {
        self.scope
            .base()
            .references
            .iter()
            .map(|reference| self.scope_analyzer.borrow_reference(*reference))
    }

    pub fn kind(&self) -> ScopeKind {
        self.scope.base().kind
    }

    pub fn through(&self) -> impl Iterator<Item = Reference<'a, 'b>> + '_ {
        self.scope
            .base()
            .through
            .iter()
            .map(|reference| self.scope_analyzer.borrow_reference(*reference))
    }
}

pub struct ScopeBase<'a> {
    kind: ScopeKind,
    variables: Vec<Id<_Variable<'a>>>,
    references: Vec<Id<_Reference<'a>>>,
    pre_resolved: Option<Vec<Id<_Reference<'a>>>>,
    through: Vec<Id<_Reference<'a>>>,
    upper: Option<Id<_Scope<'a>>>,
    id: Id<_Scope<'a>>,
    name_map: NameMap<'a>,
    #[allow(dead_code)]
    node: Node<'a>,
    child_scopes: Vec<Id<_Scope<'a>>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Root,
    Function,
}

fn find_resolution<'a>(
    variable_arena: &Arena<_Variable<'a>>,
    definition_arena: &Arena<_Definition<'a>>,
    scope: Id<_Scope<'a>>,
    variables: &[Id<_Variable<'a>>],
    reference: &_Reference<'a>,
) -> Option<Id<_Variable<'a>>> {
    variables
        .into_iter()
        .find(|&&variable| match reference.usage_kind {
            UsageKind::IdentifierReference => {
                match definition_arena[variable_arena[variable].definition].kind {
                    DefinitionKind::Variable => reference.scope == scope,
                    _ => true,
                }
            }
            UsageKind::AttributeName => {
                matches!(
                    definition_arena[variable_arena[variable].definition].kind,
                    DefinitionKind::Use
                )
            }
        })
        .copied()
}
