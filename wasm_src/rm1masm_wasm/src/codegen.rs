use crate::parser::*;
use crate::symbol::SymbolTable;

#[derive(Clone, Copy, PartialEq)]
enum Lb {
    R0l,
    R1l,
    R2l,
    R3l,
    R4l,
    R5l,
    R6l,
    R7l,
    Rb,
    Rbp,
    Pc,
    Io,
    Mm,
    Ir,
    Fsr,
    Nlb,
}

impl From<&Lbus> for Lb {
    fn from(from: &Lbus) -> Self {
        match from {
            Lbus::R0 => Lb::R0l,
            Lbus::R1 => Lb::R1l,
            Lbus::R2 => Lb::R2l,
            Lbus::R3 => Lb::R3l,
            Lbus::R4 => Lb::R4l,
            Lbus::R5 => Lb::R5l,
            Lbus::R6 => Lb::R6l,
            Lbus::R7 => Lb::R7l,
            Lbus::Rb => Lb::Rb,
            Lbus::Rbp => Lb::Rbp,
            Lbus::Pc => Lb::Pc,
            Lbus::Io => Lb::Io,
            Lbus::Mm => Lb::Mm,
            Lbus::Ir => Lb::Ir,
            Lbus::Fsr => Lb::Fsr,
            Lbus::Zero => Lb::Nlb,
        }
    }
}

impl From<&Lb> for MachineCode {
    fn from(from: &Lb) -> Self {
        (match from {
            Lb::R0l => 0b0000,
            Lb::R1l => 0b0001,
            Lb::R2l => 0b0010,
            Lb::R3l => 0b0011,
            Lb::R4l => 0b0100,
            Lb::R5l => 0b0101,
            Lb::R6l => 0b0110,
            Lb::R7l => 0b0111,
            Lb::Rb => 0b1000,
            Lb::Rbp => 0b1001,
            Lb::Pc => 0b1010,
            Lb::Io => 0b1011,
            Lb::Mm => 0b1100,
            Lb::Ir => 0b1101,
            Lb::Fsr => 0b1110,
            Lb::Nlb => 0b1111,
        }) << (4 + 3 + 3 + 4 + 2 + 4 + 3 + 4 + 9)
    }
}

type Llt = u16;
type Slt = u16;

#[derive(Clone, Copy, PartialEq)]
enum Rb {
    R0r,
    R1r,
    R2r,
    R3r,
    R4r,
    R5r,
    R6r,
    R7r,
    Ra,
    Rap,
    Slt(Slt),
    Llt(Llt),
    Nrb,
}

impl From<&Rbus> for Rb {
    fn from(from: &Rbus) -> Self {
        match from {
            Rbus::R0 => Rb::R0r,
            Rbus::R1 => Rb::R1r,
            Rbus::R2 => Rb::R2r,
            Rbus::R3 => Rb::R3r,
            Rbus::R4 => Rb::R4r,
            Rbus::R5 => Rb::R5r,
            Rbus::R6 => Rb::R6r,
            Rbus::R7 => Rb::R7r,
            Rbus::Ra => Rb::Ra,
            Rbus::Rap => Rb::Rap,
            Rbus::Literal(literal) => {
                if *literal == 0 {
                    Rb::Nrb
                } else if *literal < 512u16 {
                    Rb::Slt(*literal)
                } else {
                    Rb::Llt(*literal)
                }
            }
        }
    }
}

impl From<&Rb> for MachineCode {
    fn from(from: &Rb) -> Self {
        (match from {
            Rb::R0r => 0b0000,
            Rb::R1r => 0b0001,
            Rb::R2r => 0b0010,
            Rb::R3r => 0b0011,
            Rb::R4r => 0b0100,
            Rb::R5r => 0b0101,
            Rb::R6r => 0b0110,
            Rb::R7r => 0b0111,
            Rb::Ra => 0b1000,
            Rb::Rap => 0b1001,
            Rb::Slt(_) => 0b1010,
            Rb::Llt(_) => 0b1011,
            Rb::Nrb => 0b1111,
        }) << (3 + 3 + 4 + 2 + 4 + 3 + 4 + 9)
    }
}

