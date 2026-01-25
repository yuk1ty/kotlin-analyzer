use kotlin_analyzer::parser::{parse_with_ast, Decl, Mutability};

#[test]
fn builds_declaration_ast() {
    let source = r#"
        package demo
        import demo.Foo

        class Foo {}
        fun bar() {}
        val baz: Int = 1
    "#;

    let (file, diagnostics) = parse_with_ast(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let file = file.expect("expected AST output");
    assert_eq!(file.declarations.len(), 3);

    match &file.declarations[0] {
        Decl::Class(class_decl) => assert_eq!(class_decl.name.name, "Foo"),
        other => panic!("expected class decl, got {other:?}"),
    }

    match &file.declarations[1] {
        Decl::Fun(fun_decl) => assert_eq!(fun_decl.name.name, "bar"),
        other => panic!("expected fun decl, got {other:?}"),
    }

    match &file.declarations[2] {
        Decl::Property(prop_decl) => {
            assert_eq!(prop_decl.name.name, "baz");
            assert_eq!(prop_decl.mutability, Mutability::Val);
        }
        other => panic!("expected property decl, got {other:?}"),
    }
}
