#![cfg(test)]

use itertools::Itertools;
use speculoos::prelude::*;

use crate::{tests::helpers::tracing_subscribe, scope_analysis::DefinitionKind, kind::{ModItem, Identifier}};

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
