#![cfg(test)]

use itertools::Itertools;
use speculoos::prelude::*;

use crate::{tests::helpers::tracing_subscribe, scope_analysis::{DefinitionKind, ScopeKind}, kind::{ModItem, Identifier}};

use super::helpers::{get_scope_analyzer, parse};

#[test]
fn test_module_definition_gets_added_to_scope() {
    tracing_subscribe();

    let source_text = "
        mod foo {}
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(
        source_text,
        &tree
    );

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(1);
    let variable = &variables[0];
    assert_that!(&variable.definition().kind()).is_equal_to(DefinitionKind::Module);
    assert_that!(&variable.name()).is_equal_to("foo");
    assert_that!(&variable.definition().name().kind()).is_equal_to(Identifier);
    assert_that!(&variable.definition().node().kind()).is_equal_to(ModItem);
    assert_that!(&variable.references().collect_vec()).is_empty();
}

#[test]
fn test_module_definition_creates_new_scope() {
    tracing_subscribe();

    let source_text = "
        use bar::Baz;

        mod foo {
            use quux::Boom;

            type Whee = Boom;
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(
        source_text,
        &tree
    );

    let root_scope = scope_analyzer.scopes().next().unwrap();
    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(2);
    assert_that!(&variables[0].name()).is_equal_to("Baz");
    assert_that!(&variables[1].name()).is_equal_to("foo");

    let module_scope = scope_analyzer.scopes().nth(1).unwrap();
    assert_that!(&module_scope.kind()).is_equal_to(ScopeKind::Module);

    let variables = module_scope.variables().collect_vec();
    assert_that!(&variables).has_length(2);
    assert_that!(&variables[0].name()).is_equal_to("Boom");
    assert_that!(&variables[1].name()).is_equal_to("Whee");

    let variable = &variables[0];
    assert_that!(&variable.references().collect_vec()).has_length(1);
}
