#![allow(dead_code)]
use crate::raw_ast::{Comment, Directive, Instruction, Label, Span};
use getset::Getters;
use itertools::Itertools;
use pest::Stack;
use std::collections::HashMap;

#[derive(Debug, Getters)]
pub struct FormatterProgram {
    #[get = "pub"]
    items: Vec<FormatterProgramItem>,
}

#[derive(Debug)]
pub struct StandardTransform<'a> {
    label_buffer: Stack<Label>,
    forward_next_comment: bool,
    look_table: LineColumnLookTable<'a>,
    hybrid_inline_comment: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct LineColumn {
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
enum ProgramItem {
    Comment(Comment),
    Instruction(Vec<Label>, Instruction, Option<Comment>),
    Directive(Vec<Label>, Directive, Option<Comment>),
    EOL(Vec<Label>),
}

#[derive(Debug, Clone)]
pub enum FormatterProgramItem {
    Comment(Comment, LineColumn),
    Instruction(Vec<Label>, Instruction, Option<Comment>, LineColumn),
    Directive(Vec<Label>, Directive, Option<Comment>, LineColumn),
    EOL(Vec<Label>),
}

impl<'a> StandardTransform<'a> {
    pub fn new(hybrid_inline_comment: bool, file_content: &'a str) -> Self {
        Self {
            label_buffer: Stack::new(),
            forward_next_comment: true,
            look_table: LineColumnLookTable::new(file_content),
            hybrid_inline_comment,
        }
    }

    pub fn transform(&mut self, program: crate::raw_ast::Program) -> FormatterProgram {
        let mut labelled_items: Vec<_> = program
            .items()
            .into_iter()
            .map(|p| self.hybrid_label(p))
            .collect();
        if !self.label_buffer.is_empty() {
            let mut labels = vec![];
            while let Some(item) = self.label_buffer.pop() {
                labels.push(item);
            }
            labelled_items.push(Some(ProgramItem::EOL(labels)));
        }
        let labelled_items: Vec<_> = labelled_items
            .into_iter()
            .filter_map(|p| p)
            .map(|p| self.add_line_info(p))
            .collect();
        FormatterProgram {
            items: if self.hybrid_inline_comment {
                labelled_items
                    .into_iter()
                    .tuple_windows()
                    .map(|(prev, curr)| self.hybrid_comment(prev, curr))
                    .filter_map(|p| p)
                    .collect()
            } else {
                labelled_items
            },
        }
    }

    fn hybrid_label(&mut self, program_item: crate::raw_ast::ProgramItem) -> Option<ProgramItem> {
        match program_item {
            crate::raw_ast::ProgramItem::Label(label) => {
                self.label_buffer.push(label);
                None
            }
            crate::raw_ast::ProgramItem::Instruction(instruction) => {
                let mut labels = vec![];
                while let Some(item) = self.label_buffer.pop() {
                    labels.push(item);
                }
                Some(ProgramItem::Instruction(labels, instruction, None))
            }
            crate::raw_ast::ProgramItem::Directive(directive) => {
                let mut labels = vec![];
                while let Some(item) = self.label_buffer.pop() {
                    labels.push(item);
                }
                Some(ProgramItem::Directive(labels, directive, None))
            }
            crate::raw_ast::ProgramItem::Comment(comment) => Some(ProgramItem::Comment(comment)),
        }
    }

    fn add_line_info(&mut self, program_item: ProgramItem) -> FormatterProgramItem {
        match program_item {
            ProgramItem::Comment(comment) => {
                let lc = self.look_table.get_line_and_column(comment.span());
                FormatterProgramItem::Comment(comment, lc)
            }
            ProgramItem::Instruction(label, instruction, comment) => {
                let lc = self.look_table.get_line_and_column(instruction.span());
                FormatterProgramItem::Instruction(label, instruction, comment, lc)
            }
            ProgramItem::Directive(label, directive, comment) => {
                let lc = self.look_table.get_line_and_column(directive.span());
                FormatterProgramItem::Directive(label, directive, comment, lc)
            }
            ProgramItem::EOL(label) => FormatterProgramItem::EOL(label),
        }
    }

    fn hybrid_comment(
        &mut self,
        curr: FormatterProgramItem,
        next: FormatterProgramItem,
    ) -> Option<FormatterProgramItem> {
        match curr {
            FormatterProgramItem::Comment(comment, comment_lc) => {
                if self.forward_next_comment {
                    Some(FormatterProgramItem::Comment(comment, comment_lc))
                } else {
                    self.forward_next_comment = true;
                    None
                }
            }
            FormatterProgramItem::Instruction(labels, instruction, comment, lc) => {
                if let FormatterProgramItem::Comment(comment, lc_comment) = next {
                    if lc_comment.at_the_same_line(&lc) {
                        self.forward_next_comment = false;
                        return Some(FormatterProgramItem::Instruction(
                            labels,
                            instruction,
                            Some(comment),
                            lc,
                        ));
                    }
                }
                Some(FormatterProgramItem::Instruction(
                    labels,
                    instruction,
                    comment,
                    lc,
                ))
            }
            FormatterProgramItem::Directive(labels, directive, comment, lc) => {
                if let FormatterProgramItem::Comment(comment, lc_comment) = next {
                    if lc_comment.at_the_same_line(&lc) {
                        self.forward_next_comment = false;
                        return Some(FormatterProgramItem::Directive(
                            labels,
                            directive,
                            Some(comment),
                            lc,
                        ));
                    }
                }
                Some(FormatterProgramItem::Directive(
                    labels, directive, comment, lc,
                ))
            }
            FormatterProgramItem::EOL(labels) => Some(FormatterProgramItem::EOL(labels)),
        }
    }
}

#[derive(Debug)]
struct LineColumnLookTable<'a> {
    line_start_indices: HashMap<usize, (usize, usize)>, // Key: start index, Value: (line number, column start)
    lines: Vec<&'a str>,
}

