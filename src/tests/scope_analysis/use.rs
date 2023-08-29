#![cfg(test)]

use itertools::Itertools;
use speculoos::prelude::*;

use crate::{
    kind::{Identifier, UseDeclaration},
    scope_analysis::DefinitionKind,
    tests::helpers::tracing_subscribe,
};

use super::helpers::{get_scope_analyzer, parse};

#[test]
fn test_simple_use() {
    tracing_subscribe();

    let source_text = "
        use foo;
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(1);
    let variable = &variables[0];
    assert_that!(&variable.definition().kind()).is_equal_to(DefinitionKind::Use);
    assert_that!(&variable.name()).is_equal_to("foo");
    assert_that!(&variable.definition().name().kind()).is_equal_to(Identifier);
    assert_that!(&variable.definition().node().kind()).is_equal_to(UseDeclaration);
    assert_that!(&variable.references().collect_vec()).is_empty();
}

#[test]
fn test_scoped_use() {
    tracing_subscribe();

    let source_text = "
        use foo::bar;
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(1);
    let variable = &variables[0];
    assert_that!(&variable.definition().kind()).is_equal_to(DefinitionKind::Use);
    assert_that!(&variable.name()).is_equal_to("bar");
    assert_that!(&variable.definition().name().kind()).is_equal_to(Identifier);
    assert_that!(&variable.definition().node().kind()).is_equal_to(UseDeclaration);
    assert_that!(&variable.references().collect_vec()).is_empty();
}

#[test]
fn test_use_as() {
    tracing_subscribe();

    let source_text = "
        use foo::bar as baz;
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(1);
    let variable = &variables[0];
    assert_that!(&variable.definition().kind()).is_equal_to(DefinitionKind::Use);
    assert_that!(&variable.name()).is_equal_to("baz");
    assert_that!(&variable.definition().name().kind()).is_equal_to(Identifier);
    assert_that!(&variable.definition().node().kind()).is_equal_to(UseDeclaration);
    assert_that!(&variable.references().collect_vec()).is_empty();
}

#[test]
fn test_use_list() {
    tracing_subscribe();

    let source_text = "
        use foo::{bar as baz, quux};
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(2);
    let variable = &variables[0];
    assert_that!(&variable.definition().kind()).is_equal_to(DefinitionKind::Use);
    assert_that!(&variable.name()).is_equal_to("baz");
    assert_that!(&variable.definition().name().kind()).is_equal_to(Identifier);
    assert_that!(&variable.definition().node().kind()).is_equal_to(UseDeclaration);
    assert_that!(&variable.references().collect_vec()).is_empty();
    assert_that!(&variables[1].name()).is_equal_to("quux");
}
