use crate::token::Token;

use chumsky::prelude::*;
use chumsky::Stream;
use logos::Span;

pub type MachineAddress = u16;
pub type MachineCode = u64;

pub struct Ast<'a> {
    pub title: &'a str,
    pub instructions: Vec<Instruction<'a>>,
}

fn is_refered_by_iop(instructions: &[Instruction], label: &str) -> bool {
    instructions.iter().any(|instruction| {
        instruction.test_and_sequence_statement == Some(TestAndSequenceStatement::Iop(label))
    })
}

fn is_refered_by_ira(instructions: &[Instruction], label: &str) -> bool {
    instructions.iter().any(|instruction| {
        instruction.test_and_sequence_statement == Some(TestAndSequenceStatement::Ira(label))
    })
}

fn is_refered_by_iab(instructions: &[Instruction], label: &str) -> bool {
    instructions.iter().any(|instruction| {
        instruction.test_and_sequence_statement == Some(TestAndSequenceStatement::Iab(label))
    })
}

impl<'a> Ast<'a> {
    pub fn set_address(&'a self) -> Result<Self, ()> {
        let mut instructions = Vec::<Instruction>::new();

        let mut address = 0;
        for instruction in &self.instructions {
            address = if let Some(label) = instruction.label {
                if is_refered_by_iop(&self.instructions, label) {
                    if let Some(constant) = instruction.address {
                        if (constant & 0xF) == 0 {
                            constant
                        } else {
                            return Err(());
                        }
                    } else {
                        address += 0x10;
                        address & 0xFF0
                    }
                } else if is_refered_by_ira(&self.instructions, label) {
                    if let Some(constant) = instruction.address {
                        if (constant & 0xC) == 0 {
                            constant
                        } else {
                            return Err(());
                        }
                    } else {
                        address += 0x10;
                        address & 0xFF3
                    }
                } else if is_refered_by_iab(&self.instructions, label) {
                    if let Some(constant) = instruction.address {
                        if (constant & 0x3C) == 0 {
                            constant
                        } else {
                            return Err(());
                        }
                    } else {
                        address += 0x40;
                        address & 0xFC0
                    }
                } else if let Some(constant) = instruction.address {
                    constant
                } else {
                    address
                }
            } else if let Some(constant) = instruction.address {
                constant
            } else {
                address
            };

            // let mut instruction = self.instructions[index].clone();
            // instruction.address = Some(address);
            instructions.push(Instruction {
                address: Some(address),
                ..instruction.clone()
            });

            address += 1;
        }

        Ok(Ast {
            title: self.title,
            instructions,
        })
    }
}

#[derive(Clone)]
pub struct Instruction<'a> {
    pub label: Option<&'a str>,
    pub address: Option<MachineAddress>,
    pub test_and_sequence_statement: Option<TestAndSequenceStatement<'a>>,
    pub memory_statement: Option<MemoryStatement>,
    pub calculation_statement: Option<CalculationStatement>,
    pub ex_statement: Option<ExStatement>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum FlagState {
    Zero,
    One,
}

