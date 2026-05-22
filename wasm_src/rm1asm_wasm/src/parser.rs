use crate::instruction::*;
use crate::token::Token;

use chumsky::prelude::*;
use chumsky::Stream;
use logos::Span;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Ast<'a> {
    pub title: &'a str,
    pub lines: Vec<ProgramLine<'a>>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProgramLine<'a> {
    pub label: Option<&'a str>,
    pub instruction: Option<Instruction<'a>>,
}

fn parser<'a>() -> impl Parser<Token<'a>, Ast<'a>, Error = Simple<Token<'a>>> {
    let string = select! { Token::String(s) => s };
    let decimal = select! { Token::Decimal(d) => d };
    let unsigned_integer = select! {
        Token::Decimal(d) => d,
        Token::Hexadecimal(h) => h,
        Token::Octal(o) => o,
        Token::Binary(b) => b,
    };
    let signed_integer = just(Token::Plus)
        .or(just(Token::Minus))
        .or_not()
        .then(unsigned_integer)
        .map(|(sign, d)| {
            if sign == Some(Token::Minus) {
                (-(d as i16)) as u16
            } else {
                d
            }
        });
    let register = select! {
        Token::Decimal(0) => Register::Zero,
        Token::Decimal(1) => Register::One,
        Token::Decimal(2) => Register::Two,
        Token::Decimal(3) => Register::Three,
    };
    let address = just(Token::Star)
        .ignore_then(
            just(Token::Plus)
                .or(just(Token::Minus))
                .then(decimal)
                .or_not(),
        )
        .map(|op_and_d| {
            Address::Constant(if let Some((op, d)) = op_and_d {
                if op == Token::Plus {
                    d as MachineAddress
                } else {
                    (-(d as i16)) as MachineAddress
                }
            } else {
                0
            })
        })
        .or(string
            .then(
                just(Token::Plus)
                    .or(just(Token::Minus))
                    .then(decimal)
                    .or_not(),
            )
            .map(|(symbol_name, op_and_d)| Address::Unresolved {
                symbol_name,
                offset: {
                    if let Some((op, d)) = op_and_d {
                        if op == Token::Plus {
                            d as i16
                        } else {
                            -(d as i16)
                        }
                    } else {
                        0i16
                    }
                },
            }));

    let opecode_1th = select! {
        Token::String("ADD") => Opecode1::Add,
        Token::String("SUB") => Opecode1::Sub,
        Token::String("AND") => Opecode1::And,
        Token::String("OR") => Opecode1::Or,
        Token::String("XOR") => Opecode1::Xor,
        Token::String("MULT") => Opecode1::Mult,
        Token::String("DIV") => Opecode1::Div,
        Token::String("CMP") => Opecode1::Cmp,
        Token::String("EX") => Opecode1::Ex,
    };
    let instruction_1th_first_half = opecode_1th.then(register).then_ignore(just(Token::Comma));
    let instruction_1th = instruction_1th_first_half
        .clone()
        .then(unsigned_integer)
        .then(register.delimited_by(just(Token::Lparen), just(Token::Rparen)))
        .map(|(((op, rb), constant), ra)| Instruction::Group1 {
            op,
            ra,
            rb,
            constant: constant as u8,
        })
        .or(instruction_1th_first_half
            .clone()
            .then(register.delimited_by(just(Token::Lparen), just(Token::Rparen)))
            .map(|((op, rb), ra)| Instruction::Group1 {
                op,
                ra,
                rb,
                constant: 0u8,
            }))
        .or(instruction_1th_first_half
            .then(unsigned_integer)
            .map(|((op, rb), constant)| Instruction::Group1 {
                op,
                ra: Register::Zero,
                rb,
                constant: constant as u8,
            }));

    let opecode_2th = select! {
        Token::String("LC") => Opecode2::Lc,
        Token::String("PUSH") => Opecode2::Push,
        Token::String("POP") => Opecode2::Pop,
    };
    let instruction_2th = opecode_2th
        .then(register)
        .then_ignore(just(Token::Comma))
        .then(unsigned_integer)
        .map(|((op, rb), constant)| Instruction::Group2 {
            op,
            rb,
            constant: constant as u8,
        });

    let opecode_3th = select! {
        Token::String("SL") => Opecode3::Sl,
        Token::String("SA") => Opecode3::Sa,
        Token::String("SC") => Opecode3::Sc,
        Token::String("BIX") => Opecode3::Bix,
    };
    let instruction_3th = opecode_3th
        .then(register)
        .then_ignore(just(Token::Comma))
        .then(signed_integer.clone())
        .map(|((op, rb), constant)| Instruction::Group3 {
            op,
            rb,
            constant: constant as i8,
        });

    let opecode_4th = select! {
        Token::String("LEA") => Opecode4::Lea,
        Token::String("LX") => Opecode4::Lx,
        Token::String("STX") => Opecode4::Stx,
    };
    let instruction_4th = opecode_4th
        .then(register)
        .then_ignore(just(Token::Comma))
        .then(signed_integer.clone().or_not())
        .then(register.delimited_by(just(Token::Lparen), just(Token::Rparen)))
        .map(|(((op, rb), constant), ra)| Instruction::Group4 {
            op,
            ra,
            rb,
            constant: if let Some(constant) = constant {
                constant as i8
            } else {
                0
            },
        });

    let opecode_5th = select! {
        Token::String("L") => Opecode5::L,
        Token::String("ST") => Opecode5::St,
        Token::String("LA") => Opecode5::La,
    };
    let instruction_5th = opecode_5th
        .then(register)
        .then_ignore(just(Token::Comma))
        .then(address.clone())
        .map(|((op, rb), address)| Instruction::Group5 { op, rb, address });

    let opecode_6th = select! {
        Token::String("BDIS") => Opecode6::Bdis,
        Token::String("BP") => Opecode6::Bp,
        Token::String("BZ") => Opecode6::Bz,
        Token::String("BM") => Opecode6::Bm,
        Token::String("BC") => Opecode6::Bc,
        Token::String("BNP") => Opecode6::Bnp,
        Token::String("BNZ") => Opecode6::Bnz,
        Token::String("BNM") => Opecode6::Bnm,
        Token::String("BNC") => Opecode6::Bnc,
        Token::String("B") => Opecode6::B,
        Token::String("BI") => Opecode6::Bi,
        Token::String("BSR") => Opecode6::Bsr,
    };
    let instruction_6th = opecode_6th
        .then(address)
        .map(|(op, address)| Instruction::Group6 { op, address });

    let opecode_7th = select! {
        Token::String("RIO") => Opecode7::Rio,
        Token::String("WIO") => Opecode7::Wio,
    };
    let device = select! {
        Token::String("CR") => Device::Cr,
        Token::Decimal(0) => Device::Cr,
        Token::String("LPT") => Device::Lpt,
        Token::Decimal(1) => Device::Lpt,
    };
    let instruction_7th = opecode_7th
        .then(device)
        .map(|(op, device)| Instruction::Group7 { op, device });

    let opecode_8th = select! {
        Token::String("RET") => Opecode8::Ret,
        Token::String("NOP") => Opecode8::Nop,
        Token::String("HLT") => Opecode8::Hlt,
    };
    let instruction_8th = opecode_8th.map(|op| Instruction::Group8 { op });

    let instruction_9th = just(Token::String("DC"))
        .ignore_then(
            signed_integer
                .map(|value| Instruction::Dc {
                    value,
                    unresolved_symbol: None,
                })
            .or(
                (select! { Token::Chars(s) => ((s.chars().next().unwrap() as MachineCode) << 8) + (s.chars().nth(1).unwrap() as MachineCode) })
                .map(|value| Instruction::Dc {
                    value,
                    unresolved_symbol: None,
                })
            )
            .or(string
                .map(|unresolved_symbol| Instruction::Dc {
                    value: 0,
                    unresolved_symbol: Some(unresolved_symbol),
                })
            )
        ).or(
            just(Token::String("DS"))
                .ignore_then(decimal)
                .map(Instruction::Ds)
        ).or(
            just(Token::String("ORG"))
                .ignore_then(select! {
                    Token::Decimal(d) => MachineAddress::from_str_radix(&d.to_string(), 16).unwrap(),
                    Token::NoPrefixHexadecimal(h) => h,
                }.or(
                    string.try_map(|s, span| MachineAddress::from_str_radix(s, 16)
                    .map_err(|e| Simple::custom(span, format!("{e}"))))
                ))
                .map(Instruction::Org)
        );

    let instruction = string
        .then_ignore(just(Token::Colon))
        .or_not()
        .then(
            instruction_1th
                .or(instruction_2th)
                .or(instruction_3th)
                .or(instruction_4th)
                .or(instruction_5th)
                .or(instruction_6th)
                .or(instruction_7th)
                .or(instruction_8th)
                .or(instruction_9th)
                .or_not(),
        )
        .then_ignore(just(Token::Eol))
        .recover_with(skip_then_retry_until([Token::Eol]));

    let program_head = just(Token::String("TITLE")).ignore_then(string);
    let program_body = instruction.repeated().at_least(1).map(|lines| {
        lines
            .into_iter()
            .filter(|(label, instruction)| label.is_some() || instruction.is_some())
            .map(|(label, instruction)| ProgramLine { label, instruction })
            .collect()
    });
    let program_tail = just(Token::String("END"));

    let program = program_head
        .then(program_body.delimited_by(just(Token::Eol), program_tail))
        .then_ignore(end());

    program.map(|(title, lines)| Ast { title, lines })
}