impl<'a> LineColumnLookTable<'a> {
    // Build a new lookup table based on the file content
    pub fn new(file_content: &'a str) -> Self {
        let mut line_start_indices = HashMap::new();
        let mut char_count = 0; // Keeps track of the starting character index of the line
        let mut line_number = 1; // Line numbers are 1-based

        for line in file_content.lines() {
            // Insert the line start index and its corresponding line number and column start
            line_start_indices.insert(char_count, (line_number, 1));
            char_count += line.len() + 1; // Increment by the length of the line + 1 for newline character
            line_number += 1;
        }
        let lines: Vec<&str> = file_content.lines().collect();

        LineColumnLookTable {
            line_start_indices,
            lines,
        }
    }

    // Function to get the line and column for a given span
    pub fn get_line_and_column(&self, span: &Span) -> LineColumn {
        let start = *span.start();
        // Find the line where the span starts
        for (start_index, (line_number, _)) in self.line_start_indices.iter() {
            let line_len = self.lines[*line_number - 1].len();
            if start >= *start_index && start < *start_index + line_len {
                // Find the column number
                let column = start - *start_index + 1;
                return LineColumn {
                    line: *line_number,
                    column,
                };
            }
        }

        unreachable!()
    }
}

impl LineColumn {
    pub fn at_the_same_line(&self, other: &LineColumn) -> bool {
        self.line == other.line
    }
}

impl FormatterProgramItem {
    pub fn is_comment(&self) -> bool {
        matches!(self, FormatterProgramItem::Comment(..))
    }

    pub fn is_instruction(&self) -> bool {
        matches!(self, FormatterProgramItem::Instruction(..))
    }

    pub fn is_directive(&self) -> bool {
        matches!(self, FormatterProgramItem::Directive(..))
    }

    pub fn is_eol(&self) -> bool {
        matches!(self, FormatterProgramItem::EOL(..))
    }
}
