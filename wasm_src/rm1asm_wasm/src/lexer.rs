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
    fn test_string() {
        let input = "ADD SUB AND OR XOR MULT DIV CMP EX LC PUSH POP SL SA SC BIX LEA LX STX L ST LA BDIS BP BZ BM BC BNP BNZ BNM BNC B BI BSR CR LPT RIO WIO RET NOP HLT";
        let expected = vec![
            (Token::String("ADD"), 0..3),
            (Token::String("SUB"), 4..7),
            (Token::String("AND"), 8..11),
            (Token::String("OR"), 12..14),
            (Token::String("XOR"), 15..18),
            (Token::String("MULT"), 19..23),
            (Token::String("DIV"), 24..27),
            (Token::String("CMP"), 28..31),
            (Token::String("EX"), 32..34),
            (Token::String("LC"), 35..37),
            (Token::String("PUSH"), 38..42),
            (Token::String("POP"), 43..46),
            (Token::String("SL"), 47..49),
            (Token::String("SA"), 50..52),
            (Token::String("SC"), 53..55),
            (Token::String("BIX"), 56..59),
            (Token::String("LEA"), 60..63),
            (Token::String("LX"), 64..66),
            (Token::String("STX"), 67..70),
            (Token::String("L"), 71..72),
            (Token::String("ST"), 73..75),
            (Token::String("LA"), 76..78),
            (Token::String("BDIS"), 79..83),
            (Token::String("BP"), 84..86),
            (Token::String("BZ"), 87..89),
            (Token::String("BM"), 90..92),
            (Token::String("BC"), 93..95),
            (Token::String("BNP"), 96..99),
            (Token::String("BNZ"), 100..103),
            (Token::String("BNM"), 104..107),
            (Token::String("BNC"), 108..111),
            (Token::String("B"), 112..113),
            (Token::String("BI"), 114..116),
            (Token::String("BSR"), 117..120),
            (Token::String("CR"), 121..123),
            (Token::String("LPT"), 124..127),
            (Token::String("RIO"), 128..131),
            (Token::String("WIO"), 132..135),
            (Token::String("RET"), 136..139),
            (Token::String("NOP"), 140..143),
            (Token::String("HLT"), 144..147),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_decimal() {
        let input = "0 1 2 3 4 5 6 7 8 9 10 255 65535 65536";
        let expected = vec![
            (Token::Decimal(0u16), 0..1),
            (Token::Decimal(1u16), 2..3),
            (Token::Decimal(2u16), 4..5),
            (Token::Decimal(3u16), 6..7),
            (Token::Decimal(4u16), 8..9),
            (Token::Decimal(5u16), 10..11),
            (Token::Decimal(6u16), 12..13),
            (Token::Decimal(7u16), 14..15),
            (Token::Decimal(8u16), 16..17),
            (Token::Decimal(9u16), 18..19),
            (Token::Decimal(10u16), 20..22),
            (Token::Decimal(255u16), 23..26),
            (Token::Decimal(65535u16), 27..32),
            (Token::Error, 33..38),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_no_prefix_hexadecimal() {
        let input = "37BF";
        let expected = vec![(Token::NoPrefixHexadecimal(0x37BFu16), 0..4)];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_hexadecimal() {
        let input = "X\"01 X\"23 X\"45 X\"67 X\"89 X\"AB X\"cd X\"Ef X\"FFFFF";
        let expected = vec![
            (Token::Hexadecimal(0x01u16), 0..4),
            (Token::Hexadecimal(0x23u16), 5..9),
            (Token::Hexadecimal(0x45u16), 10..14),
            (Token::Hexadecimal(0x67u16), 15..19),
            (Token::Hexadecimal(0x89u16), 20..24),
            (Token::Hexadecimal(0xABu16), 25..29),
            (Token::Hexadecimal(0xCDu16), 30..34),
            (Token::Hexadecimal(0xEFu16), 35..39),
            (Token::Error, 40..47),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_octal() {
        let input = "O\"0123 O\"4567 O\"01234567";
        let expected = vec![
            (Token::Octal(0o0123u16), 0..6),
            (Token::Octal(0o4567u16), 7..13),
            (Token::Error, 14..24),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_binary() {
        let input = "B\"01011010 B\"10100101 B\"11111111111111111";
        let expected = vec![
            (Token::Binary(0b01011010u16), 0..10),
            (Token::Binary(0b10100101u16), 11..21),
            (Token::Error, 22..41),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_lparen() {
        let input = "( (    (";
        let expected = vec![
            (Token::Lparen, 0..1),
            (Token::Lparen, 2..3),
            (Token::Lparen, 7..8),
        ];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_rparen() {
        let input = ") )    )";
        let expected = vec![
            (Token::Rparen, 0..1),
            (Token::Rparen, 2..3),
            (Token::Rparen, 7..8),
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
    fn test_chars() {
        let input = "'!\" '#$ '%& ''( ')* '+, '-. '/0 '12 '34 '56 '78 '9: ';< '=> '?@ 'AB 'CD 'EF 'GH 'IJ 'KL 'MN 'OP 'QR 'ST 'UV 'WX 'YZ '[\\ ']^ '_` 'ab 'cd 'ef 'gh 'ij 'kl 'mn 'op 'qr 'st 'uv 'wx 'yz '{| '}~";
        let expected = vec![
            (Token::Chars("!\""), 0..3),
            (Token::Chars("#$"), 4..7),
            (Token::Chars("%&"), 8..11),
            (Token::Chars("'("), 12..15),
            (Token::Chars(")*"), 16..19),
            (Token::Chars("+,"), 20..23),
            (Token::Chars("-."), 24..27),
            (Token::Chars("/0"), 28..31),
            (Token::Chars("12"), 32..35),
            (Token::Chars("34"), 36..39),
            (Token::Chars("56"), 40..43),
            (Token::Chars("78"), 44..47),
            (Token::Chars("9:"), 48..51),
            (Token::Chars(";<"), 52..55),
            (Token::Chars("=>"), 56..59),
            (Token::Chars("?@"), 60..63),
            (Token::Chars("AB"), 64..67),
            (Token::Chars("CD"), 68..71),
            (Token::Chars("EF"), 72..75),
            (Token::Chars("GH"), 76..79),
            (Token::Chars("IJ"), 80..83),
            (Token::Chars("KL"), 84..87),
            (Token::Chars("MN"), 88..91),
            (Token::Chars("OP"), 92..95),
            (Token::Chars("QR"), 96..99),
            (Token::Chars("ST"), 100..103),
            (Token::Chars("UV"), 104..107),
            (Token::Chars("WX"), 108..111),
            (Token::Chars("YZ"), 112..115),
            (Token::Chars("[\\"), 116..119),
            (Token::Chars("]^"), 120..123),
            (Token::Chars("_`"), 124..127),
            (Token::Chars("ab"), 128..131),
            (Token::Chars("cd"), 132..135),
            (Token::Chars("ef"), 136..139),
            (Token::Chars("gh"), 140..143),
            (Token::Chars("ij"), 144..147),
            (Token::Chars("kl"), 148..151),
            (Token::Chars("mn"), 152..155),
            (Token::Chars("op"), 156..159),
            (Token::Chars("qr"), 160..163),
            (Token::Chars("st"), 164..167),
            (Token::Chars("uv"), 168..171),
            (Token::Chars("wx"), 172..175),
            (Token::Chars("yz"), 176..179),
            (Token::Chars("{|"), 180..183),
            (Token::Chars("}~"), 184..187),
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
    fn test_comma() {
        let input = ", ,    ,";
        let expected = vec![
            (Token::Comma, 0..1),
            (Token::Comma, 2..3),
            (Token::Comma, 7..8),
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
    fn test_eol() {
        let input = "\n \n    \n";
        let expected = vec![(Token::Eol, 0..1), (Token::Eol, 2..3), (Token::Eol, 7..8)];
        let actual = tokenize(input);
        assert_eq!(expected, actual);
    }
}
