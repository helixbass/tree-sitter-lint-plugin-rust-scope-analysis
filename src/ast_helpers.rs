use tree_sitter_lint::{tree_sitter::Node, SourceTextProvider, NodeExt};

use crate::kind::{TypeIdentifier, ScopedTypeIdentifier};

pub fn is_underscore<'a>(node: Node<'a>, source_text_provider: &impl SourceTextProvider<'a>) -> bool {
    node.text(source_text_provider) == "_"
}

pub fn is_simple_type_identifier(node: Node) -> bool {
    node.kind() == TypeIdentifier && node.parent().unwrap().kind() != ScopedTypeIdentifier
}
