#![cfg(test)]

use itertools::Itertools;
use speculoos::prelude::*;
use tree_sitter_lint::NodeExt;

use crate::{tests::helpers::tracing_subscribe, scope_analysis::{ScopeKind, UsageKind}};

use super::helpers::{get_scope_analyzer, parse};

#[test]
fn test_reference_in_type_alias_gets_resolved() {
    tracing_subscribe();

    let source_text = "
        struct Foo {}

        type Bar = Foo;
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(2);
    let references = root_scope.references().collect_vec();
    assert_that!(&references).has_length(1);
    let variable_foo = &variables[0];
    let references_foo = variable_foo.references().collect_vec();
    assert_that!(&references_foo).has_length(1);
    assert_that!(&references[0].resolved()).is_some().is_equal_to(variable_foo);
    // assert_that!(&references[0].usage_kind()).is_equal_to(UsageKind::TypeReference);
    assert_that!(&references[0].usage_kind()).is_equal_to(UsageKind::IdentifierReference);
}

#[test]
fn test_reference_in_function_scope_gets_resolved_locally() {
    tracing_subscribe();

    let source_text = "
        fn whee() -> usize {
            let foo = 3;

            foo + 2
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let function_scope = scope_analyzer.scopes().nth(1).unwrap();
    assert_that!(&function_scope.kind()).is_equal_to(ScopeKind::Function);

    let variables = function_scope.variables().collect_vec();
    assert_that!(&variables).has_length(1);
    let variable_foo = &variables[0];
    let references = function_scope.references().collect_vec();
    assert_that!(&references).has_length(1);
    assert_that!(&references[0].usage_kind()).is_equal_to(UsageKind::IdentifierReference);
    let references_foo = variable_foo.references().collect_vec();
    assert_that!(&references_foo).has_length(1);
}

#[test]
fn test_reference_in_nested_function_scope_doesnt_get_resolved_to_outer_function_scope() {
    tracing_subscribe();

    let source_text = "
        fn whee() -> usize {
            let foo = 3;

            fn whoo() -> usize {
                foo + 2
            }

            whoo()
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let outer_function_scope = scope_analyzer.scopes().nth(1).unwrap();
    let inner_function_scope = scope_analyzer.scopes().nth(2).unwrap();

    let outer_through = outer_function_scope.through().collect_vec();
    let inner_through = inner_function_scope.through().collect_vec();
    assert_that!(&outer_through).has_length(1);
    assert_that!(&inner_through).has_length(1);
    assert_that(&&*outer_through[0].node().text(&scope_analyzer)).is_equal_to("foo");
    assert_that(&&*inner_through[0].node().text(&scope_analyzer)).is_equal_to("foo");
}

#[test]
fn test_reference_in_function_scope_gets_resolved_to_static() {
    tracing_subscribe();

    let source_text = "
        static FOO: usize = 0;

        fn whee() -> usize {
            FOO + 2
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.scopes().next().unwrap();
    let function_scope = scope_analyzer.scopes().nth(1).unwrap();

    let root_through = root_scope.through().collect_vec();
    let function_through = function_scope.through().collect_vec();
    assert_that!(&root_through).is_empty();
    assert_that!(&function_through).has_length(1);
}

#[test]
fn test_reference_in_struct_field_type_gets_resolved() {
    tracing_subscribe();

    let source_text = "
        struct Foo {}

        struct Bar {
            foo: Foo,
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    let variable_foo = &variables[0];
    assert_that!(&variable_foo.name()).is_equal_to("Foo");

    let references = root_scope.references().collect_vec();
    assert_that!(&references).has_length(1);

    let references_foo = variable_foo.references().collect_vec();
    assert_that!(&references_foo).has_length(1);

    assert_that!(&references[0].resolved()).is_some().is_equal_to(variable_foo);
    // assert_that!(&references[0].usage_kind()).is_equal_to(UsageKind::TypeReference);
    assert_that!(&references[0].usage_kind()).is_equal_to(UsageKind::IdentifierReference);
}

#[test]
fn test_enum_variant_doesnt_add_reference() {
    tracing_subscribe();

    let source_text = "
        enum Foo {
            Bar,
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    assert_that!(&root_scope.variables().collect_vec()).has_length(1);
    assert_that!(&root_scope.references().collect_vec()).is_empty();
}

#[test]
fn test_scoped_path_reference_gets_resolved() {
    tracing_subscribe();

    let source_text = "
        enum Foo {
            Bar,
        }

        let foo = Foo::Bar;
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    let variable_foo = &variables[0];
    assert_that!(&variable_foo.name()).is_equal_to("Foo");

    let references = root_scope.references().collect_vec();
    let references_foo = variable_foo.references().collect_vec();

    assert_that!(&references_foo).has_length(1);
    assert_that!(&references[0].resolved()).is_some().is_equal_to(variable_foo);
}

#[test]
fn test_attribute_reference_gets_resolved() {
    tracing_subscribe();

    let source_text = "
        use foo::bar;

        #[bar]
        struct Foo;
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    let variable_bar = &variables[0];
    assert_that!(&variable_bar.name()).is_equal_to("bar");

    let references_bar = variable_bar.references().collect_vec();

    assert_that!(&references_bar).has_length(1);
    assert_that!(&references_bar[0].resolved()).is_some().is_equal_to(variable_bar);
    assert_that!(&references_bar[0].usage_kind()).is_equal_to(UsageKind::AttributeName);
}

#[test]
fn test_macro_reference_gets_resolved() {
    tracing_subscribe();

    let source_text = "
        use foo::bar;

        fn whee() {
            bar!();
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    let variable_bar = &variables[0];
    assert_that!(&variable_bar.name()).is_equal_to("bar");

    let references_bar = variable_bar.references().collect_vec();

    assert_that!(&references_bar).has_length(1);
    assert_that!(&references_bar[0].usage_kind()).is_equal_to(UsageKind::Macro);
}
