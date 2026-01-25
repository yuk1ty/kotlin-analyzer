use ropey::Rope;
use tower_lsp::lsp_types::Position;

use kotlin_analyzer::text::{char_idx_to_position, position_to_char_idx};

#[test]
fn position_to_char_idx_handles_utf16() {
    let text = Rope::from_str("a🍣b\n");

    let pos_a = Position::new(0, 0);
    let pos_after_a = Position::new(0, 1);
    let pos_after_emoji = Position::new(0, 3);
    let pos_after_b = Position::new(0, 4);

    assert_eq!(position_to_char_idx(&text, pos_a), 0);
    assert_eq!(position_to_char_idx(&text, pos_after_a), 1);
    assert_eq!(position_to_char_idx(&text, pos_after_emoji), 2);
    assert_eq!(position_to_char_idx(&text, pos_after_b), 3);
}

#[test]
fn char_idx_to_position_handles_utf16() {
    let text = Rope::from_str("a🍣b\n");

    let pos_a = char_idx_to_position(&text, 0);
    let pos_after_a = char_idx_to_position(&text, 1);
    let pos_after_emoji = char_idx_to_position(&text, 2);
    let pos_after_b = char_idx_to_position(&text, 3);

    assert_eq!(pos_a, Position::new(0, 0));
    assert_eq!(pos_after_a, Position::new(0, 1));
    assert_eq!(pos_after_emoji, Position::new(0, 3));
    assert_eq!(pos_after_b, Position::new(0, 4));
}
