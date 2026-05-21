use super::SourceFile;
use std::path::PathBuf;

#[test]
fn new_uses_filename_and_preserves_content() {
    let source = SourceFile::new(PathBuf::from("/tmp/demo.fab"), "a\nb\n".to_owned());

    assert_eq!(source.name, "demo.fab");
    assert_eq!(source.content, "a\nb\n");
}

#[test]
fn inline_sets_name_and_empty_path() {
    let source = SourceFile::inline("<stdin>", "incipit {}".to_owned());

    assert_eq!(source.name, "<stdin>");
    assert!(source.path.as_os_str().is_empty());
}

#[test]
fn offset_to_line_col_maps_offsets_across_lines() {
    let source = SourceFile::inline("x", "ab\ncd\nef".to_owned());

    assert_eq!(source.offset_to_line_col(0), (1, 1));
    assert_eq!(source.offset_to_line_col(2), (1, 3));
    assert_eq!(source.offset_to_line_col(3), (2, 1));
    assert_eq!(source.offset_to_line_col(6), (3, 1));
}

#[test]
fn line_content_returns_trimmed_lines_and_none_out_of_range() {
    let source = SourceFile::inline("x", "prima\nsecunda\ntertia".to_owned());

    assert_eq!(source.line_content(1), Some("prima"));
    assert_eq!(source.line_content(2), Some("secunda"));
    assert_eq!(source.line_content(3), Some("tertia"));
    assert_eq!(source.line_content(4), None);
    assert_eq!(source.line_content(0), Some("prima"));
}
