use crate::ast::processed_ast::{FormatterProgram, FormatterProgramItem};
use crate::ast::raw_ast::{
    Comment, Directive, DirectiveType, Immediate, Instruction, InstructionType, Label, Register,
};
use either::Either;
use serde::{Deserialize, Serialize};

trait FormattedDisplay {
    // label body comment
    fn formatted_display(&self, style: &FormatStyle) -> (Vec<String>, String, Option<String>);
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FormatStyle {
    pub indent_directive: u8,              // horizontal // done
    pub indent_instruction: u8,            // horizontal // done
    pub indent_label: u8,                  // horizontal // done
    pub indent_min_comment_from_block: u8, // horizontal // done
    pub space_block_to_comment: u8,        // vertical //done
    pub space_comment_stick_to_body: u8,   // vertical //done
    pub space_from_label_block: u8,        // vertical //done
    pub space_from_start_end_block: u8,    // vertical  // done
    pub colon_after_label: bool,
}

pub struct Formatter<'a> {
    style: &'a FormatStyle,
    buffer: Vec<u8>,
}

impl<'a> Formatter<'a> {
    pub fn new(style: &'a FormatStyle) -> Self {
        Self {
            style,
            buffer: Vec::new(),
        }
    }

    pub fn format(&mut self, program: FormatterProgram) {
        self.buffer.reserve(program.items().len() * 10);
        let mut lines: Vec<(Vec<String>, String, Option<String>, usize)> = vec![];
        for (index, line) in program.items().iter().enumerate() {
            let (labels, body, comments) = line.formatted_display(&self.style);
            lines.push((
                labels,
                body,
                comments,
                self.control_padding(line, program.items().get(index + 1)) + 1,
            ));
        }

        let comment_start_column = lines.iter().map(|e| e.1.len()).max().unwrap_or(0)
            + (self.style.indent_min_comment_from_block as usize);

        for (labels, body, comment, space) in lines.into_iter() {
            let missing_indent = comment_start_column - body.len();
            let mut label = "".to_owned();
            labels
                .into_iter()
                .map(|mut l| {
                    l.push('\n');
                    l
                })
                .for_each(|e| label.push_str(e.as_str()));
            self.buffer.append(&mut label.into_bytes());
            self.buffer.append(&mut body.into_bytes());
            self.add_indent(missing_indent);
            match comment {
                None => {}
                Some(comment) => {
                    self.buffer.append(&mut comment.into_bytes());
                }
            }
            self.add_newline(space);
        }
    }

    pub fn contents(&self) -> &Vec<u8> {
        &self.buffer
    }

    #[inline]
    fn add_newline(&mut self, lines: usize) {
        for _ in 0..lines {
            self.buffer.push(b'\n');
        }
    }

    #[inline]
    fn add_indent(&mut self, indent: usize) {
        for _ in 0..indent {
            self.buffer.push(b' ');
        }
    }