#[derive(Clone, Copy)]
enum Al {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Ial,
    Nal,
}

impl From<&Alu> for Al {
    fn from(from: &Alu) -> Self {
        match from {
            Alu::Plus => Al::Add,
            Alu::Minus => Al::Sub,
            Alu::And => Al::And,
            Alu::Or => Al::Or,
            Alu::Xor => Al::Xor,
            Alu::Dollar => Al::Ial,
            Alu::At => Al::Nal,
        }
    }
}

impl From<&Al> for MachineCode {
    fn from(from: &Al) -> Self {
        (match from {
            Al::Add => 0b000,
            Al::Sub => 0b001,
            Al::And => 0b010,
            Al::Or => 0b011,
            Al::Xor => 0b100,
            Al::Ial => 0b101,
            Al::Nal => 0b111,
        }) << (3 + 4 + 2 + 4 + 3 + 4 + 9)
    }
}

#[derive(Clone, Copy)]
enum Sh {
    Sll,
    Srl,
    Sla,
    Sra,
    Snx,
    Swp,
    Nsh,
}

impl From<&Option<Shift>> for Sh {
    fn from(from: &Option<Shift>) -> Self {
        if let Some(value) = from {
            match value {
                Shift::Sll => Sh::Sll,
                Shift::Srl => Sh::Srl,
                Shift::Sla => Sh::Sla,
                Shift::Sra => Sh::Sra,
                Shift::Snx => Sh::Snx,
                Shift::Swp => Sh::Swp,
                Shift::Nsb => Sh::Nsh,
            }
        } else {
            Sh::Nsh
        }
    }
}

impl From<&Sh> for MachineCode {
    fn from(from: &Sh) -> Self {
        (match from {
            Sh::Sll => 0b000,
            Sh::Srl => 0b001,
            Sh::Sla => 0b010,
            Sh::Sra => 0b011,
            Sh::Snx => 0b100,
            Sh::Swp => 0b101,
            Sh::Nsh => 0b111,
        }) << (4 + 2 + 4 + 3 + 4 + 9)
    }
}

#[derive(Clone, Copy)]
enum Sb {
    R0s,
    R1s,
    R2s,
    R3s,
    R4s,
    R5s,
    R6s,
    R7s,
    Sa,
    Sap,
    Sb,
    Sbp,
    Pcs,
    Nsb,
}

impl From<&Sbus> for Sb {
    fn from(from: &Sbus) -> Self {
        match from {
            Sbus::R0 => Sb::R0s,
            Sbus::R1 => Sb::R1s,
            Sbus::R2 => Sb::R2s,
            Sbus::R3 => Sb::R3s,
            Sbus::R4 => Sb::R4s,
            Sbus::R5 => Sb::R5s,
            Sbus::R6 => Sb::R6s,
            Sbus::R7 => Sb::R7s,
            Sbus::Ra => Sb::Sa,
            Sbus::Rap => Sb::Sap,
            Sbus::Rb => Sb::Sb,
            Sbus::Rbp => Sb::Sbp,
            Sbus::Pc => Sb::Pcs,
        }
    }
}

impl From<&Sb> for MachineCode {
    fn from(from: &Sb) -> Self {
        (match from {
            Sb::R0s => 0b0000,
            Sb::R1s => 0b0001,
            Sb::R2s => 0b0010,
            Sb::R3s => 0b0011,
            Sb::R4s => 0b0100,
            Sb::R5s => 0b0101,
            Sb::R6s => 0b0110,
            Sb::R7s => 0b0111,
            Sb::Sa => 0b1000,
            Sb::Sap => 0b1001,
            Sb::Sb => 0b1010,
            Sb::Sbp => 0b1011,
            Sb::Pcs => 0b1100,
            Sb::Nsb => 0b1111,
        }) << (2 + 4 + 3 + 4 + 9)
    }
}

