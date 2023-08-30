#![cfg(test)]

use itertools::Itertools;
use speculoos::prelude::*;

use crate::{
    kind::{TypeIdentifier, EnumItem},
    scope_analysis::DefinitionKind,
    tests::helpers::tracing_subscribe,
};

use super::helpers::{get_scope_analyzer, parse};

#[test]
fn test_enum_gets_added_to_scope() {
    tracing_subscribe();

    let source_text = "
        enum Foo {
            Bar,
        }
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(source_text, &tree);

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(1);
    let variable = &variables[0];
    assert_that!(&variable.definition().kind()).is_equal_to(DefinitionKind::Enum);
    assert_that!(&variable.name()).is_equal_to("Foo");
    assert_that!(&variable.definition().name().kind()).is_equal_to(TypeIdentifier);
    assert_that!(&variable.definition().node().kind()).is_equal_to(EnumItem);
    assert_that!(&variable.references().collect_vec()).is_empty();
}
