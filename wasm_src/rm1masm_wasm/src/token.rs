use logos::Logos;

#[derive(Clone, Debug, Eq, Hash, Logos, PartialEq)]
pub enum Token<'a> {
    #[regex(r"\.[a-zA-Z][a-zA-Z0-9]*", |lex| &lex.slice()[1..])]
    DotString(&'a str),
    #[regex("[a-zA-Z][a-zA-Z0-9]*", |lex| lex.slice())]
    String(&'a str),
    #[regex("[0-9][a-fA-F0-9]*", |lex| u16::from_str_radix(lex.slice(), 16))]
    Hexadecimal(u16),
    #[regex(r#"D"(0|[1-9][0-9]*)"#, |lex| lex.slice()[2..].parse())]
    Decimal(u16),
    #[regex(r#"B"[01]+"#, |lex| u16::from_str_radix(&lex.slice()[2..], 2))]
    Binary(u16),
    #[token("*")]
    Star,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("$")]
    Dollar,
    #[token("@")]
    At,
    #[token(":=")]
    ColonEqual,
    #[token(":")]
    Colon,
    #[token("=")]
    Equal,
    #[regex(r"(;[^\r]*\r)?\n")]
    Eol,
    #[regex(r"[ \r\t\f]+", |_| logos::Skip)]
    #[error]
    Error,
}
