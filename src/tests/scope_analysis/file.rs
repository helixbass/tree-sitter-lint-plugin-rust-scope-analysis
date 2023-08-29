#![cfg(test)]

use speculoos::prelude::*;

use super::helpers::{get_scope_analyzer, parse};

#[test]
fn test_empty_file_creates_root_scope() {
    let source_text = "";
    let tree = parse(source_text);
    let scope_analyzer = get_scope_analyzer(
        source_text,
        &tree
    );

    assert_that!(&scope_analyzer.scopes).has_length(1);
}
