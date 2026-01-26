use kotlin_analyzer::parser::parse;

#[test]
fn parses_basic_declarations_without_errors() {
    let source = r#"
        package demo
        import demo.Foo

        class Foo<T>(val value: T) : Bar<Baz> {}

        fun <T> bar(x: T, y: List<String?> = listOf("x")): List<T> {
            val x = 1 + 2
            val y = if (x > 0) { x } else { 0 }
            val z = when (y) { 0 -> 1 else -> 2 }
            val f = { a: Int -> a + 1 }
        }
    "#;

    let diagnostics = parse(source);
    assert!(diagnostics.is_empty(), "expected no diagnostics: {diagnostics:?}");
}

#[test]
fn reports_unclosed_brace() {
    let source = "fun bar() { val x = 1";
    let diagnostics = parse(source);
    assert!(!diagnostics.is_empty());
}

#[test]
fn reports_unexpected_character() {
    let source = "val x = #";
    let diagnostics = parse(source);
    assert!(!diagnostics.is_empty());
}
