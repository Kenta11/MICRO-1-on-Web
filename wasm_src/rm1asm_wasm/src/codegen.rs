use crate::instruction::{Instruction, MachineAddress, MachineCode};
use crate::parser::ProgramLine;

pub fn generate(lines: &Vec<ProgramLine>) -> Vec<(MachineAddress, MachineCode)> {
    let mut ret = Vec::<(MachineAddress, MachineCode)>::new();

    let mut address: MachineAddress = 0;
    for line in lines {
        if let Some(instruction) = &line.instruction {
            if let Instruction::Org(constant) = &instruction {
                address = *constant;
            } else if let Some(Instruction::Ds(s)) = &line.instruction {
                for _ in 0..*s {
                    ret.push((address, 0));
                    address += 1;
                }
            } else {
                match MachineCode::try_from(instruction) {
                    Ok(c) => ret.push((address, c)),
                    _ => panic!("{:?}: Unexpected instruction", instruction),
                }
                address += 1;
            }
        }
    }

    ret
}
