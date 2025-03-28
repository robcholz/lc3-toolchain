#![crate_name = "test_fmt"]

mod test_lint;

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
        colon_after_label: true,
        fixed_body_comment_indent: false,
        directive_label_wrap: true,
    };

    const NO_COLON_STYLE: FormatStyle = FormatStyle {
        indent_directive: 3,
        indent_instruction: 4,
        indent_label: 0,
        indent_min_comment_from_block: 1,
        space_block_to_comment: 1,
        space_comment_stick_to_body: 0,
        space_from_label_block: 1,
        space_from_start_end_block: 1,
        colon_after_label: false,
        fixed_body_comment_indent: false,
        directive_label_wrap: true,
    };

    const FLEXIBLE_BODY_COMMENT_INDENT: FormatStyle = FormatStyle {
        indent_directive: 3,
        indent_instruction: 4,
        indent_label: 0,
        indent_min_comment_from_block: 1,
        space_block_to_comment: 1,
        space_comment_stick_to_body: 0,
        space_from_label_block: 1,
        space_from_start_end_block: 1,
        colon_after_label: true,
        fixed_body_comment_indent: true,
        directive_label_wrap: true,
    };

    const DISABLE_DIRECTIVE_LABEL_WRAP: FormatStyle = FormatStyle {
        indent_directive: 3,
        indent_instruction: 4,
        indent_label: 0,
        indent_min_comment_from_block: 1,
        space_block_to_comment: 1,
        space_comment_stick_to_body: 0,
        space_from_label_block: 1,
        space_from_start_end_block: 1,
        colon_after_label: true,
        fixed_body_comment_indent: true,
        directive_label_wrap: false,
    };

    fn assert_true(style: &FormatStyle, path: &'static str) {
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
        let mut formatter = Formatter::new(style);
        formatter.format(program);
        assert_eq!(
            expected_file,
            String::from_utf8_lossy(formatter.contents()).as_ref()
        );
    }

    #[test]
    fn test_1() {
        assert_true(&DEFAULT_STYLE, "fmt/normal.asm")
    }

    #[test]
    fn test_labels() {
        assert_true(&DEFAULT_STYLE, "fmt/labels.asm")
    }

    #[test]
    fn test_all() {
        assert_true(&DEFAULT_STYLE, "fmt/all.asm")
    }

    #[test]
    fn test_empty() {
        assert_true(&DEFAULT_STYLE, "fmt/empty.asm")
    }

    #[test]
    fn test_single_comment() {
        assert_true(&DEFAULT_STYLE, "fmt/single_comment.asm")
    }

    #[test]
    fn test_minimal_instruction() {
        assert_true(&DEFAULT_STYLE, "fmt/minimal_instruction.asm")
    }

    #[test]
    fn test_minimal_directive() {
        assert_true(&DEFAULT_STYLE, "fmt/minimal_directive.asm")
    }

    #[test]
    fn test_no_colon_labels() {
        assert_true(&NO_COLON_STYLE, "fmt/no_colon_labels.asm")
    }

    #[test]
    fn test_flexible_block_comment_indent() {
        assert_true(
            &FLEXIBLE_BODY_COMMENT_INDENT,
            "fmt/flexible_comment_indent.asm",
        )
    }

    #[test]
    fn test_disable_directive_label_wrap() {
        assert_true(
            &DISABLE_DIRECTIVE_LABEL_WRAP,
            "fmt/directive_label_wrap.asm",
        )
    }
}
