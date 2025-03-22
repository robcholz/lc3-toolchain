use crate::ast::raw_ast::{Comment, Directive, Instruction, Label, Span};
use getset::Getters;
use pest::Stack;
use std::collections::HashMap;

#[derive(Debug, Getters)]
pub struct Program {
    #[get = "pub"]
    items: Vec<ProgramItem>,
}

#[derive(Debug)]
pub struct StandardTransform<'a> {
    label_buffer: Stack<Label>,
    forward_next_comment: bool,
    look_table: LineColumnLookTable<'a>,
    hybrid_inline_comment: bool,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct LineColumn {
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
enum RawProgramItem {
    Comment(Comment),
    Instruction(Vec<Label>, Instruction, Option<Comment>),
    Directive(Vec<Label>, Directive, Option<Comment>),
    EOL(Vec<Label>),
}

#[derive(Debug, Clone)]
pub enum ProgramItem {
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

    pub fn transform(&mut self, program: crate::ast::raw_ast::Program) -> Program {
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
            labelled_items.push(Some(RawProgramItem::EOL(labels)));
        }
        let labelled_items: Vec<_> = labelled_items
            .into_iter()
            .filter_map(|p| p)
            .map(|p| self.add_line_info(p))
            .collect();
        Program {
            items: if self.hybrid_inline_comment {
                let mut res = vec![];
                for (index, current) in labelled_items.iter().enumerate() {
                    let next = labelled_items.get(index + 1);
                    res.push(self.hybrid_comment(current.clone(), next.map(|i| i.clone())));
                }
                res.into_iter().filter_map(|i| i).collect()
            } else {
                labelled_items
            },
        }
    }

    fn hybrid_label(
        &mut self,
        program_item: crate::ast::raw_ast::ProgramItem,
    ) -> Option<RawProgramItem> {
        match program_item {
            crate::ast::raw_ast::ProgramItem::Label(label) => {
                self.label_buffer.push(label);
                None
            }
            crate::ast::raw_ast::ProgramItem::Instruction(instruction) => {
                let mut labels = vec![];
                while let Some(item) = self.label_buffer.pop() {
                    labels.push(item);
                }
                Some(RawProgramItem::Instruction(labels, instruction, None))
            }
            crate::ast::raw_ast::ProgramItem::Directive(directive) => {
                let mut labels = vec![];
                while let Some(item) = self.label_buffer.pop() {
                    labels.push(item);
                }
                Some(RawProgramItem::Directive(labels, directive, None))
            }
            crate::ast::raw_ast::ProgramItem::Comment(comment) => {
                Some(RawProgramItem::Comment(comment))
            }
        }
    }

    fn add_line_info(&mut self, program_item: RawProgramItem) -> ProgramItem {
        match program_item {
            RawProgramItem::Comment(comment) => {
                let lc = self.look_table.get_line_and_column(comment.span());
                ProgramItem::Comment(comment, lc)
            }
            RawProgramItem::Instruction(label, instruction, comment) => {
                let lc = self.look_table.get_line_and_column(instruction.span());
                ProgramItem::Instruction(label, instruction, comment, lc)
            }
            RawProgramItem::Directive(label, directive, comment) => {
                let lc = self.look_table.get_line_and_column(directive.span());
                ProgramItem::Directive(label, directive, comment, lc)
            }
            RawProgramItem::EOL(label) => ProgramItem::EOL(label),
        }
    }

    fn hybrid_comment(
        &mut self,
        curr: ProgramItem,
        next: Option<ProgramItem>,
    ) -> Option<ProgramItem> {
        match curr {
            ProgramItem::Comment(comment, comment_lc) => {
                if self.forward_next_comment {
                    Some(ProgramItem::Comment(comment, comment_lc))
                } else {
                    self.forward_next_comment = true;
                    None
                }
            }
            ProgramItem::Instruction(labels, instruction, comment, lc) => {
                if let Some(ProgramItem::Comment(comment, lc_comment)) = next {
                    if lc_comment.at_the_same_line(&lc) {
                        self.forward_next_comment = false;
                        return Some(ProgramItem::Instruction(
                            labels,
                            instruction,
                            Some(comment),
                            lc,
                        ));
                    }
                }
                Some(ProgramItem::Instruction(labels, instruction, comment, lc))
            }
            ProgramItem::Directive(labels, directive, comment, lc) => {
                if let Some(ProgramItem::Comment(comment, lc_comment)) = next {
                    if lc_comment.at_the_same_line(&lc) {
                        self.forward_next_comment = false;
                        return Some(ProgramItem::Directive(labels, directive, Some(comment), lc));
                    }
                }
                Some(ProgramItem::Directive(labels, directive, comment, lc))
            }
            ProgramItem::EOL(labels) => Some(ProgramItem::EOL(labels)),
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

impl ProgramItem {
    pub fn is_comment(&self) -> bool {
        matches!(self, ProgramItem::Comment(..))
    }

    pub fn is_instruction(&self) -> bool {
        matches!(self, ProgramItem::Instruction(..))
    }

    pub fn is_directive(&self) -> bool {
        matches!(self, ProgramItem::Directive(..))
    }

    pub fn is_eol(&self) -> bool {
        matches!(self, ProgramItem::EOL(..))
    }
}
