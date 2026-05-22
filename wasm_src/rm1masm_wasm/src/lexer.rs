use crate::token::Token;
use logos::{Logos, Span};

pub fn tokenize(input: &str) -> Vec<(Token, Span)> {
    Token::lexer(input).spanned().collect()
}

#[cfg(test)]
mod tests {
    use super::tokenize;
    use super::Token;

    #[test]
    fn test_dotstring() {
        let input = ".TITLE .END";
        let expected = vec![
            (Token::DotString("TITLE"), 0..6),
            (Token::DotString("END"), 7..11),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_string() {
        let input = "GOTO FETCH CALL RETURN IF THEN ELSE ZER NEG CRY OV CZ T IOP IRA IRB NSQ SET BY R0 R1 R2 R3 R4 R5 R6 R7 RA RAP RB RBP PC IO MM IR FSR ZERO AND OR XOR SLL SRL SLA SRA SNX SWP NSB READ WRITE C FLAG SAVE WITH CRY ONE EXECUTE T";
        let expected = vec![
            (Token::String("GOTO"), 0..4),
            (Token::String("FETCH"), 5..10),
            (Token::String("CALL"), 11..15),
            (Token::String("RETURN"), 16..22),
            (Token::String("IF"), 23..25),
            (Token::String("THEN"), 26..30),
            (Token::String("ELSE"), 31..35),
            (Token::String("ZER"), 36..39),
            (Token::String("NEG"), 40..43),
            (Token::String("CRY"), 44..47),
            (Token::String("OV"), 48..50),
            (Token::String("CZ"), 51..53),
            (Token::String("T"), 54..55),
            (Token::String("IOP"), 56..59),
            (Token::String("IRA"), 60..63),
            (Token::String("IRB"), 64..67),
            (Token::String("NSQ"), 68..71),
            (Token::String("SET"), 72..75),
            (Token::String("BY"), 76..78),
            (Token::String("R0"), 79..81),
            (Token::String("R1"), 82..84),
            (Token::String("R2"), 85..87),
            (Token::String("R3"), 88..90),
            (Token::String("R4"), 91..93),
            (Token::String("R5"), 94..96),
            (Token::String("R6"), 97..99),
            (Token::String("R7"), 100..102),
            (Token::String("RA"), 103..105),
            (Token::String("RAP"), 106..109),
            (Token::String("RB"), 110..112),
            (Token::String("RBP"), 113..116),
            (Token::String("PC"), 117..119),
            (Token::String("IO"), 120..122),
            (Token::String("MM"), 123..125),
            (Token::String("IR"), 126..128),
            (Token::String("FSR"), 129..132),
            (Token::String("ZERO"), 133..137),
            (Token::String("AND"), 138..141),
            (Token::String("OR"), 142..144),
            (Token::String("XOR"), 145..148),
            (Token::String("SLL"), 149..152),
            (Token::String("SRL"), 153..156),
            (Token::String("SLA"), 157..160),
            (Token::String("SRA"), 161..164),
            (Token::String("SNX"), 165..168),
            (Token::String("SWP"), 169..172),
            (Token::String("NSB"), 173..176),
            (Token::String("READ"), 177..181),
            (Token::String("WRITE"), 182..187),
            (Token::String("C"), 188..189),
            (Token::String("FLAG"), 190..194),
            (Token::String("SAVE"), 195..199),
            (Token::String("WITH"), 200..204),
            (Token::String("CRY"), 205..208),
            (Token::String("ONE"), 209..212),
            (Token::String("EXECUTE"), 213..220),
            (Token::String("T"), 221..222),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_hexadecimal() {
        let input = "01 23 45 67 89 0AB 1cd 2Ef 3FFFFF";
        let expected = vec![
            (Token::Hexadecimal(0x01u16), 0..2),
            (Token::Hexadecimal(0x23u16), 3..5),
            (Token::Hexadecimal(0x45u16), 6..8),
            (Token::Hexadecimal(0x67u16), 9..11),
            (Token::Hexadecimal(0x89u16), 12..14),
            (Token::Hexadecimal(0x0ABu16), 15..18),
            (Token::Hexadecimal(0x1cdu16), 19..22),
            (Token::Hexadecimal(0x2EFu16), 23..26),
            (Token::Error, 27..33),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decimal() {
        let input = r#"D"0 D"1 D"2 D"3 D"4 D"5 D"6 D"7 D"8 D"9 D"10 D"255 D"65535 D"65536"#;
        let expected = vec![
            (Token::Decimal(0u16), 0..3),
            (Token::Decimal(1u16), 4..7),
            (Token::Decimal(2u16), 8..11),
            (Token::Decimal(3u16), 12..15),
            (Token::Decimal(4u16), 16..19),
            (Token::Decimal(5u16), 20..23),
            (Token::Decimal(6u16), 24..27),
            (Token::Decimal(7u16), 28..31),
            (Token::Decimal(8u16), 32..35),
            (Token::Decimal(9u16), 36..39),
            (Token::Decimal(10u16), 40..44),
            (Token::Decimal(255u16), 45..50),
            (Token::Decimal(65535u16), 51..58),
            (Token::Error, 59..66),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_binary() {
        let input = r#"B"01011010 B"10100101 B"11111111111111111"#;
        let expected = vec![
            (Token::Binary(0b01011010u16), 0..10),
            (Token::Binary(0b10100101u16), 11..21),
            (Token::Error, 22..41),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_star() {
        let input = "* *    *";
        let expected = vec![
            (Token::Star, 0..1),
            (Token::Star, 2..3),
            (Token::Star, 7..8),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_plus() {
        let input = "+ +    +";
        let expected = vec![
            (Token::Plus, 0..1),
            (Token::Plus, 2..3),
            (Token::Plus, 7..8),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_minus() {
        let input = "- -    -";
        let expected = vec![
            (Token::Minus, 0..1),
            (Token::Minus, 2..3),
            (Token::Minus, 7..8),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_dollar() {
        let input = "$ $    $";
        let expected = vec![
            (Token::Dollar, 0..1),
            (Token::Dollar, 2..3),
            (Token::Dollar, 7..8),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_at() {
        let input = "@ @    @";
        let expected = vec![(Token::At, 0..1), (Token::At, 2..3), (Token::At, 7..8)];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_colonequal() {
        let input = ":= :=    :=";
        let expected = vec![
            (Token::ColonEqual, 0..2),
            (Token::ColonEqual, 3..5),
            (Token::ColonEqual, 9..11),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_colon() {
        let input = ": :    :";
        let expected = vec![
            (Token::Colon, 0..1),
            (Token::Colon, 2..3),
            (Token::Colon, 7..8),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_equal() {
        let input = "= =    =";
        let expected = vec![
            (Token::Equal, 0..1),
            (Token::Equal, 2..3),
            (Token::Equal, 7..8),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_eol() {
        let input = "\n \n    \n";
        let expected = vec![(Token::Eol, 0..1), (Token::Eol, 2..3), (Token::Eol, 7..8)];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }
}
