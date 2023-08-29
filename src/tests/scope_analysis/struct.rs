#![cfg(test)]

use itertools::Itertools;
use speculoos::prelude::*;

use super::helpers::{get_scope_analyzer, parse};

#[test]
fn test_struct_definition_gets_added_to_scope() {
    let source_text = "
        struct Foo {}
    ";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(
        source_text,
        &tree
    );

    let root_scope = scope_analyzer.root_scope();

    let variables = root_scope.variables().collect_vec();
    assert_that!(&variables).has_length(1);
}
