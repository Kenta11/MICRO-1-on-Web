use crate::instruction::MachineAddress;

use logos::Logos;

#[derive(Clone, Debug, Eq, Hash, Logos, PartialEq)]
pub enum Token<'a> {
    #[regex("[a-zA-Z][a-zA-Z0-9]*", |lex| lex.slice())]
    String(&'a str),
    #[regex(r"0|[1-9][0-9]*", |lex| lex.slice().parse())]
    Decimal(u16),
    #[regex(r"[0-9]+[a-fA-F][a-fA-F0-9]+", |lex| u16::from_str_radix(lex.slice(), 16))]
    NoPrefixHexadecimal(MachineAddress), // This is only used by ORG instruction
    #[regex(r#"X"[a-fA-F0-9]+"#, |lex| u16::from_str_radix(&lex.slice()[2..], 16))]
    Hexadecimal(u16),
    #[regex(r#"O"[0-7]+"#, |lex| u16::from_str_radix(&lex.slice()[2..], 8))]
    Octal(u16),
    #[regex(r#"B"[01]+"#, |lex| u16::from_str_radix(&lex.slice()[2..], 2))]
    Binary(u16),
    #[token("(")]
    Lparen,
    #[token(")")]
    Rparen,
    #[token("*")]
    Star,
    #[regex("'..", |lex| &lex.slice()[1..=2])]
    Chars(&'a str),
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("\n")]
    Eol,
    #[regex(r"[ \r\t\f]+|;.*", |_| logos::Skip)]
    #[error]
    Error,
}
