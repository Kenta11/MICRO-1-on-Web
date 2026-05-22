// Web wrapper around the upstream rm1masm modules (vendored from
// github.com/Kenta11/rm1masm v1.0.3).  Produces the .cm hex dump
// that m1sim.c can load directly.

mod codegen;
mod lexer;
mod parser;
mod symbol;
mod token;

use parser::MachineCode;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Default)]
struct AssembleResult {
    ok: bool,
    object_text: String,
    errors: Vec<String>,
    title: String,
    word_count: u32,
}

#[wasm_bindgen]
pub fn microassemble(source: &str) -> JsValue {
    let mut result = AssembleResult::default();

    let tokens = lexer::tokenize(source);
    let (ast, errs) = parser::parse(tokens);

    if !errs.is_empty() {
        for err in errs {
            let span = err.span();
            let snippet = source.get(span.clone()).unwrap_or("?");
            result
                .errors
                .push(format!("parse error near `{snippet}` (offset {}..{})", span.start, span.end));
        }
        return to_js(&result);
    }

    let ast = match ast {
        Some(a) => a,
        None => {
            result.errors.push("empty AST".into());
            return to_js(&result);
        }
    };

    let ast = match ast.set_address() {
        Ok(a) => a,
        Err(_) => {
            result.errors.push("illegal address found".into());
            return to_js(&result);
        }
    };

    let symbol_table = symbol::create_symbol_table(&ast.instructions);
    let unresolved = symbol::check_unresolved_symbols(&symbol_table, &ast.instructions);
    if !unresolved.is_empty() {
        for u in unresolved {
            result.errors.push(format!("unresolved symbol: {u}"));
        }
        return to_js(&result);
    }

    let code = match codegen::generate(&ast.instructions) {
        Ok(c) => c,
        Err(_) => {
            result.errors.push("code generation failed".into());
            return to_js(&result);
        }
    };

    let mut text = format!("CM {}", ast.title);
    let mut count = 0;
    for (address, instruction) in code.iter() {
        let resolved = match instruction.resolve(&symbol_table) {
            Ok(r) => r,
            Err(_) => {
                result.errors.push(format!("unresolved at {address:03X}"));
                return to_js(&result);
            }
        };
        match MachineCode::try_from(&resolved) {
            Ok(c) => {
                text.push_str(&format!("\n{address:03X}  {c:010X}"));
                count += 1;
            }
            Err(_) => {
                result.errors.push(format!("bad instruction at {address:03X}"));
                return to_js(&result);
            }
        }
    }
    text.push('\n');

    result.ok = true;
    result.title = ast.title.to_string();
    result.object_text = text;
    result.word_count = count;
    to_js(&result)
}

fn to_js<T: Serialize>(v: &T) -> JsValue {
    serde_wasm_bindgen::to_value(v).unwrap_or(JsValue::NULL)
}
