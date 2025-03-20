#![crate_name = "test_fmt"]

#[cfg(test)]
mod test_fmt {
    use lc3_toolchain::ast::get_ast;
    use lc3_toolchain::fmt::{FormatStyle, Formatter};
    use lc3_toolchain::test_utils::get_test_files;
    use std::path::PathBuf;

    const DEFAULT_STYLE: FormatStyle = FormatStyle {
        indent_directive: 3,
        indent_instruction: 4,
        indent_label: 0,
        indent_min_comment_from_block: 1,
        space_block_to_comment: 1,
        space_comment_stick_to_body: 0,
        space_from_label_block: 1,
        space_from_start_end_block: 1,
    };

    fn assert_true(path: &'static str) {
        let source_path = PathBuf::from("test/data/source").join(path);
        let expected_path = PathBuf::from("test/data/expected").join(path);
        let expected_files = get_test_files(&expected_path).expect("Path does not exist!");
        let source_files = get_test_files(&source_path).expect("Path does not exist!");
        let expected_file = expected_files
            .get(
                expected_path
                    .as_path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .as_ref(),
            )
            .expect("File does not exist!");
        let source_file = source_files
            .get(
                source_path
                    .as_path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .as_ref(),
            )
            .expect("File does not exist!");
        let program = get_ast(source_file);
        assert!(program.is_ok());
        let program = program.unwrap();
        let mut formatter = Formatter::new(&DEFAULT_STYLE);
        formatter.format(program);
        assert_eq!(
            expected_file,
            String::from_utf8_lossy(formatter.contents()).as_ref()
        );
    }

    #[test]
    fn test_1() {
        assert_true("fmt/normal.asm")
    }

    #[test]
    fn test_labels() {
        assert_true("fmt/labels.asm")
    }

    #[test]
    fn test_all() {
        assert_true("fmt/all.asm")
    }

    #[test]
    fn test_empty() {
        assert_true("fmt/empty.asm")
    }

    #[test]
    fn test_single_comment() {
        assert_true("fmt/single_comment.asm")
    }

    #[test]
    fn test_minimal_instruction() {
        assert_true("fmt/minimal_instruction.asm")
    }

    #[test]
    fn test_minimal_directive() {
        assert_true("fmt/minimal_directive.asm")
    }
}
