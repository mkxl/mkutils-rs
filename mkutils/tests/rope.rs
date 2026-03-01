#![cfg(feature = "rope")]

use mkutils::{Atom, Rope, Utils};

fn test_atoms<'a>(mut atoms: impl Iterator<Item = Atom<'a>>, expected_string: &str) {
    let actual_string = atoms.collect_atoms();

    assert_eq!(expected_string, actual_string);
}

fn test_rope(rope: &Rope, expected_string: &str) {
    test_atoms(rope.atoms(), expected_string)
}

fn test_equality_impl(expected_string: &str) {
    let rope = Rope::from(expected_string);

    test_rope(&rope, expected_string);
}

fn test_collect_impl(strings: &[&str]) {
    let rope = strings.iter().copied().collect::<Rope>();
    let expected_string = strings.iter().copied().collect::<String>();

    test_rope(&rope, &expected_string);
}

// ============================================================================
// rope building / chunk splitting
// ============================================================================

#[test]
fn test_equality() {
    test_equality_impl("hello world");
    test_equality_impl("");
    test_equality_impl("\n\n\n");
}

#[test]
fn test_multi_chunk() {
    // Each ASCII character is one byte, so 2048 characters exceeds the 1024-byte chunk capacity.
    let string = (0u32..2048)
        .map(|i| (b'a' as u32) + (i % 26))
        .filter_map(char::from_u32)
        .collect::<String>();

    test_equality_impl(&string);
}

#[test]
fn test_multiple_pushes() {
    test_collect_impl(&["abc", "def"])
}

// ============================================================================
// atoms seeking (from_rope)
// ============================================================================

#[test]
fn atoms_seek_line_zero() {
    test_equality_impl("aaa\nbbb\nccc");
}

#[test]
fn atoms_seek_middle_line() {
    let rope = Rope::from("aaa\nbbb\nccc");
    let atoms = rope.atoms_at_line(1);

    test_atoms(atoms, "bbb\nccc");
}

#[test]
fn atoms_seek_last_line() {
    let rope = Rope::from("aaa\nbbb\nccc");
    let atoms = rope.atoms_at_line(2);

    test_atoms(atoms, "ccc");
}

#[test]
fn atoms_seek_past_end() {
    let rope = Rope::from("aaa\nbbb");
    let atoms = rope.atoms_at_line(99);

    test_atoms(atoms, "");
}

#[test]
fn atoms_seek_line_at_chunk_boundary() {
    // Fill a chunk to capacity with 'a's then a newline, so the next line starts in a new chunk.
    let text = std::format!("{:-^1023}\nsecond line", "");
    let rope = Rope::from(text.as_str());
    let atoms = rope.atoms_at_line(1);

    test_atoms(atoms, "second line");
}

// // ============================================================================
// // atoms rope_offset tracking
// // ============================================================================

// #[test]
// fn atoms_offset_increases_monotonically() {
//     let rope = Rope::from("ab\ncd");
//     let offsets: Vec<Distance> = rope.atoms().map(Atom.offset).collect();

//     for window in offsets.windows(2) {
//         assert!(
//             window[0].extended_graphemes < window[1].extended_graphemes,
//             "extended_graphemes offset should strictly increase: {:?} vs {:?}",
//             window[0],
//             window[1],
//         );
//     }
// }

// #[test]
// fn atoms_offset_tracks_newlines() {
//     let rope = Rope::from("a\nb");
//     let atoms: Vec<Atom<'_>> = rope.atoms().collect();

//     // 'a' at offset (newlines=0, eg=0)
//     assert_eq!(atoms[0].extended_grapheme, "a");
//     assert_eq!(atoms[0].offset.newlines, n(0));

//     // '\n' at offset (newlines=0, eg=1)
//     assert_eq!(atoms[1].extended_grapheme, "\n");
//     assert_eq!(atoms[1].offset.newlines, n(0));

//     // 'b' at offset (newlines=1, eg=2)
//     assert_eq!(atoms[2].extended_grapheme, "b");
//     assert_eq!(atoms[2].offset.newlines, n(1));
// }

// #[test]
// fn atoms_offset_across_chunk_boundary() {
//     // Create content spanning multiple chunks to verify offsets remain correct across boundaries.
//     let mut input = "x".repeat(1020);
//     input.push_str("yz\nabc");

//     let rope = Rope::from(input.as_str());
//     let atoms: Vec<Atom<'_>> = rope.atoms().collect();
//     let total = atoms.len();