struct Calculation(Lb, Rb, Al, Sh, Sb);

impl From<&Option<CalculationStatement>> for Calculation {
    fn from(from: &Option<CalculationStatement>) -> Self {
        if let Some(value) = from {
            match value {
                CalculationStatement::Alu(sbus, statement) => match statement {
                    Statement::First(lbus, alu_and_shift, rbus) => Calculation(
                        Lb::from(lbus),
                        Rb::from(rbus),
                        Al::from(&alu_and_shift.alu),
                        Sh::from(&alu_and_shift.shift),
                        Sb::from(sbus),
                    ),
                    Statement::AluThrough(alu_through) => match &alu_through.lbus_or_rbus {
                        LbusOrRbus::Lbus(lbus) => Calculation(
                            Lb::from(lbus),
                            Rb::Nrb,
                            Al::Nal,
                            Sh::from(&alu_through.shift),
                            Sb::from(sbus),
                        ),
                        LbusOrRbus::Rbus(rbus) => Calculation(
                            Lb::Nlb,
                            Rb::from(rbus),
                            Al::Nal,
                            Sh::from(&alu_through.shift),
                            Sb::from(sbus),
                        ),
                    },
                },
                CalculationStatement::Set(statement) => match statement {
                    Statement::First(lbus, alu_and_shift, rbus) => Calculation(
                        Lb::from(lbus),
                        Rb::from(rbus),
                        Al::from(&alu_and_shift.alu),
                        Sh::from(&alu_and_shift.shift),
                        Sb::Nsb,
                    ),
                    Statement::AluThrough(alu_through) => match &alu_through.lbus_or_rbus {
                        LbusOrRbus::Lbus(lbus) => Calculation(
                            Lb::from(lbus),
                            Rb::Nrb,
                            Al::Or,
                            Sh::from(&alu_through.shift),
                            Sb::Nsb,
                        ),
                        LbusOrRbus::Rbus(rbus) => Calculation(
                            Lb::Nlb,
                            Rb::from(rbus),
                            Al::Or,
                            Sh::from(&alu_through.shift),
                            Sb::Nsb,
                        ),
                    },
                },
            }
        } else {
            Calculation(Lb::Nlb, Rb::Nrb, Al::Nal, Sh::Nsh, Sb::Nsb)
        }
    }
}

#[derive(Clone, Copy)]
enum Mm {
    Rm,
    Wm,
    Nmm,
}

impl From<&Option<MemoryStatement>> for Mm {
    fn from(from: &Option<MemoryStatement>) -> Self {
        if let Some(value) = from {
            match value {
                MemoryStatement::Read => Mm::Rm,
                MemoryStatement::Write => Mm::Wm,
            }
        } else {
            Mm::Nmm
        }
    }
}

impl From<&Mm> for MachineCode {
    fn from(from: &Mm) -> Self {
        (match from {
            Mm::Rm => 0b00,
            Mm::Wm => 0b01,
            Mm::Nmm => 0b11,
        }) << (4 + 3 + 4 + 9)
    }
}

#[derive(Clone, Copy)]
pub enum Sq {
    B,
    Bp,
    Rtn,
    Bt,
    Bf,
    Iop,
    Ira,
    Iab,
    Ei,
    Nsq,
}

