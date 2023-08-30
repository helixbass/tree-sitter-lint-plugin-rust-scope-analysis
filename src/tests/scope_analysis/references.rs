#![cfg(test)]

use itertools::Itertools;
use speculoos::prelude::*;

use crate::tests::helpers::tracing_subscribe;

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
}