//     // Last atom should have an offset consistent with total grapheme count minus one.
//     assert_eq!(atoms[total - 1].offset.extended_graphemes, g(total - 1));
// }

// // ============================================================================
// // lines — basic
// // ============================================================================

// #[test]
// fn lines_all_lines() {
//     let rope = Rope::from("aaa\nbbb\nccc");
//     let lines = collect_lines(&rope, 0..3, 0..80);

//     assert_eq!(lines, vec!["aaa\n", "bbb\n", "ccc"]);
// }

// #[test]
// fn lines_sub_range() {
//     let rope = Rope::from("aaa\nbbb\nccc");
//     let lines = collect_lines(&rope, 1..2, 0..80);

//     assert_eq!(lines, vec!["bbb\n"]);
// }

// #[test]
// fn lines_empty_range() {
//     let rope = Rope::from("aaa\nbbb\nccc");
//     let lines = collect_lines(&rope, 0..0, 0..80);

//     assert!(lines.is_empty());
// }

// // ============================================================================
// // lines — extended_graphemes range
// // ============================================================================

// #[test]
// fn lines_grapheme_sub_range() {
//     let rope = Rope::from("abcde\nfghij");
//     let lines = collect_lines(&rope, 0..2, 2..4);

//     assert_eq!(lines, vec!["cd", "hi"]);
// }

// #[test]
// fn lines_grapheme_start_beyond_line_length() {
//     let rope = Rope::from("ab\ncd");
//     let lines = collect_lines(&rope, 0..2, 99..120);

//     assert_eq!(lines, vec!["", ""]);
// }

// #[test]
// fn lines_grapheme_zero_width() {
//     let rope = Rope::from("abcde\nfghij");
//     let lines = collect_lines(&rope, 0..2, 0..0);

//     assert_eq!(lines, vec!["", ""]);
// }

// // ============================================================================
// // lines — partial consumption
// // ============================================================================

// #[test]
// fn lines_partial_consume_then_next_line() {
//     let rope = Rope::from("abcdefghij\nklmnopqrst\nuvwxyz");
//     let mut lines = rope.lines(n(0)..n(3), g(0)..g(80));

//     // Partially consume first line (take only 2 atoms).
//     {
//         let mut line = lines.next_line().unwrap();
//         let first = line.next().unwrap();
//         let second = line.next().unwrap();

//         assert_eq!(first.extended_grapheme, "a");
//         assert_eq!(second.extended_grapheme, "b");
//         // Drop line without consuming rest.
//     }

//     // Second line should be correct, not the remainder of the first.
//     let second_line: String = lines.next_line().unwrap().collect_atoms();

//     assert_eq!(second_line, "klmnopqrst\n");
// }

// #[test]
// fn lines_drop_without_consuming() {
//     let rope = Rope::from("aaa\nbbb\nccc\nddd");
//     let mut lines = rope.lines(n(0)..n(4), g(0)..g(80));

//     // Drop every line iterator without consuming any atoms.
//     let _ = lines.next_line().unwrap();
//     let _ = lines.next_line().unwrap();
//     let _ = lines.next_line().unwrap();
//     let _ = lines.next_line().unwrap();

//     // Should terminate after the range is exhausted.
//     assert!(lines.next_line().is_none());
// }

// #[test]
// fn lines_full_consume_does_not_skip() {
//     let rope = Rope::from("aaa\nbbb\nccc");
//     let mut lines = rope.lines(n(0)..n(3), g(0)..g(80));

//     // Fully consume first line (including the newline atom).
//     let first: String = lines.next_line().unwrap().collect_atoms();
//     assert_eq!(first, "aaa\n");

//     // Second line should be "bbb\n", not skipped.
//     let second: String = lines.next_line().unwrap().collect_atoms();
//     assert_eq!(second, "bbb\n");

//     let third: String = lines.next_line().unwrap().collect_atoms();
//     assert_eq!(third, "ccc");

//     assert!(lines.next_line().is_none());
// }

// #[test]
// fn lines_mixed_partial_and_full_consumption() {
//     let rope = Rope::from("111\n222\n333\n444");
//     let mut lines = rope.lines(n(0)..n(4), g(0)..g(80));

//     // Fully consume line 0.
//     let line0: String = lines.next_line().unwrap().collect_atoms();
//     assert_eq!(line0, "111\n");

//     // Drop line 1 without consuming.
//     let _ = lines.next_line().unwrap();

//     // Partially consume line 2 (1 atom).
//     {
//         let mut line2 = lines.next_line().unwrap();
//         let atom = line2.next().unwrap();
//         assert_eq!(atom.extended_grapheme, "3");
//     }