impl From<&Sq> for MachineCode {
    fn from(from: &Sq) -> Self {
        (match from {
            Sq::B => 0b0000,
            Sq::Bp => 0b0001,
            Sq::Rtn => 0b0010,
            Sq::Bt => 0b0011,
            Sq::Bf => 0b0100,
            Sq::Iop => 0b0101,
            Sq::Ira => 0b0110,
            Sq::Iab => 0b0111,
            Sq::Ei => 0b1000,
            Sq::Nsq => 0b1111,
        }) << (3 + 4 + 9)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Ts {
    Zer,
    Neg,
    Cry,
    Ov,
    T,
    Cz,
    Nts,
}

impl From<&Ts> for MachineCode {
    fn from(from: &Ts) -> Self {
        (match from {
            Ts::Zer => 0b000,
            Ts::Neg => 0b001,
            Ts::Cry => 0b010,
            Ts::Ov => 0b011,
            Ts::T => 0b100,
            Ts::Cz => 0b101,
            Ts::Nts => 0b111,
        }) << (4 + 9)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Ex {
    Cm1,
    Fls,
    Asc,
    As1,
    Lir(Option<Lb>),
    Lio(Option<Lb>),
    Sc(Option<Rb>),
    Eio,
    St,
    Rt,
    Ina,
    Inb,
    Dcb,
    Hlt,
    Ov,
    Nex,
}

impl From<&Option<ExStatement>> for Ex {
    fn from(from: &Option<ExStatement>) -> Self {
        if let Some(value) = from {
            match value {
                ExStatement::DecrimentC => Ex::Cm1,
                ExStatement::FlagSave => Ex::Fls,
                ExStatement::WithCry => Ex::Asc,
                ExStatement::WithOne => Ex::As1,
                ExStatement::Ir(lbus) => Ex::Lir(match lbus {
                    LbusSource::Lbus => None,
                    LbusSource::Source(src) => Some(Lb::from(src)),
                }),
                ExStatement::Io(lbus) => Ex::Lio(match lbus {
                    LbusSource::Lbus => None,
                    LbusSource::Source(src) => Some(Lb::from(src)),
                }),
                ExStatement::C(rbus) => Ex::Sc(match rbus {
                    RbusSource::Rbus => None,
                    RbusSource::Source(src) => Some(Rb::from(src)),
                }),
                ExStatement::ExecuteIo => Ex::Eio,
                ExStatement::Assign1ToT => Ex::St,
                ExStatement::Assign0ToT => Ex::Rt,
                ExStatement::IraPlus1 => Ex::Ina,
                ExStatement::IrbPlus1 => Ex::Inb,
                ExStatement::IrbMinus1 => Ex::Dcb,
                ExStatement::SetHlt => Ex::Hlt,
                ExStatement::SetOv => Ex::Ov,
                ExStatement::Nex => Ex::Nex,
            }
        } else {
            Ex::Nex
        }
    }
}

impl From<&Ex> for MachineCode {
    fn from(from: &Ex) -> Self {
        (match from {
            Ex::Cm1 => 0b0000,
            Ex::Fls => 0b0001,
            Ex::Asc => 0b0010,
            Ex::As1 => 0b0011,
            Ex::Lir(_) => 0b0100,
            Ex::Lio(_) => 0b0101,
            Ex::Sc(_) => 0b0110,
            Ex::Eio => 0b0111,
            Ex::St => 0b1000,
            Ex::Rt => 0b1001,
            Ex::Ina => 0b1010,
            Ex::Inb => 0b1011,
            Ex::Dcb => 0b1100,
            Ex::Hlt => 0b1101,
            Ex::Ov => 0b1110,
            Ex::Nex => 0b1111,
        }) << 9
    }
}

struct TestAndSequence<'a>(Ts, Sq, Option<&'a str>);

impl<'a> From<&Option<TestAndSequenceStatement<'a>>> for TestAndSequence<'a> {
    fn from(from: &Option<TestAndSequenceStatement<'a>>) -> Self {
        if let Some(value) = from {
            match value {
                TestAndSequenceStatement::Goto(label) => {
                    if *label == "FETCH" {
                        TestAndSequence(Ts::Nts, Sq::Ei, None)
                    } else {
                        TestAndSequence(Ts::Nts, Sq::B, Some(label))
                    }
                }
                TestAndSequenceStatement::Call(label) => {
                    TestAndSequence(Ts::Nts, Sq::Bp, Some(label))
                }
                TestAndSequenceStatement::Return => TestAndSequence(Ts::Nts, Sq::Rtn, None),
                TestAndSequenceStatement::If(flag, flag_state, label, else_exists) => {
                    let ts = match flag {
                        Flag::Zer => Ts::Zer,
                        Flag::Neg => Ts::Neg,
                        Flag::Cry => Ts::Cry,
                        Flag::Ov => Ts::Ov,
                        Flag::Cz => Ts::Cz,
                        Flag::T => Ts::T,
                    };
                    if *else_exists {
                        TestAndSequence(ts, Sq::Ei, Some(label))
                    } else if *flag_state == FlagState::One {
                        TestAndSequence(ts, Sq::Bt, Some(label))
                    } else {
                        TestAndSequence(ts, Sq::Bf, Some(label))
                    }
                }
                TestAndSequenceStatement::Iop(label) => {
                    TestAndSequence(Ts::Nts, Sq::Iop, Some(label))
                }
                TestAndSequenceStatement::Ira(label) => {
                    TestAndSequence(Ts::Nts, Sq::Ira, Some(label))
                }
                TestAndSequenceStatement::Iab(label) => {
                    TestAndSequence(Ts::Nts, Sq::Iab, Some(label))
                }
                TestAndSequenceStatement::Nsq => TestAndSequence(Ts::Nts, Sq::Nsq, None),
            }
        } else {
            TestAndSequence(Ts::Nts, Sq::Nsq, None)
        }
    }
}

#[derive(Clone, Copy)]
pub struct MicroInstruction<'a> {
    lb: Lb,
    rb: Rb,
    al: Al,
    sh: Sh,
    sb: Sb,
    mm: Mm,
    pub sq: Sq,
    lsb: MicroInstructionLSB16<'a>,
}

impl<'a> MicroInstruction<'a> {
    pub fn resolve(&'a self, table: &SymbolTable) -> Result<Self, ()> {
        match self.lsb.resolve(table) {
            Ok(lsb) => Ok(Self { lsb, ..*self }),
            Err(_) => Err(()),
        }
    }
}

#[derive(Clone, Copy)]
enum MicroInstructionLSB16<'a> {
    Llt(Llt),
    TsAndEx(Ts, Ex),
    TsExAndLt(Ts, Ex, Slt),
    WithReference(Ts, Ex, Reference<'a>),
}

impl<'a> MicroInstructionLSB16<'a> {
    pub fn resolve(&'a self, table: &SymbolTable) -> Result<Self, ()> {
        match self {
            MicroInstructionLSB16::WithReference(ts, ex, reference) => {
                if let Ok(reference) = reference.resolve(table) {
                    Ok(MicroInstructionLSB16::WithReference(*ts, *ex, reference))
                } else {
                    Err(())
                }
            }
            _ => Ok(*self),
        }
    }
}

#[derive(Clone, Copy)]
enum Reference<'a> {
    Unresolved(&'a str),
    Resolved(u16),
}

impl<'a> Reference<'a> {
    pub fn resolve(&'a self, table: &SymbolTable) -> Result<Self, ()> {
        match self {
            Reference::Unresolved(label) => {
                if let Some(address) = table.get(label) {
                    Ok(Reference::Resolved(*address))
                } else {
                    Err(())
                }
            }
            Reference::Resolved(_) => Ok(*self),
        }
    }
}

impl<'a> TryFrom<&'a Instruction<'a>> for MicroInstruction<'a> {
    type Error = ();