#[derive(Clone, Eq, PartialEq)]
pub enum TestAndSequenceStatement<'a> {
    Goto(&'a str),
    Call(&'a str),
    r#Return,
    r#If(Flag, FlagState, &'a str, bool),
    Iop(&'a str),
    Ira(&'a str),
    Iab(&'a str),
    Nsq,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Flag {
    Zer,
    Neg,
    Cry,
    Ov,
    Cz,
    T,
}

#[derive(Clone)]
pub enum MemoryStatement {
    Read,
    Write,
}

#[derive(Clone)]
pub enum CalculationStatement {
    Alu(Sbus, Statement),
    Set(Statement),
}

#[derive(Clone)]
pub enum Sbus {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    Ra,
    Rap,
    Rb,
    Rbp,
    Pc,
}

#[derive(Clone)]
pub enum Statement {
    First(Lbus, AluAndShift, Rbus),
    AluThrough(AluThrough),
}

#[derive(Clone)]
pub enum Lbus {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    Rb,
    Rbp,
    Pc,
    Io,
    Mm,
    Ir,
    Fsr,
    Zero,
}

#[derive(Clone)]
pub struct AluAndShift {
    pub alu: Alu,
    pub shift: Option<Shift>,
}

#[derive(Clone)]
pub enum Alu {
    Plus,
    Minus,
    And,
    Or,
    Xor,
    Dollar,
    At,
}

#[derive(Clone)]
pub enum Shift {
    Sll,
    Srl,
    Sla,
    Sra,
    Snx,
    Swp,
    Nsb,
}

#[derive(Clone)]
pub enum Rbus {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    Ra,
    Rap,
    Literal(u16),
}

#[derive(Clone)]
pub struct AluThrough {
    pub shift: Option<Shift>,
    pub lbus_or_rbus: LbusOrRbus,
}

#[derive(Clone)]
pub enum LbusOrRbus {
    Lbus(Lbus),
    Rbus(Rbus),
}

#[derive(Clone)]
pub enum ExStatement {
    DecrimentC,
    FlagSave,
    WithCry,
    WithOne,
    Ir(LbusSource),
    Io(LbusSource),
    C(RbusSource),
    ExecuteIo,
    Assign1ToT,
    Assign0ToT,
    IraPlus1,
    IrbPlus1,
    IrbMinus1,
    SetHlt,
    SetOv,
    Nex,
}

#[derive(Clone)]
pub enum LbusSource {
    Source(Lbus),
    Lbus,
}

#[derive(Clone)]
pub enum RbusSource {
    Source(Rbus),
    Rbus,
}

fn parser<'a>() -> impl Parser<Token<'a>, Ast<'a>, Error = Simple<Token<'a>>> {
    let string = select! { Token::String(s) => s };

    let label = string.then_ignore(just(Token::Colon)).or_not();
    let unsigned_integer = select! {
        Token::String(s) if u16::from_str_radix(s, 16).is_ok() => u16::from_str_radix(s, 16).unwrap(),
        Token::Hexadecimal(h) => h,
        Token::Decimal(d) => d,
        Token::Binary(b) => b,
    };
    let literal = unsigned_integer;
    let address = unsigned_integer;

    let instruction_head = just(Token::Star)
        .ignore_then(label)
        .then(address.or_not())
        .then_ignore(just(Token::Eol));

    // Test and sequence statement
    let goto = just(Token::String("GOTO"))
        .ignore_then(string)
        .map(TestAndSequenceStatement::Goto);
    let call = just(Token::String("CALL"))
        .ignore_then(string)
        .map(TestAndSequenceStatement::Call);
    let r#return = just(Token::String("RETURN")).map(|_| TestAndSequenceStatement::Return);
    let flag = select! {
        Token::String("ZER") => Flag::Zer,
        Token::String("NEG") => Flag::Neg,
        Token::String("CRY") => Flag::Cry,
        Token::String("OV") => Flag::Ov,
        Token::String("CZ") => Flag::Cz,
        Token::String("T") => Flag::T,
    };
    let r#if = just(Token::String("IF"))
        .ignore_then(flag)
        .then_ignore(just(Token::Equal))
        .then_ignore(just(Token::Hexadecimal(0)))
        .then_ignore(just(Token::String("THEN")))
        .then(string)
        .then_ignore(just(Token::String("ELSE")))
        .then_ignore(just(Token::String("FETCH")))
        .map(|(flag, name)| TestAndSequenceStatement::If(flag, FlagState::Zero, name, true))
        .or(just(Token::String("IF"))
            .ignore_then(flag)
            .then_ignore(just(Token::Equal))
            .then(just(Token::Hexadecimal(1)).or(just(Token::Hexadecimal(0))))
            .then_ignore(just(Token::String("THEN")))
            .then(string)
            .map(|((flag, flag_state), name)| {
                TestAndSequenceStatement::If(
                    flag,
                    if flag_state == Token::Hexadecimal(1) {
                        FlagState::One
                    } else {
                        FlagState::Zero
                    },
                    name,
                    false,
                )
            }));
    let iop = just(Token::String("IOP"))
        .ignore_then(string)
        .map(TestAndSequenceStatement::Iop);
    let ira = just(Token::String("IRA"))
        .ignore_then(string)
        .map(TestAndSequenceStatement::Ira);
    let irb = just(Token::String("IAB"))
        .ignore_then(string)
        .map(TestAndSequenceStatement::Iab);
    let nsq = just(Token::String("NSQ")).map(|_| TestAndSequenceStatement::Nsq);
    let test_and_sequence_statement = goto
        .or(call)
        .or(r#return)
        .or(r#if)
        .or(iop)
        .or(ira)
        .or(irb)
        .or(nsq)
        .then_ignore(just(Token::Eol))
        .or_not();

    // memory statement
    let memory_statement = choice((
        just(Token::String("READ")).to(MemoryStatement::Read),
        just(Token::String("WRITE")).to(MemoryStatement::Write),
    ))
    .or_not();

    // calculation statement
    let sbus = select! {
        Token::String("R0") => Sbus::R0,
        Token::String("R1") => Sbus::R1,
        Token::String("R2") => Sbus::R2,
        Token::String("R3") => Sbus::R3,
        Token::String("R4") => Sbus::R4,
        Token::String("R5") => Sbus::R5,
        Token::String("R6") => Sbus::R6,
        Token::String("R7") => Sbus::R7,
        Token::String("RA") => Sbus::Ra,
        Token::String("RAP") => Sbus::Rap,
        Token::String("RB") => Sbus::Rb,
        Token::String("RBP") => Sbus::Rbp,
        Token::String("PC") => Sbus::Pc,
    };
    let lbus = select! {
        Token::String("R0") => Lbus::R0,
        Token::String("R1") => Lbus::R1,
        Token::String("R2") => Lbus::R2,
        Token::String("R3") => Lbus::R3,
        Token::String("R4") => Lbus::R4,
        Token::String("R5") => Lbus::R5,
        Token::String("R6") => Lbus::R6,
        Token::String("R7") => Lbus::R7,
        Token::String("RB") => Lbus::Rb,
        Token::String("RBP") => Lbus::Rbp,
        Token::String("PC") => Lbus::Pc,
        Token::String("IO") => Lbus::Io,
        Token::String("MM") => Lbus::Mm,
        Token::String("IR") => Lbus::Ir,
        Token::String("FSR") => Lbus::Fsr,
        Token::String("ZERO") => Lbus::Zero,
    };
    let alu = select! {
        Token::Plus => Alu::Plus,
        Token::Minus => Alu::Minus,
        Token::String("AND") => Alu::And,
        Token::String("OR") => Alu::Or,
        Token::String("XOR") => Alu::Xor,
        Token::Dollar => Alu::Dollar,
        Token::At => Alu::At,
    };
    let shift = select! {
        Token::String("SLL") => Shift::Sll,
        Token::String("SRL") => Shift::Srl,
        Token::String("SLA") => Shift::Sla,
        Token::String("SRA") => Shift::Sra,
        Token::String("SNX") => Shift::Snx,
        Token::String("SWP") => Shift::Swp,
        Token::String("NSB") => Shift::Nsb,
    };
    let alu_and_shift = alu
        .then(just(Token::Colon).ignore_then(shift).or_not())
        .map(|(alu, shift)| AluAndShift { alu, shift });
    let rbus = select! {
        Token::String("R0") => Rbus::R0,
        Token::String("R1") => Rbus::R1,
        Token::String("R2") => Rbus::R2,
        Token::String("R3") => Rbus::R3,
        Token::String("R4") => Rbus::R4,
        Token::String("R5") => Rbus::R5,
        Token::String("R6") => Rbus::R6,
        Token::String("R7") => Rbus::R7,
        Token::String("RA") => Rbus::Ra,
        Token::String("RAP") => Rbus::Rap,
    }
    .or(literal.map(Rbus::Literal));
    let alu_through_statement = shift
        .or_not()
        .map(|shift| shift)
        .then(lbus.map(LbusOrRbus::Lbus).or(rbus.map(LbusOrRbus::Rbus)))
        .map(|(shift, lbus_or_rbus)| AluThrough {
            shift,
            lbus_or_rbus,
        });
    let statement = lbus
        .then(alu_and_shift)
        .then(rbus)
        .map(|((lbus, alu_and_shift), rbus)| Statement::First(lbus, alu_and_shift, rbus))
        .or(alu_through_statement.map(Statement::AluThrough));
    let alu_statement = sbus
        .then_ignore(just(Token::ColonEqual))
        .then(statement.clone())
        .map(|(sb, stmt)| CalculationStatement::Alu(sb, stmt));
    let set_statement = just(Token::String("SET"))
        .ignore_then(just(Token::String("BY")))
        .ignore_then(statement)
        .map(CalculationStatement::Set);
    let calculation_statement = alu_statement
        .or(set_statement)
        .then_ignore(just(Token::Eol))
        .or_not();

    // EX statement
    let lbus_source = lbus
        .map(LbusSource::Source)
        .or(just(Token::String("LBUS")).map(|_| LbusSource::Lbus));
    let rbus_source = rbus
        .map(RbusSource::Source)
        .or(just(Token::String("RBUS")).map(|_| RbusSource::Rbus));
    let ex_statement = just(Token::String("C"))
        .ignore_then(just(Token::Minus))
        .ignore_then(just(Token::Hexadecimal(1)))
        .to(ExStatement::DecrimentC)
        .or(just(Token::String("FLAG"))
            .ignore_then(just(Token::String("SAVE")))
            .to(ExStatement::FlagSave))
        .or(just(Token::String("WITH"))
            .ignore_then(just(Token::String("CRY")))
            .to(ExStatement::WithCry))
        .or(just(Token::String("WITH"))
            .ignore_then(just(Token::String("ONE")))
            .to(ExStatement::WithOne))
        .or(just(Token::String("IR"))
            .ignore_then(just(Token::ColonEqual))
            .ignore_then(lbus_source.clone())
            .map(ExStatement::Ir))
        .or(just(Token::String("IO"))
            .ignore_then(just(Token::ColonEqual))
            .ignore_then(lbus_source)
            .map(ExStatement::Io))
        .or(just(Token::String("C"))
            .ignore_then(just(Token::ColonEqual))
            .ignore_then(rbus_source)
            .map(ExStatement::C))
        .or(just(Token::String("EXECUTE"))
            .ignore_then(just(Token::String("IO")))
            .to(ExStatement::ExecuteIo))
        .or(just(Token::String("T"))
            .ignore_then(just(Token::ColonEqual))
            .ignore_then(just(Token::Hexadecimal(1)))
            .to(ExStatement::Assign1ToT))
        .or(just(Token::String("T"))
            .ignore_then(just(Token::ColonEqual))
            .ignore_then(just(Token::Hexadecimal(0)))
            .to(ExStatement::Assign0ToT))
        .or(just(Token::String("IRA"))
            .ignore_then(just(Token::Plus))
            .ignore_then(just(Token::Hexadecimal(1)))
            .to(ExStatement::IraPlus1))
        .or(just(Token::String("IRB"))
            .ignore_then(just(Token::Plus))
            .ignore_then(just(Token::Hexadecimal(1)))
            .to(ExStatement::IrbPlus1))
        .or(just(Token::String("IRB"))
            .ignore_then(just(Token::Minus))
            .ignore_then(just(Token::Hexadecimal(1)))
            .to(ExStatement::IrbMinus1))
        .or(just(Token::String("SET"))
            .ignore_then(just(Token::String("HLT")))
            .to(ExStatement::SetHlt))
        .or(just(Token::String("SET"))
            .ignore_then(just(Token::String("OV")))
            .to(ExStatement::SetOv))
        .or(just(Token::String("NEX")).to(ExStatement::Nex))
        .then_ignore(just(Token::Eol))
        .or_not();

    let eols = just(Token::Eol).repeated();
    let instruction_body = test_and_sequence_statement
        .then_ignore(eols.clone())
        .then(memory_statement.then_ignore(eols.clone()))
        .then(calculation_statement.then_ignore(eols.clone()))
        .then(ex_statement.then_ignore(eols.clone()));

    let instruction = eols
        .clone()
        .ignore_then(instruction_head)
        .then_ignore(eols.clone())
        .then(instruction_body)
        .recover_with(skip_then_retry_until([Token::Star]));

    let program_title = just(Token::DotString("TITLE")).ignore_then(string);
    let program_body = eols
        .clone()
        .ignore_then(instruction)
        .repeated()
        .at_least(1)
        .map(|instructions| {
            instructions
                .into_iter()
                .map(
                    |(
                        (label, address),
                        (
                            (
                                (test_and_sequence_statement, memory_statement),
                                calculation_statement,
                            ),
                            ex_statement,
                        ),
                    )| Instruction {
                        label,
                        address,
                        test_and_sequence_statement,
                        memory_statement,
                        calculation_statement,
                        ex_statement,
                    },
                )
                .collect()
        });

    let program = program_title
        .then(program_body.delimited_by(just(Token::Eol), just(Token::DotString("END"))))
        .then_ignore(eols.clone())
        .then_ignore(end());

    eols.ignore_then(program).map(|(title, instructions)| Ast {
        title,
        instructions,
    })
}

pub fn parse(tokens: Vec<(Token, Span)>) -> (Option<Ast>, Vec<Simple<Token>>) {
    parser().parse_recovery(Stream::from_iter(0..tokens.len(), tokens.into_iter()))
}
