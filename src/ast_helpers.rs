use tree_sitter_lint::{tree_sitter::Node, SourceTextProvider, NodeExt};

pub fn is_underscore<'a>(node: Node<'a>, source_text_provider: &impl SourceTextProvider<'a>) -> bool {
    node.text(source_text_provider) == "_"
}