    fn try_from(value: &Instruction<'a>) -> Result<Self, Self::Error> {
        let TestAndSequence(ts, sq, reference) =
            TestAndSequence::from(&value.test_and_sequence_statement);
        let mm = Mm::from(&value.memory_statement);
        let Calculation(lb, rb, al, sh, sb) = Calculation::from(&value.calculation_statement);
        let ex = Ex::from(&value.ex_statement);

        if let Rb::Llt(literal) = rb {
            if matches!(ts, Ts::Nts) && reference.is_none() && matches!(ex, Ex::Nex) {
                Ok(Self {
                    lb,
                    rb,
                    al,
                    sh,
                    sb,
                    mm,
                    sq,
                    lsb: MicroInstructionLSB16::Llt(literal),
                })
            } else {
                Err(())
            }
        } else if let Rb::Slt(literal) = rb {
            if reference.is_none() {
                Ok(Self {
                    lb,
                    rb,
                    al,
                    sh,
                    sb,
                    mm,
                    sq,
                    lsb: MicroInstructionLSB16::TsExAndLt(ts, ex, literal),
                })
            } else {
                Err(())
            }
        } else if let Ex::Lir(lbus) = ex {
            if let Some(src) = lbus {
                if value.calculation_statement.is_none() {
                    Ok(Self {
                        lb: src,
                        rb,
                        al,
                        sh,
                        sb,
                        mm,
                        sq,
                        lsb: if let Some(reference) = reference {
                            MicroInstructionLSB16::WithReference(
                                ts,
                                ex,
                                Reference::Unresolved(reference),
                            )
                        } else {
                            MicroInstructionLSB16::TsAndEx(ts, ex)
                        },
                    })
                } else {
                    Err(())
                }
            } else {
                Ok(Self {
                    lb,
                    rb,
                    al,
                    sh,
                    sb,
                    mm,
                    sq,
                    lsb: if let Some(reference) = reference {
                        MicroInstructionLSB16::WithReference(
                            ts,
                            ex,
                            Reference::Unresolved(reference),
                        )
                    } else {
                        MicroInstructionLSB16::TsAndEx(ts, ex)
                    },
                })
            }
        } else if let Ex::Lio(lbus) = ex {
            if let Some(src) = lbus {
                if value.calculation_statement.is_none() {
                    Ok(Self {
                        lb: src,
                        rb,
                        al,
                        sh,
                        sb,
                        mm,
                        sq,
                        lsb: if let Some(reference) = reference {
                            MicroInstructionLSB16::WithReference(
                                ts,
                                ex,
                                Reference::Unresolved(reference),
                            )
                        } else {
                            MicroInstructionLSB16::TsAndEx(ts, ex)
                        },
                    })
                } else {
                    Err(())
                }
            } else {
                Ok(Self {
                    lb,
                    rb,
                    al,
                    sh,
                    sb,
                    mm,
                    sq,
                    lsb: if let Some(reference) = reference {
                        MicroInstructionLSB16::WithReference(
                            ts,
                            ex,
                            Reference::Unresolved(reference),
                        )
                    } else {
                        MicroInstructionLSB16::TsAndEx(ts, ex)
                    },
                })
            }
        } else if let Ex::Sc(Some(src)) = ex {
            if value.calculation_statement.is_none() {
                match src {
                    Rb::Llt(literal) => {
                        if matches!(ts, Ts::Nts) && reference.is_none() && matches!(ex, Ex::Nex) {
                            Ok(Self {
                                lb,
                                rb: src,
                                al,
                                sh,
                                sb,
                                mm,
                                sq,
                                lsb: MicroInstructionLSB16::Llt(literal),
                            })
                        } else {
                            Err(())
                        }
                    }
                    Rb::Slt(literal) => {
                        if reference.is_none() {
                            Ok(Self {
                                lb,
                                rb: src,
                                al,
                                sh,
                                sb,
                                mm,
                                sq,
                                lsb: MicroInstructionLSB16::TsExAndLt(ts, ex, literal),
                            })
                        } else {
                            Err(())
                        }
                    }
                    _ => Ok(Self {
                        lb,
                        rb: src,
                        al,
                        sh,
                        sb,
                        mm,
                        sq,
                        lsb: if let Some(reference) = reference {
                            MicroInstructionLSB16::WithReference(
                                ts,
                                ex,
                                Reference::Unresolved(reference),
                            )
                        } else {
                            MicroInstructionLSB16::TsAndEx(ts, ex)
                        },
                    }),
                }
            } else {
                Err(())
            }
        } else {
            Ok(Self {
                lb,
                rb,
                al,
                sh,
                sb,
                mm,
                sq,
                lsb: if let Some(reference) = reference {
                    MicroInstructionLSB16::WithReference(ts, ex, Reference::Unresolved(reference))
                } else {
                    MicroInstructionLSB16::TsAndEx(ts, ex)
                },
            })
        }
    }
}

impl<'a> TryFrom<&MicroInstruction<'a>> for MachineCode {
    type Error = ();

