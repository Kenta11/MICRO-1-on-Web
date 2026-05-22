pub type MachineCode = u16;
pub type MachineAddress = u16;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Address<'a> {
    Constant(MachineAddress),
    Unresolved { symbol_name: &'a str, offset: i16 },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Register {
    Zero,
    One,
    Two,
    Three,
}

impl From<&Register> for MachineCode {
    fn from(from: &Register) -> Self {
        match from {
            Register::Zero => 0,
            Register::One => 1,
            Register::Two => 2,
            Register::Three => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Opecode1 {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Mult,
    Div,
    Cmp,
    Ex,
}

impl From<&Opecode1> for MachineCode {
    fn from(from: &Opecode1) -> Self {
        (match from {
            Opecode1::Add => 0x0,
            Opecode1::Sub => 0x1,
            Opecode1::And => 0x2,
            Opecode1::Or => 0x3,
            Opecode1::Xor => 0x4,
            Opecode1::Mult => 0x6,
            Opecode1::Div => 0x7,
            Opecode1::Cmp => 0x8,
            Opecode1::Ex => 0xF,
        }) << 12
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Opecode2 {
    Lc,
    Push,
    Pop,
}

impl From<&Opecode2> for MachineCode {
    fn from(from: &Opecode2) -> Self {
        (match from {
            Opecode2::Lc => (0x9 << 2) + 3,
            Opecode2::Push => 0xD << 2,
            Opecode2::Pop => (0xD << 2) + 1,
        }) << 10
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Opecode3 {
    Sl,
    Sa,
    Sc,
    Bix,
}

impl From<&Opecode3> for MachineCode {
    fn from(from: &Opecode3) -> Self {
        (match from {
            Opecode3::Sl => 0x5 << 2,
            Opecode3::Sa => (0x5 << 2) + 1,
            Opecode3::Sc => (0x5 << 2) + 2,
            Opecode3::Bix => (0xD << 2) + 2,
        }) << 10
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Opecode4 {
    Lea,
    Lx,
    Stx,
}

impl From<&Opecode4> for MachineCode {
    fn from(from: &Opecode4) -> Self {
        (match from {
            Opecode4::Lea => 0xA,
            Opecode4::Lx => 0xB,
            Opecode4::Stx => 0xC,
        }) << 12
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Opecode5 {
    L,
    St,
    La,
}

impl From<&Opecode5> for MachineCode {
    fn from(from: &Opecode5) -> Self {
        (match from {
            Opecode5::L => 0x9 << 2,
            Opecode5::St => (0x9 << 2) + 1,
            Opecode5::La => (0x9 << 2) + 2,
        }) << 10
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Opecode6 {
    Bdis,
    Bp,
    Bz,
    Bm,
    Bc,
    Bnp,
    Bnz,
    Bnm,
    Bnc,
    B,
    Bi,
    Bsr,
}

impl From<&Opecode6> for MachineCode {
    fn from(from: &Opecode6) -> Self {
        (match from {
            Opecode6::Bdis => (0xD << 4) + (3 << 2),
            Opecode6::Bp => 0xE << 4,
            Opecode6::Bz => (0xE << 4) + 1,
            Opecode6::Bm => (0xE << 4) + 2,
            Opecode6::Bc => (0xE << 4) + 3,
            Opecode6::Bnp => (0xE << 4) + (1 << 2),
            Opecode6::Bnz => (0xE << 4) + (1 << 2) + 1,
            Opecode6::Bnm => (0xE << 4) + (1 << 2) + 2,
            Opecode6::Bnc => (0xE << 4) + (1 << 2) + 3,
            Opecode6::B => (0xE << 4) + (2 << 2),
            Opecode6::Bi => (0xE << 4) + (2 << 2) + 1,
            Opecode6::Bsr => (0xE << 4) + (2 << 2) + 2,
        }) << 8
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Device {
    Cr,
    Lpt,
}

impl From<&Device> for MachineCode {
    fn from(from: &Device) -> Self {
        match from {
            Device::Cr => 0,
            Device::Lpt => 1,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Opecode7 {
    Rio,
    Wio,
}

impl From<&Opecode7> for MachineCode {
    fn from(from: &Opecode7) -> Self {
        (match from {
            Opecode7::Rio => (0xE << 4) + (3 << 2),
            Opecode7::Wio => (0xE << 4) + (3 << 2) + 1,
        }) << 8
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Opecode8 {
    Ret,
    Nop,
    Hlt,
}

impl From<&Opecode8> for MachineCode {
    fn from(from: &Opecode8) -> Self {
        (match from {
            Opecode8::Ret => (0xE << 4) + (2 << 2) + 3,
            Opecode8::Nop => (0xE << 4) + (3 << 2) + 2,
            Opecode8::Hlt => (0xE << 4) + (3 << 2) + 3,
        }) << 8
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Instruction<'a> {
    // 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // |    op    | ra  | rb  |       constant       |
    Group1 {
        op: Opecode1,
        ra: Register,
        rb: Register,
        constant: u8,
    },
    // 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // |       op       | rb  |       constant       |
    Group2 {
        op: Opecode2,
        rb: Register,
        constant: u8,
    },
    // 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // |       op       | rb  |       constant       |
    Group3 {
        op: Opecode3,
        rb: Register,
        constant: i8,
    },
    // 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // |    op    | ra  | rb  |       constant       |
    Group4 {
        op: Opecode4,
        ra: Register,
        rb: Register,
        constant: i8,
    },
    // 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // |       op       | rb  |       address        |
    Group5 {
        op: Opecode5,
        rb: Register,
        address: Address<'a>,
    },
    // 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // |         op           |       address        |
    Group6 {
        op: Opecode6,
        address: Address<'a>,
    },
    // 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // |         op           |       address        |
    Group7 {
        op: Opecode7,
        device: Device,
    },
    // 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // |         op           |                      |
    Group8 {
        op: Opecode8,
    },
    Dc {
        value: MachineCode,
        unresolved_symbol: Option<&'a str>,
    },
    Ds(u16),
    Org(MachineAddress),
}

impl TryFrom<&Instruction<'_>> for MachineCode {
    type Error = ();

    fn try_from(value: &Instruction) -> Result<Self, Self::Error> {
        match value {
            Instruction::Group1 {
                op,
                ra,
                rb,
                constant,
            } => {
                let mut code = MachineCode::from(op);
                code |= MachineCode::from(ra) << 10;
                code |= MachineCode::from(rb) << 8;
                code |= *constant as MachineCode;
                Ok(code)
            }
            Instruction::Group2 { op, rb, constant } => {
                let mut code = MachineCode::from(op);
                code |= MachineCode::from(rb) << 8;
                code |= *constant as MachineCode;
                Ok(code)
            }
            Instruction::Group3 { op, rb, constant } => {
                let mut code = MachineCode::from(op);
                code |= MachineCode::from(rb) << 8;
                code |= (*constant as u8) as MachineCode;
                Ok(code)
            }
            Instruction::Group4 {
                op,
                ra,
                rb,
                constant,
            } => {
                let mut code = MachineCode::from(op);
                code |= MachineCode::from(ra) << 10;
                code |= MachineCode::from(rb) << 8;
                code |= (*constant as u8) as MachineCode;
                Ok(code)
            }
            Instruction::Group5 { op, rb, address } => {
                let mut code = MachineCode::from(op);
                code |= MachineCode::from(rb) << 8;
                if let Address::Constant(c) = address {
                    code |= c & 0xFF;
                    Ok(code)
                } else {
                    Err(())
                }
            }
            Instruction::Group6 { op, address } => {
                let mut code = MachineCode::from(op);
                if let Address::Constant(c) = address {
                    code |= c & 0xFF;
                    Ok(code)
                } else {
                    Err(())
                }
            }
            Instruction::Group7 { op, device } => {
                let mut code = MachineCode::from(op);
                code |= MachineCode::from(device);
                Ok(code)
            }
            Instruction::Group8 { op } => Ok(MachineCode::from(op)),
            Instruction::Dc {
                value,
                unresolved_symbol,
            } => {
                if unresolved_symbol.is_none() {
                    Ok(*value)
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let input = Instruction::Group1 {
            op: Opecode1::Add,
            ra: Register::One,
            rb: Register::Two,
            constant: 4,
        };
        let expected: Result<MachineCode, ()> = Ok(0x0604 as MachineCode);
        let actual: Result<MachineCode, ()> = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_sub() {
        let input = Instruction::Group1 {
            op: Opecode1::Sub,
            ra: Register::Zero,
            rb: Register::Three,
            constant: 5,
        };
        let expected = Ok(0x1305);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_and() {
        let input = Instruction::Group1 {
            op: Opecode1::And,
            ra: Register::One,
            rb: Register::Two,
            constant: 0xAB,
        };
        let expected = Ok(0x26AB);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_or() {
        let input = Instruction::Group1 {
            op: Opecode1::Or,
            ra: Register::Zero,
            rb: Register::Two,
            constant: 0xAB,
        };
        let expected = Ok(0x32AB);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_xor() {
        let input = Instruction::Group1 {
            op: Opecode1::Xor,
            ra: Register::Three,
            rb: Register::One,
            constant: 0xAB,
        };
        let expected = Ok(0x4DAB);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_mult() {
        let input = Instruction::Group1 {
            op: Opecode1::Mult,
            ra: Register::One,
            rb: Register::Two,
            constant: 4,
        };
        let expected = Ok(0x6604);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_div() {
        let input = Instruction::Group1 {
            op: Opecode1::Div,
            ra: Register::Zero,
            rb: Register::One,
            constant: 4,
        };
        let expected = Ok(0x7104);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cmp() {
        let input = Instruction::Group1 {
            op: Opecode1::Cmp,
            ra: Register::Two,
            rb: Register::One,
            constant: 0,
        };
        let expected = Ok(0x8900);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_ex() {
        let input = Instruction::Group1 {
            op: Opecode1::Ex,
            ra: Register::Zero,
            rb: Register::One,
            constant: 0,
        };
        let expected = Ok(0xF100);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_lc() {
        let input = Instruction::Group2 {
            op: Opecode2::Lc,
            rb: Register::Two,
            constant: 0x45,
        };
        let expected = Ok(0x9E45);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_push() {
        let input = Instruction::Group2 {
            op: Opecode2::Push,
            rb: Register::One,
            constant: 4,
        };
        let expected = Ok(0xD104);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_pop() {
        let input = Instruction::Group2 {
            op: Opecode2::Pop,
            rb: Register::Zero,
            constant: 4,
        };
        let expected = Ok(0xD404);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_sl() {
        let input = Instruction::Group3 {
            op: Opecode3::Sl,
            rb: Register::One,
            constant: 4,
        };
        let expected = Ok(0x5104);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_sa() {
        let input = Instruction::Group3 {
            op: Opecode3::Sa,
            rb: Register::Two,
            constant: -2,
        };
        let expected = Ok(0x56FE);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_sc() {
        let input = Instruction::Group3 {
            op: Opecode3::Sc,
            rb: Register::Three,
            constant: 3,
        };
        let expected = Ok(0x5B03);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bix() {
        let input = Instruction::Group3 {
            op: Opecode3::Bix,
            rb: Register::Zero,
            constant: 4,
        };
        let expected = Ok(0xD804);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_lea() {
        let input = Instruction::Group4 {
            op: Opecode4::Lea,
            ra: Register::Two,
            rb: Register::One,
            constant: 4,
        };
        let expected = Ok(0xA904);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_lx() {
        let input = Instruction::Group4 {
            op: Opecode4::Lx,
            ra: Register::Three,
            rb: Register::One,
            constant: -1,
        };
        let expected = Ok(0xBDFF);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_stx() {
        let input = Instruction::Group4 {
            op: Opecode4::Stx,
            ra: Register::Three,
            rb: Register::One,
            constant: 2,
        };
        let expected = Ok(0xCD02);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_l() {
        let input = Instruction::Group5 {
            op: Opecode5::L,
            rb: Register::One,
            address: Address::Constant(5),
        };
        let expected = Ok(0x9105);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_st() {
        let input = Instruction::Group5 {
            op: Opecode5::St,
            rb: Register::Two,
            address: Address::Constant(-5i16 as MachineCode),
        };
        let expected = Ok(0x96FB);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_la() {
        let input = Instruction::Group5 {
            op: Opecode5::La,
            rb: Register::One,
            address: Address::Constant(4),
        };
        let expected = Ok(0x9904);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bdis() {
        let input = Instruction::Group6 {
            op: Opecode6::Bdis,
            address: Address::Constant(0),
        };
        let expected = Ok(0xDC00);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bp() {
        let input = Instruction::Group6 {
            op: Opecode6::Bp,
            address: Address::Constant(5),
        };
        let expected = Ok(0xE005);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bz() {
        let input = Instruction::Group6 {
            op: Opecode6::Bz,
            address: Address::Constant(-3i16 as MachineAddress),
        };
        let expected = Ok(0xE1FD);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bm() {
        let input = Instruction::Group6 {
            op: Opecode6::Bm,
            address: Address::Constant(9),
        };
        let expected = Ok(0xE209);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bc() {
        let input = Instruction::Group6 {
            op: Opecode6::Bc,
            address: Address::Unresolved {
                symbol_name: "TEST0",
                offset: 0,
            },
        };
        let expected = Err(());
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bnp() {
        let input = Instruction::Group6 {
            op: Opecode6::Bnp,
            address: Address::Unresolved {
                symbol_name: "TEST1",
                offset: 5,
            },
        };
        let expected = Err(());
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bnz() {
        let input = Instruction::Group6 {
            op: Opecode6::Bnz,
            address: Address::Unresolved {
                symbol_name: "TEST2",
                offset: -3,
            },
        };
        let expected = Err(());
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bnm() {
        let input = Instruction::Group6 {
            op: Opecode6::Bnm,
            address: Address::Unresolved {
                symbol_name: "TEST3",
                offset: 9,
            },
        };
        let expected = Err(());
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bnc() {
        let input = Instruction::Group6 {
            op: Opecode6::Bnc,
            address: Address::Constant(0),
        };
        let expected = Ok(0xE700);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_b() {
        let input = Instruction::Group6 {
            op: Opecode6::B,
            address: Address::Constant(-5i16 as MachineAddress),
        };
        let expected = Ok(0xE8FB);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bi() {
        let input = Instruction::Group6 {
            op: Opecode6::Bi,
            address: Address::Constant(3),
        };
        let expected = Ok(0xE903);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_bsr() {
        let input = Instruction::Group6 {
            op: Opecode6::Bsr,
            address: Address::Constant(9),
        };
        let expected = Ok(0xEA09);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_rio() {
        let input = Instruction::Group7 {
            op: Opecode7::Rio,
            device: Device::Cr,
        };
        let expected = Ok(0xEC00);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_wio() {
        let input = Instruction::Group7 {
            op: Opecode7::Wio,
            device: Device::Lpt,
        };
        let expected = Ok(0xED01);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_ret() {
        let input = Instruction::Group8 { op: Opecode8::Ret };
        let expected = Ok(0xEB00);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_nop() {
        let input = Instruction::Group8 { op: Opecode8::Nop };
        let expected = Ok(0xEE00);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_hlt() {
        let input = Instruction::Group8 { op: Opecode8::Hlt };
        let expected = Ok(0xEF00);
        let actual = MachineCode::try_from(&input);
        assert_eq!(expected, actual);
    }
}
