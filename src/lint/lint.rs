use crate::ast::processed_ast::{LineColumn, Program, ProgramItem};
use crate::ast::raw_ast::{Comment, Directive, Instruction, Label, Span};
use getset::Getters;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CaseStyle {
    LowerCamelCase,
    UpperCamelCase,
    SnakeCase,
    ScreamingSnakeCase,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LintStyle {
    pub colon_after_label: bool,
    pub label_style: CaseStyle,
    pub instruction_style: CaseStyle,
    pub directive_style: CaseStyle,
}

#[derive(Debug, Getters)]
pub struct Error {
    #[get = "pub"]
    case_style_error: Result<(), (CaseStyle, Option<CaseStyle>)>,
    #[get = "pub"]
    colon_style_error: Result<(), ()>,
    #[get = "pub"]
    span: Span,
}

pub struct Linter {
    program: Program,
    visitor: Box<dyn ProgramItemVisitor>,
}

impl Linter {
    pub fn new(style: LintStyle, program: Program) -> Self {
        Self {
            program,
            visitor: Box::new(StyleCheckerVisitor { style }),
        }
    }

    pub fn check(&mut self) -> Result<(), Vec<Error>> {
        self.accept()
    }

    fn accept(&mut self) -> Result<(), Vec<Error>> {
        let mut errors = vec![];
        for line in self.program.items() {
            let mut res = match line {
                ProgramItem::Comment(comment, lc) => self.visitor.visit_comment(comment, lc),
                ProgramItem::Instruction(labels, instruction, comment, lc) => self
                    .visitor
                    .visit_instruction(labels, instruction, comment, lc),
                ProgramItem::Directive(labels, directive, comment, lc) => {
                    self.visitor.visit_directive(labels, directive, comment, lc)
                }
                ProgramItem::EOL(labels) => self.visitor.visit_eol(labels),
            };
            errors.append(&mut res);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

trait ProgramItemVisitor {
    fn visit_comment(&mut self, comment: &Comment, location: &LineColumn) -> Vec<Error>;
    fn visit_instruction(
        &mut self,
        labels: &[Label],
        instruction: &Instruction,
        comment: &Option<Comment>,
        location: &LineColumn,
    ) -> Vec<Error>;
    fn visit_directive(
        &mut self,
        labels: &[Label],
        directive: &Directive,
        comment: &Option<Comment>,
        location: &LineColumn,
    ) -> Vec<Error>;
    fn visit_eol(&mut self, labels: &[Label]) -> Vec<Error>;
}

struct StyleCheckerVisitor {
    style: LintStyle,
}

static LOWER_CAMEL: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z]+(?:[A-Z][a-z0-9]*)*$").unwrap());
static UPPER_CAMEL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z][a-z0-9]*(?:[A-Z][a-z0-9]*)*$").unwrap());
static SNAKE_CASE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z]+(?:_[a-z0-9]+)*$").unwrap());
static SCREAMING_SNAKE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z0-9]+(?:_[A-Z0-9]+)*$").unwrap());

impl StyleCheckerVisitor {
    fn check_label(&self, label: &str) -> (Result<(), Option<CaseStyle>>, Result<(), ()>) {
        let case_error = Self::check_keyword_style(
            label.strip_suffix(":").unwrap_or_else(|| label),
            &self.style.label_style,
        );
        let colon_error = match label.ends_with(":") {
            true => {
                if self.style.colon_after_label {
                    Ok(())
                } else {
                    Err(())
                }
            }
            false => {
                if self.style.colon_after_label {
                    Err(())
                } else {
                    Ok(())
                }
            }
        };
        (case_error, colon_error)
    }

    fn check_instruction(&self, instruction: &str) -> Result<(), Option<CaseStyle>> {
        Self::check_keyword_style(instruction, &self.style.instruction_style)
    }

    fn check_directive_style(&self, directive: &str) -> Result<(), Option<CaseStyle>> {
        Self::check_keyword_style(directive, &self.style.directive_style)
    }