pub fn parse(tokens: Vec<(Token, Span)>) -> (Option<Ast>, Vec<Simple<Token>>) {
    let start = if let Some(start) = tokens.iter().position(|(x, _)| *x != Token::Eol) {
        start
    } else {
        0
    };
    let end = if let Some(end) = tokens.iter().rposition(|(x, _)| *x != Token::Eol) {
        end + 1
    } else {
        tokens.len()
    };
    parser().parse_recovery(Stream::from_iter(
        start..end,
        tokens[start..end].iter().cloned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use super::Address;
    use super::Ast;
    use super::Device;
    use super::Instruction;
    use super::MachineAddress;
    use super::ProgramLine;
    use super::Register;
    use super::Simple;
    use super::Token;
    use super::{Opecode1, Opecode2, Opecode3, Opecode4, Opecode5, Opecode6, Opecode7, Opecode8};

    #[test]
    fn test_op1() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op1"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("ADD"), 17..20),
            (Token::Decimal(0), 21..22),
            (Token::Comma, 23..24),
            (Token::Decimal(10), 25..27),
            (Token::Lparen, 28..29),
            (Token::Decimal(1), 30..31),
            (Token::Rparen, 32..33),
            (Token::Eol, 34..35),
            (Token::String("SUB"), 36..39),
            (Token::Decimal(2), 40..41),
            (Token::Comma, 42..43),
            (Token::Hexadecimal(0x32), 44..48),
            (Token::Eol, 49..50),
            (Token::String("AND"), 51..54),
            (Token::Decimal(0), 55..56),
            (Token::Comma, 57..58),
            (Token::Lparen, 59..60),
            (Token::Decimal(3), 61..62),
            (Token::Rparen, 63..64),
            (Token::Eol, 65..66),
            (Token::String("OR"), 67..69),
            (Token::Decimal(0), 70..71),
            (Token::Comma, 72..73),
            (Token::Octal(0o54), 74..78),
            (Token::Lparen, 79..80),
            (Token::Decimal(1), 81..82),
            (Token::Rparen, 83..84),
            (Token::Eol, 85..86),
            (Token::String("XOR"), 87..90),
            (Token::Decimal(2), 91..92),
            (Token::Comma, 93..94),
            (Token::Binary(0b01110101), 95..105),
            (Token::Eol, 106..107),
            (Token::String("MULT"), 108..112),
            (Token::Decimal(0), 113..114),
            (Token::Comma, 115..116),
            (Token::Lparen, 117..118),
            (Token::Decimal(3), 119..120),
            (Token::Rparen, 121..122),
            (Token::Eol, 123..124),
            (Token::String("DIV"), 125..128),
            (Token::Decimal(2), 129..130),
            (Token::Comma, 131..132),
            (Token::Decimal(98), 133..135),
            (Token::Lparen, 136..137),
            (Token::Decimal(1), 138..139),
            (Token::Rparen, 140..141),
            (Token::Eol, 142..143),
            (Token::String("CMP"), 144..147),
            (Token::Decimal(3), 148..149),
            (Token::Comma, 150..151),
            (Token::Hexadecimal(0xBA), 152..156),
            (Token::Eol, 157..158),
            (Token::String("EX"), 159..161),
            (Token::Decimal(0), 162..163),
            (Token::Comma, 164..165),
            (Token::Lparen, 169..170),
            (Token::Decimal(1), 171..172),
            (Token::Rparen, 173..174),
            (Token::Eol, 175..176),
            (Token::String("END"), 177..180),
        ];
        let expected = (
            Some(Ast {
                title: "test_op1",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::Add,
                            ra: Register::One,
                            rb: Register::Zero,
                            constant: 10,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::Sub,
                            ra: Register::Zero,
                            rb: Register::Two,
                            constant: 0x32,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::And,
                            ra: Register::Three,
                            rb: Register::Zero,
                            constant: 0,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::Or,
                            ra: Register::One,
                            rb: Register::Zero,
                            constant: 0o54,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::Xor,
                            ra: Register::Zero,
                            rb: Register::Two,
                            constant: 0b01110101,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::Mult,
                            ra: Register::Three,
                            rb: Register::Zero,
                            constant: 0,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::Div,
                            ra: Register::One,
                            rb: Register::Two,
                            constant: 98,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::Cmp,
                            ra: Register::Zero,
                            rb: Register::Three,
                            constant: 0xBA,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group1 {
                            op: Opecode1::Ex,
                            ra: Register::One,
                            rb: Register::Zero,
                            constant: 0,
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_op2() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op2"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("LC"), 17..19),
            (Token::Decimal(0), 20..21),
            (Token::Comma, 22..23),
            (Token::Decimal(12), 24..26),
            (Token::Eol, 27..28),
            (Token::String("PUSH"), 29..33),
            (Token::Decimal(1), 34..35),
            (Token::Comma, 36..37),
            (Token::Hexadecimal(0x34), 38..42),
            (Token::Eol, 43..44),
            (Token::String("POP"), 45..48),
            (Token::Decimal(2), 49..50),
            (Token::Comma, 51..52),
            (Token::Octal(0o56), 53..57),
            (Token::Eol, 58..59),
            (Token::String("END"), 60..63),
        ];
        let expected = (
            Some(Ast {
                title: "test_op2",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group2 {
                            op: Opecode2::Lc,
                            rb: Register::Zero,
                            constant: 12,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group2 {
                            op: Opecode2::Push,
                            rb: Register::One,
                            constant: 0x34,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group2 {
                            op: Opecode2::Pop,
                            rb: Register::Two,
                            constant: 0o56,
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_op3() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op3"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("SL"), 17..19),
            (Token::Decimal(0), 20..21),
            (Token::Comma, 22..23),
            (Token::Plus, 24..25),
            (Token::Decimal(12), 26..28),
            (Token::Eol, 29..30),
            (Token::String("SA"), 31..33),
            (Token::Decimal(1), 34..35),
            (Token::Comma, 36..37),
            (Token::Minus, 38..39),
            (Token::Hexadecimal(0x34), 40..44),
            (Token::Eol, 45..46),
            (Token::String("SC"), 47..49),
            (Token::Decimal(2), 50..51),
            (Token::Comma, 52..53),
            (Token::Plus, 54..55),
            (Token::Octal(0o56), 56..60),
            (Token::Eol, 61..62),
            (Token::String("BIX"), 65..68),
            (Token::Decimal(3), 69..70),
            (Token::Comma, 71..72),
            (Token::Minus, 73..74),
            (Token::Binary(0b0100101), 75..85),
            (Token::Eol, 86..87),
            (Token::String("END"), 88..91),
        ];
        let expected = (
            Some(Ast {
                title: "test_op3",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group3 {
                            op: Opecode3::Sl,
                            rb: Register::Zero,
                            constant: 12,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group3 {
                            op: Opecode3::Sa,
                            rb: Register::One,
                            constant: -0x34,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group3 {
                            op: Opecode3::Sc,
                            rb: Register::Two,
                            constant: 0o56,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group3 {
                            op: Opecode3::Bix,
                            rb: Register::Three,
                            constant: -0b0100101,
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_op4() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op4"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("LEA"), 17..20),
            (Token::Decimal(1), 21..22),
            (Token::Comma, 23..24),
            (Token::Plus, 25..26),
            (Token::Decimal(12), 27..29),
            (Token::Lparen, 30..31),
            (Token::Decimal(0), 32..33),
            (Token::Rparen, 34..35),
            (Token::Eol, 36..37),
            (Token::String("LX"), 38..40),
            (Token::Decimal(3), 41..42),
            (Token::Comma, 43..44),
            (Token::Lparen, 45..46),
            (Token::Decimal(2), 47..48),
            (Token::Rparen, 49..50),
            (Token::Eol, 51..52),
            (Token::String("STX"), 53..56),
            (Token::Decimal(1), 57..58),
            (Token::Comma, 59..60),
            (Token::Minus, 61..62),
            (Token::Hexadecimal(0x34), 63..67),
            (Token::Lparen, 68..69),
            (Token::Decimal(0), 70..71),
            (Token::Rparen, 72..73),
            (Token::Eol, 74..75),
            (Token::String("END"), 76..79),
        ];
        let expected = (
            Some(Ast {
                title: "test_op4",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group4 {
                            op: Opecode4::Lea,
                            ra: Register::Zero,
                            rb: Register::One,
                            constant: 12,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group4 {
                            op: Opecode4::Lx,
                            ra: Register::Two,
                            rb: Register::Three,
                            constant: 0,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group4 {
                            op: Opecode4::Stx,
                            ra: Register::Zero,
                            rb: Register::One,
                            constant: -0x34,
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_op5() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op5"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("L"), 17..18),
            (Token::Decimal(0), 19..20),
            (Token::Comma, 21..22),
            (Token::Star, 23..24),
            (Token::Eol, 25..26),
            (Token::String("ST"), 27..29),
            (Token::Decimal(1), 30..31),
            (Token::Comma, 32..33),
            (Token::String("Label1"), 34..40),
            (Token::Plus, 41..42),
            (Token::Decimal(12), 43..45),
            (Token::Eol, 46..47),
            (Token::String("LA"), 48..50),
            (Token::Decimal(2), 51..52),
            (Token::Comma, 53..54),
            (Token::Star, 55..56),
            (Token::Minus, 57..58),
            (Token::Decimal(34), 59..61),
            (Token::Eol, 62..63),
            (Token::String("END"), 64..65),
        ];
        let expected = (
            Some(Ast {
                title: "test_op5",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group5 {
                            op: Opecode5::L,
                            rb: Register::Zero,
                            address: Address::Constant(0),
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group5 {
                            op: Opecode5::St,
                            rb: Register::One,
                            address: Address::Unresolved {
                                symbol_name: "Label1",
                                offset: 12,
                            },
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group5 {
                            op: Opecode5::La,
                            rb: Register::Two,
                            address: Address::Constant(-34i16 as MachineAddress),
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_op6() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op6"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("BDIS"), 17..18),
            (Token::Star, 19..20),
            (Token::Eol, 21..22),
            (Token::String("BP"), 23..25),
            (Token::String("Label1"), 26..32),
            (Token::Plus, 33..34),
            (Token::Decimal(12), 35..37),
            (Token::Eol, 38..39),
            (Token::String("BZ"), 40..42),
            (Token::Star, 43..44),
            (Token::Minus, 45..46),
            (Token::Decimal(34), 47..49),
            (Token::Eol, 50..51),
            (Token::String("BM"), 52..54),
            (Token::String("Label2"), 55..61),
            (Token::Eol, 62..63),
            (Token::String("BC"), 64..66),
            (Token::Star, 67..68),
            (Token::Plus, 69..70),
            (Token::Decimal(56), 71..73),
            (Token::Eol, 74..75),
            (Token::String("BNP"), 76..79),
            (Token::String("Label3"), 80..86),
            (Token::Minus, 87..88),
            (Token::Decimal(78), 89..91),
            (Token::Eol, 92..93),
            (Token::String("BNZ"), 94..97),
            (Token::Star, 98..99),
            (Token::Eol, 100..101),
            (Token::String("BNM"), 102..105),
            (Token::String("Label4"), 106..112),
            (Token::Plus, 113..114),
            (Token::Decimal(90), 115..117),
            (Token::Eol, 118..119),
            (Token::String("BNC"), 120..123),
            (Token::Star, 124..125),
            (Token::Minus, 126..127),
            (Token::Decimal(12), 128..130),
            (Token::Eol, 131..132),
            (Token::String("B"), 133..134),
            (Token::String("Label5"), 135..141),
            (Token::Eol, 142..143),
            (Token::String("BI"), 144..146),
            (Token::Star, 147..148),
            (Token::Plus, 149..150),
            (Token::Decimal(34), 151..153),
            (Token::Eol, 154..155),
            (Token::String("BSR"), 156..159),
            (Token::String("Label6"), 160..166),
            (Token::Minus, 167..168),
            (Token::Decimal(56), 169..171),
            (Token::Eol, 172..173),
            (Token::String("END"), 174..175),
        ];
        let expected = (
            Some(Ast {
                title: "test_op6",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bdis,
                            address: Address::Constant(0),
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bp,
                            address: Address::Unresolved {
                                symbol_name: "Label1",
                                offset: 12,
                            },
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bz,
                            address: Address::Constant(-34i16 as MachineAddress),
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bm,
                            address: Address::Unresolved {
                                symbol_name: "Label2",
                                offset: 0,
                            },
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bc,
                            address: Address::Constant(56),
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bnp,
                            address: Address::Unresolved {
                                symbol_name: "Label3",
                                offset: -78,
                            },
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bnz,
                            address: Address::Constant(0),
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bnm,
                            address: Address::Unresolved {
                                symbol_name: "Label4",
                                offset: 90,
                            },
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bnc,
                            address: Address::Constant(-12i16 as MachineAddress),
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::B,
                            address: Address::Unresolved {
                                symbol_name: "Label5",
                                offset: 0,
                            },
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bi,
                            address: Address::Constant(34),
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group6 {
                            op: Opecode6::Bsr,
                            address: Address::Unresolved {
                                symbol_name: "Label6",
                                offset: -56,
                            },
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_op7() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op7"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("RIO"), 17..18),
            (Token::Decimal(0), 19..20),
            (Token::Eol, 21..22),
            (Token::String("WIO"), 23..26),
            (Token::String("LPT"), 27..30),
            (Token::Eol, 31..32),
            (Token::String("END"), 33..36),
        ];
        let expected = (
            Some(Ast {
                title: "test_op7",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group7 {
                            op: Opecode7::Rio,
                            device: Device::Cr,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group7 {
                            op: Opecode7::Wio,
                            device: Device::Lpt,
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_op8() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op8"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("RET"), 17..20),
            (Token::Eol, 21..22),
            (Token::String("NOP"), 23..26),
            (Token::Eol, 27..28),
            (Token::String("HLT"), 29..32),
            (Token::Eol, 33..34),
            (Token::String("END"), 35..38),
        ];
        let expected = (
            Some(Ast {
                title: "test_op8",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group8 { op: Opecode8::Ret }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group8 { op: Opecode8::Nop }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group8 { op: Opecode8::Hlt }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_op9() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op9"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("DC"), 17..19),
            (Token::Plus, 20..21),
            (Token::Decimal(12), 22..24),
            (Token::Eol, 25..26),
            (Token::String("DC"), 27..29),
            (Token::Chars("a0"), 30..32),
            (Token::Eol, 33..34),
            (Token::String("DC"), 35..37),
            (Token::String("Label0"), 38..44),
            (Token::Eol, 45..46),
            (Token::String("DS"), 47..49),
            (Token::Decimal(10), 50..51),
            (Token::Eol, 52..53),
            (Token::String("ORG"), 54..57),
            (Token::Decimal(432), 58..62),
            (Token::Eol, 63..64),
            (Token::String("ORG"), 65..68),
            (Token::NoPrefixHexadecimal(0x89A), 69..72),
            (Token::Eol, 73..74),
            (Token::String("ORG"), 75..78),
            (Token::String("A98"), 79..82),
            (Token::Eol, 83..84),
            (Token::String("END"), 85..88),
        ];
        let expected = (
            Some(Ast {
                title: "test_op9",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Dc {
                            value: 12,
                            unresolved_symbol: None,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Dc {
                            value: 0x6130,
                            unresolved_symbol: None,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Dc {
                            value: 0,
                            unresolved_symbol: Some("Label0"),
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Ds(10)),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Org(0x432)),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Org(0x89A)),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Org(0xA98)),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_labels() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op7"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("Label1"), 17..23),
            (Token::Colon, 24..25),
            (Token::String("RIO"), 26..29),
            (Token::Decimal(0), 30..31),
            (Token::Eol, 32..33),
            (Token::String("Label2"), 34..40),
            (Token::Colon, 41..42),
            (Token::String("WIO"), 43..46),
            (Token::String("LPT"), 47..50),
            (Token::Eol, 51..52),
            (Token::String("END"), 53..56),
        ];
        let expected = (
            Some(Ast {
                title: "test_op7",
                lines: vec![
                    ProgramLine {
                        label: Some("Label1"),
                        instruction: Some(Instruction::Group7 {
                            op: Opecode7::Rio,
                            device: Device::Cr,
                        }),
                    },
                    ProgramLine {
                        label: Some("Label2"),
                        instruction: Some(Instruction::Group7 {
                            op: Opecode7::Wio,
                            device: Device::Lpt,
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eols() {
        let input = vec![
            (Token::Eol, 0..1),
            (Token::String("TITLE"), 2..7),
            (Token::String("test_op7"), 8..16),
            (Token::Eol, 17..18),
            (Token::Eol, 19..20),
            (Token::String("RIO"), 21..24),
            (Token::Decimal(0), 25..26),
            (Token::Eol, 27..28),
            (Token::Eol, 29..30),
            (Token::Eol, 31..32),
            (Token::String("WIO"), 33..36),
            (Token::String("LPT"), 37..40),
            (Token::Eol, 41..42),
            (Token::Eol, 43..44),
            (Token::Eol, 45..46),
            (Token::Eol, 47..48),
            (Token::String("END"), 49..52),
            (Token::Eol, 53..54),
            (Token::Eol, 55..56),
            (Token::Eol, 57..58),
            (Token::Eol, 59..60),
            (Token::Eol, 61..62),
        ];
        let expected = (
            Some(Ast {
                title: "test_op7",
                lines: vec![
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group7 {
                            op: Opecode7::Rio,
                            device: Device::Cr,
                        }),
                    },
                    ProgramLine {
                        label: None,
                        instruction: Some(Instruction::Group7 {
                            op: Opecode7::Wio,
                            device: Device::Lpt,
                        }),
                    },
                ],
            }),
            Vec::<Simple<Token>>::new(),
        );
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_invalid_opecode() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op7"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("rio"), 17..18),
            (Token::Decimal(0), 19..20),
            (Token::Eol, 21..22),
            (Token::String("WI"), 23..25),
            (Token::String("LPT"), 26..29),
            (Token::Eol, 30..31),
            (Token::String("END"), 32..35),
        ];
        let expected = None;
        let actual = parse(input);
        assert_eq!(expected, actual.0);
        assert!(!actual.1.is_empty());
    }

    #[test]
    fn test_invalid_opecode2() {
        let input = vec![
            (Token::String("TITLE"), 0..5),
            (Token::String("test_op8"), 6..14),
            (Token::Eol, 15..16),
            (Token::String("ret"), 17..20),
            (Token::Eol, 21..22),
            (Token::String("no"), 23..25),
            (Token::Eol, 26..27),
            (Token::String("HLt"), 28..29),
            (Token::Eol, 30..31),
            (Token::String("END"), 32..35),
        ];
        // let expected = None;
        let actual = parse(input);
        // assert_eq!(expected, actual.0);
        assert!(!actual.1.is_empty());
    }
}
