use tree_sitter_lint::{
    tree_sitter::Node, tree_sitter_grep::SupportedLanguage, NodeExt, SourceTextProvider,
};

use crate::kind::{
    BracketedType, EnumVariant, GenericType, Identifier, QualifiedType, ScopedIdentifier,
    ScopedTypeIdentifier, TypeIdentifier, Attribute,
};

pub fn is_underscore<'a>(
    node: Node<'a>,
    source_text_provider: &impl SourceTextProvider<'a>,
) -> bool {
    node.text(source_text_provider) == "_"
}

#[allow(dead_code)]
pub fn is_simple_type_identifier(node: Node) -> bool {
    node.kind() == TypeIdentifier && node.parent().unwrap().kind() != ScopedTypeIdentifier
}

pub fn is_simple_identifier(node: Node) -> bool {
    node.kind() == Identifier && node.parent().unwrap().kind() != ScopedIdentifier
}

#[macro_export]
macro_rules! assert_kind {
    ($node:expr, $kind:pat) => {
        assert!(
            matches!($node.kind(), $kind),
            "Expected kind {:?}, got: {:?}",
            stringify!($kind),
            $node.kind()
        );
    };
}

pub fn get_leading_name_node_of_scoped_identifier(node: Node) -> Option<Node> {
    assert_kind!(node, ScopedTypeIdentifier | ScopedIdentifier);

    let path = node.child_by_field_name("path")?;

    match path.kind() {
        Identifier => Some(path),
        ScopedIdentifier => get_leading_name_node_of_scoped_identifier(path),
        BracketedType => {
            let bracketed_type = path.first_non_comment_named_child(SupportedLanguage::Rust);
            match bracketed_type.kind() {
                QualifiedType => get_leading_name_node_of_type(bracketed_type.field("type")),
                _ => get_leading_name_node_of_type(bracketed_type),
            }
        }
        GenericType => get_leading_name_node_of_type(path.field("type")),
        _ => None,
    }
}

pub fn get_leading_name_node_of_type(node: Node) -> Option<Node> {
    match node.kind() {
        TypeIdentifier => Some(node),
        ScopedTypeIdentifier => get_leading_name_node_of_scoped_identifier(node),
        _ => None,
    }
}

pub fn is_enum_variant_name(node: Node) -> bool {
    let parent = node.parent().unwrap();
    parent.kind() == EnumVariant && node == parent.field("name")
}

pub fn is_attribute_name(mut node: Node) -> bool {
    loop {
        let Some(parent) = node.parent() else {
            return false;
        };
        match parent.kind() {
            Attribute => {
                return node == parent.first_non_comment_named_child(SupportedLanguage::Rust)
            }
            Identifier | ScopedIdentifier => (),
            _ => return false,
        }
        node = parent;
    }
}
