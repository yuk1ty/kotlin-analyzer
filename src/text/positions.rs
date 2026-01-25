use ropey::Rope;
use ropey::RopeSlice;
use tower_lsp::lsp_types::Position;

fn utf16_col_to_char_offset(line: RopeSlice<'_>, utf16_col: usize) -> usize {
    let mut units = 0;
    let mut chars = 0;

    for ch in line.chars() {
        let len = ch.len_utf16();
        if units + len > utf16_col {
            break;
        }
        units += len;
        chars += 1;
    }

    chars
}

fn char_offset_to_utf16_col(line: RopeSlice<'_>, char_offset: usize) -> usize {
    line.chars()
        .take(char_offset)
        .map(|ch| ch.len_utf16())
        .sum()
}

pub fn position_to_char_idx(text: &Rope, position: Position) -> usize {
    let total_lines = text.len_lines();
    if total_lines == 0 {
        return 0;
    }

    let line = position.line as usize;
    if line >= total_lines {
        return text.len_chars();
    }

    let line_start = text.line_to_char(line);
    let line_slice = text.line(line);
    let col = utf16_col_to_char_offset(line_slice, position.character as usize);
    let line_chars = line_slice.len_chars();
    let clamped_col = col.min(line_chars);

    line_start + clamped_col
}

pub fn char_idx_to_position(text: &Rope, char_idx: usize) -> Position {
    let capped = char_idx.min(text.len_chars());
    if text.len_lines() == 0 {
        return Position::new(0, 0);
    }

    let line = text.char_to_line(capped);
    let line_start = text.line_to_char(line);
    let line_slice = text.line(line);
    let char_offset = capped.saturating_sub(line_start);
    let utf16_col = char_offset_to_utf16_col(line_slice, char_offset);

    Position::new(line as u32, utf16_col as u32)
}
