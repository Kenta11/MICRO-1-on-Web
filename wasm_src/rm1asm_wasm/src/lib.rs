// Web wrapper around the upstream rm1asm modules (vendored from
// github.com/Kenta11/rm1asm v1.0.3).  Exposes a single JS-callable
// entry point `assemble(source)` that returns the `.b` object text
// plus any error diagnostics.

mod codegen;
mod instruction;
mod lexer;
mod parser;
mod symbol;
mod token;

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
pub fn assemble(source: &str) -> JsValue {
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

    let mut ast = match ast {
        Some(a) => a,
        None => {
            result.errors.push("empty AST".to_string());
            return to_js(&result);
        }
    };

    symbol::resolve_symbols(&mut ast.lines);
    let unresolved = symbol::check_unresolve_symbols(&ast.lines);
    if !unresolved.is_empty() {
        for sym in unresolved {
            result.errors.push(format!("unresolved symbol: {sym}"));
        }
        return to_js(&result);
    }

    let code = codegen::generate(&ast.lines);
    let mut text = format!("MM {}", ast.title);
    for (a, c) in &code {
        text.push_str(&format!("\n{a:04X}  {c:04X}"));
    }
    text.push('\n');

    result.ok = true;
    result.title = ast.title.to_string();
    result.object_text = text;
    result.word_count = code.len() as u32;
    to_js(&result)
}

fn to_js<T: Serialize>(v: &T) -> JsValue {
    serde_wasm_bindgen::to_value(v).unwrap_or(JsValue::NULL)
}