    fn check_keyword_style(keyword: &str, case_style: &CaseStyle) -> Result<(), Option<CaseStyle>> {
        let found_style = match Self::get_identifier_style(keyword) {
            None => {
                return Err(None);
            }
            Some(st) => st,
        };
        if &found_style == case_style {
            Ok(())
        }
        // snakecase is a subset of lower camelcase without _
        else if found_style == CaseStyle::SnakeCase
            && (!keyword.contains("_"))
            && *case_style == CaseStyle::LowerCamelCase
        {
            Ok(())
        } else {
            Err(Some(found_style))
        }
    }
    fn get_identifier_style(identifier: &str) -> Option<CaseStyle> {
        if SNAKE_CASE.is_match(identifier) {
            Some(CaseStyle::SnakeCase)
        } else if SCREAMING_SNAKE.is_match(identifier) {
            Some(CaseStyle::ScreamingSnakeCase)
        } else if LOWER_CAMEL.is_match(identifier) {
            Some(CaseStyle::LowerCamelCase)
        } else if UPPER_CAMEL.is_match(identifier) {
            Some(CaseStyle::UpperCamelCase)
        } else {
            None
        }
    }

    fn label_error_to_error(
        label: &Label,
        expected_case: &CaseStyle,
        case_error: Result<(), Option<CaseStyle>>,
        colon_error: Result<(), ()>,
    ) -> Error {
        Error {
            case_style_error: case_error.map_err(|e| (expected_case.clone(), e)),
            colon_style_error: colon_error,
            span: label.span().clone(),
        }
    }

    fn check_label_style(&self, labels: &[Label]) -> Vec<Error> {
        let mut errors = vec![];
        for label in labels {
            let (case_error, colon_error) = self.check_label(label.content());
            if case_error.is_err() || colon_error.is_err() {
                errors.push(Self::label_error_to_error(
                    label,
                    &self.style.label_style,
                    case_error,
                    colon_error,
                ))
            }
        }
        errors
    }
}

impl ProgramItemVisitor for StyleCheckerVisitor {
    fn visit_comment(&mut self, _: &Comment, _: &LineColumn) -> Vec<Error> {
        vec![]
    }

    fn visit_instruction(
        &mut self,
        labels: &[Label],
        instruction: &Instruction,
        comment: &Option<Comment>,
        lc: &LineColumn,
    ) -> Vec<Error> {
        let mut errors = vec![];
        errors.append(&mut self.check_label_style(labels));
        match self.check_instruction(instruction.content()) {
            Ok(_) => {}
            Err(err) => errors.push(Error {
                case_style_error: Err((self.style.instruction_style.clone(), err)),
                colon_style_error: Ok(()),
                span: instruction.span().clone(),
            }),
        }
        match comment {
            None => {}
            Some(comment) => {
                errors.append(&mut self.visit_comment(comment, lc));
            }
        }
        errors
    }

    fn visit_directive(
        &mut self,
        labels: &[Label],
        directive: &Directive,
        comment: &Option<Comment>,
        lc: &LineColumn,
    ) -> Vec<Error> {
        let mut errors = vec![];
        errors.append(&mut self.check_label_style(labels));
        assert!(directive.content().starts_with("."));
        match self.check_directive_style(directive.content().strip_prefix(".").unwrap()) {
            Ok(_) => {}
            Err(error) => {
                errors.push(Error {
                    case_style_error: Err((self.style.directive_style.clone(), error)),
                    colon_style_error: Ok(()),
                    span: directive.span().clone(),
                });
            }
        }
        match comment {
            None => {}
            Some(comment) => {
                errors.append(&mut self.visit_comment(comment, lc));
            }
        }
        errors
    }

