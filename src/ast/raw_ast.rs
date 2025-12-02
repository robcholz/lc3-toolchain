use crate::ast::parse;
use either::Either;
use getset::Getters;
use parse::Rule;
use pest::iterators::Pair;

#[derive(Debug)]
pub struct Program {
    items: Vec<ProgramItem>,
}

impl Program {
    pub fn items(self) -> Vec<ProgramItem> {
        self.items
    }
}

#[derive(Debug, Clone, Getters)]
pub struct Span {
    #[get = "pub"]
    start: usize,
    #[get = "pub"]
    end: usize,
}

#[derive(Debug)]
pub enum ProgramItem {
    Comment(Comment),
    Label(Label),
    Instruction(Instruction),
    Directive(Directive),
}

#[derive(Debug, Clone, Getters)]
pub struct Comment {
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
}

#[derive(Debug, Clone, Getters)]
pub struct Label {
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterType {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

#[derive(Debug, Clone, Getters)]
pub struct Register {
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
    #[get = "pub"]
    register_type: RegisterType,
}

#[derive(Debug, Clone, Getters)]
pub struct LabelReference {
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
}

#[derive(Debug, Copy, Clone)]
pub enum BrType {
    N,
    Z,
    P,
    Nz,
    Zp,
    Np,
    Nzp,
    None,
}

#[derive(Debug, Clone)]
pub enum InstructionType {
    // arithmetic
    Add(Register, Register, Either<Register, Immediate>),
    And(Register, Register, Either<Register, Immediate>),
    Not(Register, Register),
    // load and store
    Ld(Register, LabelReference),
    Ldi(Register, LabelReference),
    Ldr(Register, Register, Immediate),
    Lea(Register, LabelReference),
    St(Register, LabelReference),
    Sti(Register, LabelReference),
    Str(Register, Register, Immediate),
    // branch
    Br(BrType, LabelReference),
    Jmp(Register),
    Jsr(LabelReference),
    Jsrr(Register),
    // control
    Nop,
    Ret,
    Halt,
    // io
    Puts,
    Getc,
    Out,
    In,
    // trap
    Trap(HexAddress),
}

#[derive(Debug, Clone, Getters)]
pub struct Instruction {
    #[get = "pub"]
    instruction_type: InstructionType,
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
}

#[derive(Debug, Clone, Getters)]
pub struct StringLiteral {
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
}

#[derive(Debug, Clone, Getters)]
pub struct Immediate {
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
}

#[derive(Debug, Clone, Getters)]
pub struct HexAddress {
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
}

#[derive(Debug, Clone)]
pub enum DirectiveType {
    ORIG(HexAddress),
    END,
    BLKW(Immediate),
    FILL(Immediate),
    STRINGZ(StringLiteral),
}

#[derive(Debug, Clone, Getters)]
pub struct Directive {
    #[get = "pub"]
    directive_type: DirectiveType,
    #[get = "pub"]
    content: String,
    #[get = "pub"]
    span: Span,
}

pub fn parse_ast(pair: Pair<Rule>) -> Program {
    if pair.as_rule() != Rule::Program {
        unreachable!();
    }
    Program {
        items: pair
            .into_inner()
            .map_while(|p| parse_program_item(p))
            .collect(),
    }
}

fn parse_program_item(pair: Pair<Rule>) -> Option<ProgramItem> {
    match pair.as_rule() {
        Rule::Comment => Some(ProgramItem::Comment(parse_comment(pair))),
        Rule::Label => Some(ProgramItem::Label(parse_label(pair))),
        Rule::Instruction => Some(ProgramItem::Instruction(parse_instruction(pair))),
        Rule::Directive => Some(ProgramItem::Directive(parse_directive(pair))),
        Rule::EOI => None,
        _ => {
            unreachable!();
        }
    }
}

fn parse_comment(pair: Pair<Rule>) -> Comment {
    Comment {
        content: pair.as_str().to_string(),
        span: Span::from(pair.as_span()),
    }
}

fn parse_label(pair: Pair<Rule>) -> Label {
    Label {
        content: pair.as_str().to_owned(),
        span: Span::from(pair.as_span()),
    }
}

fn parse_instruction(pair: Pair<Rule>) -> Instruction {
    assert_eq!(pair.as_rule(), Rule::Instruction);
    let mut inner = pair.into_inner();
    let instruction = inner.next();
    assert!(instruction.is_some());
    let instruction = instruction.unwrap();
    let mut instruction_line = instruction.clone().into_inner();
    let true_instruction = instruction_line.next();
    assert!(true_instruction.is_some());
    let true_instruction = true_instruction.unwrap();
    Instruction {
        instruction_type: match true_instruction.as_rule() {
            Rule::AddInstruction => {
                let register1 = instruction_line.next();
                let register2 = instruction_line.next();
                let register_or_immediate = instruction_line.next();
                assert!(
                    register1.is_some() && register2.is_some() && register_or_immediate.is_some()
                );
                let register1 = register1.unwrap();
                let register2 = register2.unwrap();
                let register_or_immediate = register_or_immediate.unwrap();
                InstructionType::Add(
                    parse_register(register1),
                    parse_register(register2),
                    parse_register_immediate(register_or_immediate),
                )
            }
            Rule::AndInstruction => {
                let register1 = instruction_line.next();
                let register2 = instruction_line.next();
                let register_or_immediate = instruction_line.next();
                assert!(
                    register1.is_some() && register2.is_some() && register_or_immediate.is_some()
                );
                let register1 = register1.unwrap();
                let register2 = register2.unwrap();
                let register_or_immediate = register_or_immediate.unwrap();
                InstructionType::And(
                    parse_register(register1),
                    parse_register(register2),
                    parse_register_immediate(register_or_immediate),
                )
            }
            Rule::NotInstruction => {
                let register1 = instruction_line.next();
                let register2 = instruction_line.next();
                assert!(register1.is_some() && register2.is_some());
                let register1 = register1.unwrap();
                let register2 = register2.unwrap();
                InstructionType::Not(parse_register(register1), parse_register(register2))
            }
            Rule::LdInstruction => {
                let register = instruction_line.next();
                let label_reference = instruction_line.next();
                assert!(register.is_some() && label_reference.is_some());
                let register = register.unwrap();
                let label_reference = label_reference.unwrap();
                InstructionType::Ld(
                    parse_register(register),
                    parse_label_reference(label_reference),
                )
            }
            Rule::LdiInstruction => {
                let register = instruction_line.next();
                let label_reference = instruction_line.next();
                assert!(register.is_some() && label_reference.is_some());
                let register = register.unwrap();
                let label_reference = label_reference.unwrap();
                InstructionType::Ldi(
                    parse_register(register),
                    parse_label_reference(label_reference),
                )
            }
            Rule::LdrInstruction => {
                let register1 = instruction_line.next();
                let register2 = instruction_line.next();
                let immediate = instruction_line.next();
                assert!(register1.is_some() && register2.is_some() && immediate.is_some());
                let register1 = register1.unwrap();
                let register2 = register2.unwrap();
                let immediate = immediate.unwrap();
                InstructionType::Ldr(
                    parse_register(register1),
                    parse_register(register2),
                    parse_immediate(immediate),
                )
            }
            Rule::LeaInstruction => {
                let register = instruction_line.next();
                let label_reference = instruction_line.next();
                assert!(register.is_some() && label_reference.is_some());
                let register = register.unwrap();
                let label_reference = label_reference.unwrap();
                InstructionType::Lea(
                    parse_register(register),
                    parse_label_reference(label_reference),
                )
            }
            Rule::StInstruction => {
                let register = instruction_line.next();
                let label_reference = instruction_line.next();
                assert!(register.is_some() && label_reference.is_some());
                let register = register.unwrap();
                let label_reference = label_reference.unwrap();
                InstructionType::St(
                    parse_register(register),
                    parse_label_reference(label_reference),
                )
            }
            Rule::StiInstruction => {
                let register = instruction_line.next();
                let label_reference = instruction_line.next();
                assert!(register.is_some() && label_reference.is_some());
                let register = register.unwrap();
                let label_reference = label_reference.unwrap();
                InstructionType::Sti(
                    parse_register(register),
                    parse_label_reference(label_reference),
                )
            }
            Rule::StrInstruction => {
                let register1 = instruction_line.next();
                let register2 = instruction_line.next();
                let immediate = instruction_line.next();
                assert!(register1.is_some() && register2.is_some() && immediate.is_some());
                let register1 = register1.unwrap();
                let register2 = register2.unwrap();
                let immediate = immediate.unwrap();
                InstructionType::Str(
                    parse_register(register1),
                    parse_register(register2),
                    parse_immediate(immediate),
                )
            }
            Rule::BrInstruction => {
                let instruction_type = instruction;
                assert_eq!(instruction_type.as_rule(), Rule::Br);
                let instruction = instruction_type.as_str().split_whitespace().next();
                assert!(instruction.is_some());
                let instruction = instruction.unwrap().to_uppercase();
                let instruction_tp = instruction.strip_prefix("BR");
                assert!(instruction_tp.is_some());
                let instruction_tp = instruction_tp.unwrap();

                let br_tp = if instruction_tp.contains('N')
                    && instruction_tp.contains('Z')
                    && instruction_tp.contains('P')
                {
                    BrType::Nzp
                } else if instruction_tp.contains('N') && instruction_tp.contains('Z') {
                    BrType::Nz
                } else if instruction_tp.contains('N') && instruction_tp.contains('P') {
                    BrType::Np
                } else if instruction_tp.contains('Z') && instruction_tp.contains('P') {
                    BrType::Zp
                } else if instruction_tp.contains('N') {
                    BrType::N
                } else if instruction_tp.contains('P') {
                    BrType::P
                } else if instruction_tp.contains('Z') {
                    BrType::Z
                } else if instruction_tp.is_empty() {
                    BrType::None
                } else {
                    unreachable!()
                };

                let label_reference = instruction_line.next();
                assert!(label_reference.is_some());
                let label_reference = label_reference.unwrap();

                InstructionType::Br(br_tp, parse_label_reference(label_reference))
            }
            Rule::JmpInstruction => {
                let register1 = instruction_line.next();
                assert!(register1.is_some());
                let register1 = register1.unwrap();
                InstructionType::Jmp(parse_register(register1))
            }
            Rule::JsrInstruction => {
                let label_ref = instruction_line.next();
                assert!(label_ref.is_some());
                let label_ref = label_ref.unwrap();
                InstructionType::Jsr(parse_label_reference(label_ref))
            }
            Rule::JsrrInstruction => {
                let register1 = instruction_line.next();
                assert!(register1.is_some());
                let register1 = register1.unwrap();
                InstructionType::Jsrr(parse_register(register1))
            }
            Rule::NopInstruction => InstructionType::Nop,
            Rule::RetInstruction => InstructionType::Ret,
            Rule::HaltInstruction => InstructionType::Halt,
            Rule::PutsInstruction => InstructionType::Puts,
            Rule::GetcInstruction => InstructionType::Getc,
            Rule::OutInstruction => InstructionType::Out,
            Rule::InInstruction => InstructionType::In,
            Rule::TrapInstruction => {
                let hex_address = instruction_line.next();
                assert!(hex_address.is_some());
                let hex_address = hex_address.unwrap();
                InstructionType::Trap(parse_hex_address(hex_address))
            }
            _ => {
                unreachable!()
            }
        },
        content: true_instruction.as_str().to_owned(),
        span: Span::from(true_instruction.as_span()),
    }
}

fn parse_directive(pair: Pair<Rule>) -> Directive {
    assert_eq!(pair.as_rule(), Rule::Directive);
    let mut inner = pair.into_inner();
    let directive_line = inner.next();
    assert!(directive_line.is_some());
    let mut directive_line = directive_line.unwrap().into_inner();
    let directive = directive_line.next();
    assert!(directive.is_some());
    let directive = directive.unwrap();
    Directive {
        directive_type: match directive.as_rule() {
            Rule::StringzDirective => {
                let string = directive_line.next();
                assert!(string.is_some());
                DirectiveType::STRINGZ(parse_string_literal(string.unwrap()))
            }
            Rule::FillDirective => {
                let immediate = directive_line.next();
                assert!(immediate.is_some());
                DirectiveType::FILL(parse_immediate(immediate.unwrap()))
            }
            Rule::OrigDirective => {
                let address = directive_line.next();
                assert!(address.is_some());
                DirectiveType::ORIG(parse_hex_address(address.unwrap()))
            }
            Rule::EndDirective => DirectiveType::END,
            Rule::BlkwDirective => {
                let immediate = directive_line.next();
                assert!(immediate.is_some());
                DirectiveType::BLKW(parse_immediate(immediate.unwrap()))
            }
            _ => {
                unreachable!()
            }
        },
        content: directive.as_span().as_str().to_owned(),
        span: Span::from(directive.as_span()),
    }
}

fn parse_string_literal(pair: Pair<Rule>) -> StringLiteral {
    assert_eq!(pair.as_rule(), Rule::StringLiteral);
    StringLiteral {
        content: pair.as_str().to_owned(),
        span: Span::from(pair.as_span()),
    }
}

fn parse_immediate(pair: Pair<Rule>) -> Immediate {
    assert_eq!(pair.as_rule(), Rule::Immediate);
    Immediate {
        content: pair.as_str().to_owned(),
        span: Span::from(pair.as_span()),
    }
}

fn parse_hex_address(pair: Pair<Rule>) -> HexAddress {
    assert_eq!(pair.as_rule(), Rule::HexAddress);
    HexAddress {
        content: pair.as_str().to_owned(),
        span: Span::from(pair.as_span()),
    }
}

fn parse_register(pair: Pair<Rule>) -> Register {
    assert_eq!(pair.as_rule(), Rule::Register);
    Register {
        content: pair.as_str().to_owned(),
        span: Span::from(pair.as_span()),
        register_type: match pair.as_str().to_uppercase().as_str() {
            "R0" => RegisterType::R0,
            "R1" => RegisterType::R1,
            "R2" => RegisterType::R2,
            "R3" => RegisterType::R3,
            "R4" => RegisterType::R4,
            "R5" => RegisterType::R5,
            "R6" => RegisterType::R6,
            "R7" => RegisterType::R7,
            _ => {
                unreachable!()
            }
        },
    }
}

fn parse_register_immediate(pair: Pair<Rule>) -> Either<Register, Immediate> {
    if pair.as_rule() == Rule::Register {
        return Either::Left(parse_register(pair));
    }
    Either::Right(parse_immediate(pair))
}

fn parse_label_reference(pair: Pair<Rule>) -> LabelReference {
    assert_eq!(pair.as_rule(), Rule::LabelReference);
    LabelReference {
        content: pair.as_str().to_owned(),
        span: Span::from(pair.as_span()),
    }
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span) -> Self {
        Span {
            start: value.start(),
            end: value.end(),
        }
    }
}
