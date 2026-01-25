use kotlin_analyzer::analysis::SymbolIndex;
use kotlin_analyzer::parser::parse_with_ast;
use ropey::Rope;

#[test]
fn finds_symbol_by_name() {
    let source = r#"
        class Foo {}
        fun bar() {}
        val baz = 1
    "#;

    let (ast, diagnostics) = parse_with_ast(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    let file = ast.expect("expected ast");

    let rope = Rope::from_str(source);
    let index = SymbolIndex::from_file(&file, &rope);

    assert!(index.find("Foo").is_some());
    assert!(index.find("bar").is_some());
    assert!(index.find("baz").is_some());
}
