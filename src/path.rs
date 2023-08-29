use std::borrow::Cow;

use tree_sitter_lint::{tree_sitter::Node, NodeExt, SourceTextProvider};

use crate::kind::{BracketedType, GenericType, Metavariable, ScopedIdentifier};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimplePath<'a> {
    segments: Vec<Cow<'a, str>>,
}

impl<'a> SimplePath<'a> {
    pub fn new(segments: Vec<Cow<'a, str>>) -> Self {
        Self { segments }
    }

    fn and_push(mut self, segment: Cow<'a, str>) -> Self {
        self.segments.push(segment);
        self
    }

    pub fn from_node(
        node: Node<'a>,
        source_text_provider: &impl SourceTextProvider<'a>,
    ) -> Option<Self> {
        match node.kind() {
            ScopedIdentifier => {
                let path = node.field("path");
                match path.kind() {
                    BracketedType | GenericType => None,
                    _ => Self::from_node(path, source_text_provider).map(|path| {
                        path.and_push(node.field("name").text(source_text_provider))
                    }),
                }
            }
            Metavariable => None,
            _ => Some(Self::new(vec![node.text(source_text_provider)])),
        }
    }
}
