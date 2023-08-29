#![cfg(test)]

use tree_sitter_lint::{tree_sitter::{Parser, Tree}, tree_sitter_grep::SupportedLanguage};

use crate::scope_analysis::ScopeAnalyzer;

pub fn parse(source_text: &str) -> Tree {
    let mut parser = Parser::new();
    parser.set_language(SupportedLanguage::Rust.language()).unwrap();

    parser.parse(source_text, None).unwrap()
}

pub fn get_scope_analyzer<'a>(source_text: &'a str, tree: &'a Tree) -> ScopeAnalyzer<'a> {
    let mut scope_analyzer = ScopeAnalyzer::new(source_text);

    scope_analyzer.visit(tree.root_node());

    scope_analyzer
}
