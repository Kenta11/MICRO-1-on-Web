use crate::parser::{Instruction, MachineAddress, TestAndSequenceStatement};

use std::collections::{HashMap, HashSet};

pub type SymbolTable<'a> = HashMap<&'a str, MachineAddress>;

pub fn create_symbol_table<'a>(instructions: &'a Vec<Instruction>) -> SymbolTable<'a> {
    let mut symbol_table = SymbolTable::new();

    for instruction in instructions {
        if let Some(label) = instruction.label {
            if let Some(address) = instruction.address {
                if !symbol_table.contains_key(label) {
                    symbol_table.entry(label).or_insert(address);
                }
            }
        }
    }

    symbol_table
}

pub fn check_unresolved_symbols<'a>(
    symbol_table: &SymbolTable,
    instructions: &'a Vec<Instruction>,
) -> HashSet<&'a str> {
    let mut unresolved_symbols = HashSet::<&str>::new();

    for instruction in instructions {
        if let Some(test_and_sequence_statement) = &instruction.test_and_sequence_statement {
            match test_and_sequence_statement {
                TestAndSequenceStatement::Goto(symbol) => {
                    if *symbol != "FETCH" && !symbol_table.contains_key(symbol) {
                        unresolved_symbols.insert(symbol);
                    }
                }
                TestAndSequenceStatement::Call(symbol) => {
                    if !symbol_table.contains_key(symbol) {
                        unresolved_symbols.insert(symbol);
                    }
                }
                TestAndSequenceStatement::Iop(symbol) => {
                    if !symbol_table.contains_key(symbol) {
                        unresolved_symbols.insert(symbol);
                    }
                }
                TestAndSequenceStatement::Ira(symbol) => {
                    if !symbol_table.contains_key(symbol) {
                        unresolved_symbols.insert(symbol);
                    }
                }
                TestAndSequenceStatement::Iab(symbol) => {
                    if !symbol_table.contains_key(symbol) {
                        unresolved_symbols.insert(symbol);
                    }
                }
                _ => (),
            }
        }
    }

    unresolved_symbols
}
