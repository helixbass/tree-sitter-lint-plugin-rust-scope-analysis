#![cfg(test)]

use itertools::Itertools;
use speculoos::prelude::*;

use crate::{
    kind::{Identifier, MacroDefinition},
    scope_analysis::DefinitionKind,
    tests::helpers::tracing_subscribe,
};

use super::helpers::{get_scope_analyzer, parse};

#[test]
fn test_macro_gets_added_to_scope() {
    tracing_subscribe();

    let source_text = "
        macro_rules! foo {
            ($whee:expr) => {}
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(1);
    let variable = &variables[0];
    assert_that!(&variable.definition().kind()).is_equal_to(DefinitionKind::Macro);
    assert_that!(&variable.name()).is_equal_to("foo");
    assert_that!(&variable.definition().name().kind()).is_equal_to(Identifier);
    assert_that!(&variable.definition().node().kind()).is_equal_to(MacroDefinition);
    assert_that!(&variable.references().collect_vec()).is_empty();
}