    fn try_from(value: &MicroInstruction) -> Result<Self, Self::Error> {
        match value.lsb {
            MicroInstructionLSB16::Llt(literal) => Ok(MachineCode::from(&value.lb)
                | MachineCode::from(&value.rb)
                | MachineCode::from(&value.al)
                | MachineCode::from(&value.sh)
                | MachineCode::from(&value.sb)
                | MachineCode::from(&value.mm)
                | MachineCode::from(&value.sq)
                | literal as MachineCode),
            MicroInstructionLSB16::TsAndEx(ts, ex) => Ok(MachineCode::from(&value.lb)
                | MachineCode::from(&value.rb)
                | MachineCode::from(&value.al)
                | MachineCode::from(&value.sh)
                | MachineCode::from(&value.sb)
                | MachineCode::from(&value.mm)
                | MachineCode::from(&value.sq)
                | MachineCode::from(&ts)
                | MachineCode::from(&ex)),
            MicroInstructionLSB16::TsExAndLt(ts, ex, slt) => Ok(MachineCode::from(&value.lb)
                | MachineCode::from(&value.rb)
                | MachineCode::from(&value.al)
                | MachineCode::from(&value.sh)
                | MachineCode::from(&value.sb)
                | MachineCode::from(&value.mm)
                | MachineCode::from(&value.sq)
                | MachineCode::from(&ts)
                | MachineCode::from(&ex)
                | slt as MachineCode),
            MicroInstructionLSB16::WithReference(ts, ex, reference) => match reference {
                Reference::Unresolved(_) => Err(()),
                Reference::Resolved(literal) => Ok(MachineCode::from(&value.lb)
                    | MachineCode::from(&value.rb)
                    | MachineCode::from(&value.al)
                    | MachineCode::from(&value.sh)
                    | MachineCode::from(&value.sb)
                    | MachineCode::from(&value.mm)
                    | MachineCode::from(&value.sq)
                    | match value.sq {
                        Sq::B | Sq::Bp => ((literal as MachineCode) & 0xE00) << 4,
                        Sq::Ira | Sq::Iab => ((literal as MachineCode) & 0x800) << 2,
                        _ => MachineCode::from(&ts),
                    }
                    | MachineCode::from(&ex)
                    | match value.sq {
                        Sq::Iop => (literal as MachineCode) >> 4,
                        Sq::Ira => {
                            (((literal as MachineCode) & 0x7F0) >> 2)
                                + ((literal as MachineCode) & 0x003)
                        }
                        Sq::Iab => {
                            (((literal as MachineCode) & 0x7C0) >> 2)
                                + ((literal as MachineCode) & 0x003)
                        }
                        _ => (literal as MachineCode) & 0x1FF,
                    }),
            },
        }
    }
}

pub fn generate<'a>(
    instructions: &'a Vec<Instruction<'a>>,
) -> Result<Vec<(MachineAddress, MicroInstruction<'a>)>, ()> {
    let mut ret = Vec::<(MachineAddress, MicroInstruction)>::new();

    for instruction in instructions {
        let address = if let Some(constant) = instruction.address {
            constant
        } else {
            return Err(());
        };

        let code = match MicroInstruction::try_from(instruction) {
            Ok(code) => code,
            Err(_) => {
                return Err(());
            }
        };

        ret.push((address, code));
    }

    Ok(ret)
}
