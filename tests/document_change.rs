use tower_lsp::lsp_types::{Position, Range, TextDocumentContentChangeEvent, Url};

use kotlin_analyzer::text::TextDocument;

#[test]
fn applies_incremental_change() {
    let uri = Url::parse("file:///test.kt").expect("valid uri");
    let mut doc = TextDocument::new(uri, 1, "val x = 1\n".to_string());

    let change = TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position::new(0, 4),
            end: Position::new(0, 5),
        }),
        range_length: None,
        text: "y".to_string(),
    };

    doc.apply_change(&change);

    assert_eq!(doc.text(), "val y = 1\n");
}
