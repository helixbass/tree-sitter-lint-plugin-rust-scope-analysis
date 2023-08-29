use tree_sitter_lint::tree_sitter::Node;

use crate::ScopeAnalyzer;

#[derive(Debug)]
pub struct _Definition<'a> {
    pub kind: DefinitionKind,
    pub name: Node<'a>,
    pub node: Node<'a>,
}

impl<'a> _Definition<'a> {
    pub fn new(
        kind: DefinitionKind,
        name: Node<'a>, node: Node<'a>) -> Self {
        Self {
            kind,
            name,
            node,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    Struct,
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