    fn control_padding(
        &mut self,
        current: &FormatterProgramItem,
        next: Option<&FormatterProgramItem>,
    ) -> usize {
        let mut paddings = 0usize;

        // space_comment_stick_to_body
        if self.style.space_comment_stick_to_body != 0 {
            if current.is_comment()
                && next.is_some()
                && (next.unwrap().is_directive() || next.unwrap().is_instruction())
            {
                paddings += self.style.space_comment_stick_to_body as usize;
            }
        }

        // space_block_between
        if self.style.space_block_to_comment != 0 {
            // solve conflict with padding_start_end_directive_block
            if let FormatterProgramItem::Directive(_, directive, ..) = current {
                if matches!(directive.directive_type(), DirectiveType::ORIG(..)) {
                    // balabala
                } else {
                    if (current.is_directive() || current.is_instruction())
                        && next.is_some()
                        && next.unwrap().is_comment()
                    {
                        paddings += self.style.space_block_to_comment as usize;
                    }
                }
            } else {
                if (current.is_directive() || current.is_instruction())
                    && next.is_some()
                    && next.unwrap().is_comment()
                {
                    paddings += self.style.space_block_to_comment as usize;
                }
            }
        }

        // space_from_label_block
        if self.style.space_from_label_block != 0 {
            let space: u8 = match current {
                FormatterProgramItem::Instruction(curr_label, ..)
                | FormatterProgramItem::Directive(curr_label, ..) => match next {
                    None => 0,
                    Some(next) => match next {
                        FormatterProgramItem::Instruction(next_label, ..)
                        | FormatterProgramItem::Directive(next_label, ..) => {
                            if curr_label.is_empty() && (!next_label.is_empty()) {
                                self.style.space_from_label_block
                            } else {
                                0
                            }
                        }
                        FormatterProgramItem::EOL(..) | FormatterProgramItem::Comment(..) => 0,
                    },
                },
                FormatterProgramItem::EOL(..) | FormatterProgramItem::Comment(..) => 0,
            };
            paddings += space as usize;
        }

        // padding_start_end_directive_block
        if self.style.space_from_start_end_block != 0 {
            let space: u8 = match current {
                FormatterProgramItem::Directive(_, directive, ..) => {
                    if matches!(directive.directive_type(), DirectiveType::ORIG(..)) {
                        self.style.space_from_start_end_block
                    } else if next.is_some() {
                        match next.unwrap() {
                            FormatterProgramItem::Directive(_, directive, _, _) => {
                                if matches!(directive.directive_type(), DirectiveType::END) {
                                    self.style.space_from_start_end_block
                                } else {
                                    0
                                }
                            }
                            _ => 0,
                        }
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            paddings += space as usize;
        }
        paddings
    }
}

impl FormattedDisplay for FormatterProgramItem {
    fn formatted_display(&self, style: &FormatStyle) -> (Vec<String>, String, Option<String>) {
        match self {
            FormatterProgramItem::Comment(comment, _) => (vec![], print_comment(comment), None),
            FormatterProgramItem::Instruction(labels, instruction, comment, _) => {
                let mut label_indent = "".to_owned();
                add_indent(
                    &mut label_indent,
                    labels.is_empty().then_some(0).unwrap_or(style.indent_label),
                );
                let labels = labels
                    .into_iter()
                    .map(|l| format!("{label_indent}{}", print_label(style, l)))
                    .collect();
                let mut instruction_indent = "".to_owned();
                add_indent(&mut instruction_indent, style.indent_instruction);
                let comment = comment.as_ref().map_or(None, |c| Some(print_comment(c)));
                (
                    labels,
                    format!("{instruction_indent}{}", print_instruction(instruction)),
                    comment,
                )
            }
            FormatterProgramItem::Directive(labels, directive, comment, _) => {
                let mut label_indent = "".to_owned();
                add_indent(
                    &mut label_indent,
                    labels.is_empty().then_some(0).unwrap_or(style.indent_label),
                );
                let labels = labels
                    .into_iter()
                    .map(|l| format!("{label_indent}{}", print_label(style, l)))
                    .collect();
                let mut directive_indent = "".to_owned();
                add_indent(
                    &mut directive_indent,
                    (matches!(directive.directive_type(), DirectiveType::END)
                        || matches!(directive.directive_type(), DirectiveType::ORIG(..)))
                    .then_some(0)
                    .unwrap_or(style.indent_directive),
                );
                let comment = comment.as_ref().map_or(None, |c| Some(print_comment(c)));
                (
                    labels,
                    format!("{directive_indent}{}", print_directive(directive)),
                    comment,
                )
            }
            FormatterProgramItem::EOL(labels) => {
                let mut label_indent = "".to_owned();
                add_indent(
                    &mut label_indent,
                    labels.is_empty().then_some(0).unwrap_or(style.indent_label),
                );
                let labels = labels
                    .into_iter()
                    .map(|l| format!("{label_indent}{}", print_label(style, l)))
                    .collect();
                (labels, "".to_owned(), None)
            }
        }
    }
}

fn print_instruction(instruction: &Instruction) -> String {
    let operands: String = match instruction.instruction_type() {
        InstructionType::Add(register1, register2, register_or_immediate)
        | InstructionType::And(register1, register2, register_or_immediate) => {
            format!(
                "{}, {}, {}",
                register1.content(),
                register2.content(),
                print_register_or_immediate(register_or_immediate)
            )
        }
        InstructionType::Not(register1, register2) => {
            format!("{}, {}", register1.content(), register2.content(),)
        }
        InstructionType::Ldr(register1, register2, immediate)
        | InstructionType::Str(register1, register2, immediate) => {
            format!(
                "{}, {}, {}",
                register1.content(),
                register2.content(),
                immediate.content()
            )
        }
        InstructionType::Ld(register1, label_ref)
        | InstructionType::Ldi(register1, label_ref)
        | InstructionType::Lea(register1, label_ref)
        | InstructionType::St(register1, label_ref)
        | InstructionType::Sti(register1, label_ref) => {
            format!("{}, {}", register1.content(), label_ref.content())
        }
        InstructionType::Br(_, label_ref) => label_ref.content().to_owned(),
        InstructionType::Jmp(register) | InstructionType::Jsrr(register) => {
            register.content().to_owned()
        }
        InstructionType::Jsr(label_ref) => label_ref.content().to_owned(),
        InstructionType::Nop
        | InstructionType::Ret
        | InstructionType::Halt
        | InstructionType::Puts
        | InstructionType::Getc
        | InstructionType::Out
        | InstructionType::In => "".to_owned(),
        InstructionType::Trap(hex_address) => hex_address.content().to_owned(),
    };
    if operands.is_empty() {
        format!("{}", instruction.content())
    } else {
        format!("{} {}", instruction.content(), operands)
    }
}

fn print_comment(comment: &Comment) -> String {
    match comment.content().strip_prefix(";") {
        None => {
            unreachable!()
        }
        Some(comment) => {
            let comment = comment.trim();
            format!(";{comment}")
        }
    }
}

fn print_label(style: &FormatStyle, label: &Label) -> String {
    if style.colon_after_label {
        match label.content().ends_with(":") {
            false => {
                if style.colon_after_label {
                    format!("{}:", label.content())
                } else {
                    label.content().into()
                }
            }
            true => label.content().to_owned(),
        }
    } else {
        match label.content().strip_suffix(":") {
            None => label.content().into(),
            Some(label) => label.to_owned(),
        }
    }
}

fn print_register_or_immediate(either: &Either<Register, Immediate>) -> String {
    match either {
        Either::Left(r) => r.content(),
        Either::Right(im) => im.content(),
    }
    .to_owned()
}

fn print_directive(directive: &Directive) -> String {
    let operands: String = match directive.directive_type() {
        DirectiveType::ORIG(address) => address.content(),
        DirectiveType::END => "",
        DirectiveType::BLKW(immediate) | DirectiveType::FILL(immediate) => immediate.content(),
        DirectiveType::STRINGZ(string) => string.content(),
    }
    .to_owned();
    if operands.is_empty() {
        format!("{}", directive.content())
    } else {
        format!("{} {}", directive.content(), operands)
    }
}

fn add_indent(string: &mut String, indent: u8) {
    for _ in 0..indent {
        string.push(' ');
    }
}