    fn visit_eol(&mut self, labels: &[Label]) -> Vec<Error> {
        let mut errors = vec![];
        errors.append(&mut self.check_label_style(labels));
        errors
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::get_ast;

    fn test_true(style: LintStyle, content: &str) {
        let ast = get_ast(content);
        assert!(ast.is_ok());
        match ast {
            Ok(program) => {
                let c = Linter::new(style, program).check();
                if c.is_err() {
                    println!("{:?}", c.as_ref().err().unwrap());
                }
                assert!(c.is_ok());
            }
            Err(_) => {}
        }
    }

    fn test_false(style: LintStyle, content: &str) {
        let ast = get_ast(content);
        assert!(ast.is_ok());
        match ast {
            Ok(program) => {
                let c = Linter::new(style, program).check();
                assert!(c.is_err());
            }
            Err(_) => {}
        }
    }

    #[test]
    fn test_empty() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::LowerCamelCase,
            directive_style: CaseStyle::LowerCamelCase,
        };
        let content = r#""#;
        test_true(style, content);
    }

    #[test]
    fn test_directive_uppercase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::UpperCamelCase,
            directive_style: CaseStyle::ScreamingSnakeCase,
        };
        let content_true = r#".ORIG x3000 .END"#;
        let content_false1 = r#".OrIG x3000 .EnD"#;
        let content_false2 = r#".orig x3000 .end"#;
        let content_false3 = r#".Orig x3000 .End"#;
        test_true(style.clone(), content_true);
        test_false(style.clone(), content_false1);
        test_false(style.clone(), content_false2);
        test_false(style.clone(), content_false3);
    }

    #[test]
    fn test_directive_lowercamelcase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::UpperCamelCase,
            directive_style: CaseStyle::LowerCamelCase,
        };
        let content_true1 = r#".oRIG x3000 .eND"#;
        let content_true2 = r#".orig x3000 .eND"#;
        test_true(style, content_true1);
        test_true(style, content_true2);
    }

    #[test]
    fn test_directive_snakecase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::UpperCamelCase,
            directive_style: CaseStyle::SnakeCase,
        };
        let content_true1 = r#".orig x3000 .end"#;
        test_true(style, content_true1);
    }

    #[test]
    fn test_instruction_uppercamelcase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::UpperCamelCase,
            directive_style: CaseStyle::LowerCamelCase,
        };

        let content_true1 = r#"And R1, R2, R3"#;
        let content_true2 = r#"And R4, R5, R6"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"add R1, R2, R3"#; // LowerCamelCase
        let content_false2 = r#"add R1, R2, R3"#; // SnakeCase
        test_false(style, content_false1);
        test_false(style, content_false2);
    }

    #[test]
    fn test_instruction_screaming_camelcase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::ScreamingSnakeCase,
            directive_style: CaseStyle::LowerCamelCase,
        };

        let content_true1 = r#"AND R1, R2, R3"#;
        let content_true2 = r#"AND R4, R5, R6"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"aND R1, R2, R3"#; // LowerCamelCase
        let content_false2 = r#"AnD R1, R2, R3"#; // SnakeCase
        test_false(style, content_false1);
        test_false(style, content_false2);
    }

    #[test]
    fn test_instruction_lowercamelcase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::LowerCamelCase,
            directive_style: CaseStyle::LowerCamelCase,
        };

        let content_true1 = r#"add R1, R2, R3"#;
        let content_true2 = r#"and R4, R5, R6"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"ADD R1, R2, R3"#; // UpperCamelCase
        let content_false2 = r#"add_r1, r2, r3"#; // SnakeCase
        test_false(style, content_false1);
        test_false(style, content_false2);
    }

    #[test]
    fn test_instruction_snakecase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::SnakeCase,
            directive_style: CaseStyle::LowerCamelCase,
        };

        let content_true1 = r#"add R1, R2, R3"#;
        let content_true2 = r#"and R4, R5, R6"#;
        let content_true3 = r#"add R1, R2, R3"#; // LowerCamelCase
        test_true(style, content_true1);
        test_true(style, content_true2);
        test_true(style, content_true3);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"ADD R1, R2, R3"#; // UpperCamelCase
        test_false(style, content_false1);
    }

    #[test]
    fn test_label_lowercamelcase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::LowerCamelCase,
            instruction_style: CaseStyle::ScreamingSnakeCase,
            directive_style: CaseStyle::ScreamingSnakeCase,
        };

        let content_true1 = r#"loop: ADD R1, R2, R3"#;
        let content_true2 = r#"startLabel: AND R4, R5, R6"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"Loop: ADD R1, R2, R3"#; // UpperCamelCase
        let content_false2 = r#"start_label: AND R4, R5, R6"#; // SnakeCase
        let content_false3 = r#"START_LABEL: ADD R1, R2, R3"#; // ScreamingSnakeCase
        test_false(style, content_false1);
        test_false(style, content_false2);
        test_false(style, content_false3);
    }

    #[test]
    fn test_label_uppercamelcase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::UpperCamelCase,
            instruction_style: CaseStyle::ScreamingSnakeCase,
            directive_style: CaseStyle::ScreamingSnakeCase,
        };

        let content_true1 = r#"LoopStart: ADD R1, R2, R3"#;
        let content_true2 = r#"MainFunction: AND R4, R5, R6"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"loopStart: ADD R1, R2, R3"#; // LowerCamelCase
        let content_false2 = r#"loop_start: AND R4, R5, R6"#; // SnakeCase
        let content_false3 = r#"LOOP_START: ADD R1, R2, R3"#; // ScreamingSnakeCase
        test_false(style, content_false1);
        test_false(style, content_false2);
        test_false(style, content_false3);
    }

    #[test]
    fn test_label_scream_snake_case() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::ScreamingSnakeCase,
            instruction_style: CaseStyle::ScreamingSnakeCase,
            directive_style: CaseStyle::ScreamingSnakeCase,
        };

        let content_true1 = r#"LOOP2: ADD R1, R2, R3"#;
        let content_true2 = r#"MAIN_FUNCTION0: AND R4, R5, R6"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"loopStart: ADD R1, R2, R3"#; // LowerCamelCase
        let content_false2 = r#"loop_start: AND R4, R5, R6"#; // SnakeCase
        let content_false3 = r#"LoopStart: ADD R1, R2, R3"#; // ScreamingSnakeCase
        test_false(style, content_false1);
        test_false(style, content_false2);
        test_false(style, content_false3);
    }

    #[test]
    fn test_label_snakecase() {
        let style = LintStyle {
            colon_after_label: true,
            label_style: CaseStyle::SnakeCase,
            instruction_style: CaseStyle::ScreamingSnakeCase,
            directive_style: CaseStyle::ScreamingSnakeCase,
        };

        let content_true1 = r#"loop_start: ADD R1, R2, R3"#;
        let content_true2 = r#"main_function: AND R4, R5, R6"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"LoopStart: ADD R1, R2, R3"#; // UpperCamelCase
        let content_false2 = r#"loopStart: AND R4, R5, R6"#; // LowerCamelCase
        let content_false3 = r#"LOOP_START: ADD R4, R5, R6"#;

        test_false(style, content_false1);
        test_false(style, content_false2);
        test_false(style, content_false3);
    }

    #[test]
    fn test_label_colon() {
        let style = LintStyle {
            colon_after_label: false,
            label_style: CaseStyle::SnakeCase,
            instruction_style: CaseStyle::ScreamingSnakeCase,
            directive_style: CaseStyle::ScreamingSnakeCase,
        };

        let content_true1 = r#"loop_start ADD R1, R2, R3"#;
        let content_true2 = r#"main_function AND R4, R5, R6"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"LoopStart: ADD R1, R2, R3"#; // UpperCamelCase
        let content_false2 = r#"loopStart: AND R4, R5, R6"#; // LowerCamelCase
        let content_false3 = r#"LOOP_START: ADD R4, R5, R6"#;

        test_false(style, content_false1);
        test_false(style, content_false2);
        test_false(style, content_false3);
    }

    #[test]
    fn test_comments() {
        let style = LintStyle {
            colon_after_label: false,
            label_style: CaseStyle::SnakeCase,
            instruction_style: CaseStyle::ScreamingSnakeCase,
            directive_style: CaseStyle::ScreamingSnakeCase,
        };

        let content_true1 = r#"loop_start ADD R1, R2, R3 ; sdasd"#;
        let content_true2 = r#"main_function AND R4, R5, R6 ; asdsa"#;
        test_true(style, content_true1);
        test_true(style, content_true2);

        // Negation assertions: should fail for incorrect styles
        let content_false1 = r#"
        ;dasdas
        LoopStart: ADD R1, R2, R3"#; // UpperCamelCase
        let content_false2 = r#"loopStart: AND R4, R5, R6"#; // LowerCamelCase
        let content_false3 = r#"LOOP_START: ADD R4, R5, R6"#;

        test_false(style, content_false1);
        test_false(style, content_false2);
        test_false(style, content_false3);
    }
}
