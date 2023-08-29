use id_arena::{Arena, Id};
use squalid::EverythingExt;
use tree_sitter_lint::{
    tree_sitter::Node, tree_sitter_grep::SupportedLanguage, NodeExt, SourceTextProvider,
};

use crate::{
    kind::{Crate, Self_, Super, VisibilityModifier},
    path::SimplePath,
    ScopeAnalyzer,
};

#[derive(Debug)]
pub struct _Definition<'a> {
    pub kind: DefinitionKind,
    pub name: Node<'a>,
    pub node: Node<'a>,
    pub visibility: Visibility<'a>,
}

impl<'a> _Definition<'a> {
    pub fn new(
        arena: &mut Arena<Self>,
        kind: DefinitionKind,
        name: Node<'a>,
        node: Node<'a>,
        visibility: Visibility<'a>,
    ) -> Id<Self> {
        arena.alloc(Self {
            kind,
            name,
            node,
            visibility,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    Struct,
    Module,
    Function,
    ExternCrateDeclaration,
    Use,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Visibility<'a> {
    Pub,
    #[default]
    Private,
    PubInPath(SimplePath<'a>),
    PubCrate,
    PubSuper,
    PubSelf,
}

impl<'a> Visibility<'a> {
    pub fn from_visibility_modifier(
        node: Node<'a>,
        source_text_provider: &impl SourceTextProvider<'a>,
    ) -> Self {
        let mut children = node.non_comment_children(SupportedLanguage::Rust);
        match children.next().unwrap().kind() {
            "pub" => match children.next() {
                None => Self::Pub,
                Some(child) => {
                    assert_eq!(child.kind(), "(");
                    match children.next().unwrap().kind() {
                        Self_ => Self::PubSelf,
                        Super => Self::PubSuper,
                        Crate => Self::PubCrate,
                        "in" => Self::PubInPath(
                            SimplePath::from_node(children.next().unwrap(), source_text_provider)
                                .unwrap(),
                        ),
                        _ => unreachable!(),
                    }
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn from_item(node: Node<'a>, source_text_provider: &impl SourceTextProvider<'a>) -> Self {
        node.first_non_comment_named_child(SupportedLanguage::Rust)
            .when(|node| node.kind() == VisibilityModifier)
            .map(|node| Self::from_visibility_modifier(node, source_text_provider))
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct Definition<'a, 'b> {
    definition: &'b _Definition<'a>,
    #[allow(dead_code)]
    scope_analyzer: &'b ScopeAnalyzer<'a>,
}

impl<'a, 'b> Definition<'a, 'b> {
    pub fn new(definition: &'b _Definition<'a>, scope_analyzer: &'b ScopeAnalyzer<'a>) -> Self {
        Self {
            definition,
            scope_analyzer,
        }
    }

    pub fn kind(&self) -> DefinitionKind {
        self.definition.kind
    }

    pub fn name(&self) -> Node<'a> {
        self.definition.name
    }

    pub fn node(&self) -> Node<'a> {
        self.definition.node
    }
}