//     // Fully consume line 3.
//     let line3: String = lines.next_line().unwrap().collect_atoms();
//     assert_eq!(line3, "444");

//     assert!(lines.next_line().is_none());
// }

// // ============================================================================
// // lines — content spanning chunk boundaries
// // ============================================================================

// #[test]
// fn lines_line_spanning_chunks() {
//     // A single line longer than chunk capacity (1024 bytes).
//     let long_line: String = (0..1500).map(|i| (b'a' + (i % 26) as u8) as char).collect();
//     let rope = Rope::from(long_line.as_str());
//     let lines = collect_lines(&rope, 0..1, 0..2000);

//     assert_eq!(lines.len(), 1);
//     assert_eq!(lines[0], long_line);
// }

// #[test]
// fn lines_newline_at_chunk_boundary() {
//     // Put the newline right at the end of the first chunk.
//     let mut input = "x".repeat(1023);
//     input.push('\n');
//     input.push_str("second line");

//     let rope = Rope::from(input.as_str());
//     let lines = collect_lines(&rope, 0..2, 0..2000);

//     assert_eq!(lines.len(), 2);

//     let expected_first: String = "x".repeat(1023) + "\n";
//     assert_eq!(lines[0], expected_first);
//     assert_eq!(lines[1], "second line");
// }

// // ============================================================================
// // multi-byte / unicode
// // ============================================================================

// #[test]
// fn unicode_multibyte_codepoints() {
//     let input = "héllo\nwörld";
//     let rope = Rope::from(input);
//     let lines = collect_lines(&rope, 0..2, 0..80);

//     assert_eq!(lines, vec!["héllo\n", "wörld"]);
// }

// #[test]
// fn unicode_emoji() {
//     let input = "a😀b\nc😎d";
//     let rope = Rope::from(input);
//     let lines = collect_lines(&rope, 0..2, 0..80);

//     assert_eq!(lines, vec!["a😀b\n", "c😎d"]);
// }

// #[test]
// fn unicode_crlf_is_single_newline() {
//     let input = "aaa\r\nbbb\r\nccc";
//     let rope = Rope::from(input);

//     // CRLF should be a single extended grapheme treated as a newline.
//     let lines = collect_lines(&rope, 0..3, 0..80);

//     assert_eq!(lines.len(), 3);
//     assert_eq!(lines[0], "aaa\r\n");
//     assert_eq!(lines[1], "bbb\r\n");
//     assert_eq!(lines[2], "ccc");
// }

// #[test]
// fn unicode_flag_emoji() {
//     // Flag emoji are multi-codepoint extended graphemes (regional indicator pairs).
//     let input = "a🇺🇸b\nc🇬🇧d";
//     let rope = Rope::from(input);
//     let text: String = rope.atoms().collect_atoms();

//     assert_eq!(text, input);
// }

// // ============================================================================
// // edge cases
// // ============================================================================

// #[test]
// fn edge_no_newlines() {
//     let rope = Rope::from("hello");
//     let lines = collect_lines(&rope, 0..1, 0..80);

//     assert_eq!(lines, vec!["hello"]);
// }

// #[test]
// fn edge_no_trailing_newline() {
//     let rope = Rope::from("aaa\nbbb");
//     let lines = collect_lines(&rope, 0..2, 0..80);

//     assert_eq!(lines, vec!["aaa\n", "bbb"]);
// }

// #[test]
// fn edge_consecutive_newlines() {
//     let rope = Rope::from("a\n\n\nb");
//     let lines = collect_lines(&rope, 0..4, 0..80);

//     assert_eq!(lines, vec!["a\n", "\n", "\n", "b"]);
// }

// #[test]
// fn edge_single_char() {
//     let rope = Rope::from("x");
//     let lines = collect_lines(&rope, 0..1, 0..80);

//     assert_eq!(lines, vec!["x"]);
// }

// #[test]
// fn edge_single_newline() {
//     let rope = Rope::from("\n");
//     let lines = collect_lines(&rope, 0..1, 0..80);

//     assert_eq!(lines, vec!["\n"]);
// }

// #[test]
// fn edge_empty_rope_atoms() {
//     let rope = Rope::new();
//     let text: String = rope.atoms().collect_atoms();

//     assert_eq!(text, "");
// }

// #[test]
// fn edge_empty_rope_lines() {
//     let rope = Rope::new();
//     let lines = collect_lines(&rope, 0..1, 0..80);

//     assert!(lines.is_empty() || lines == vec![""]);
// }
