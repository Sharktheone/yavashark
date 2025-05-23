use std::path::{Path, PathBuf};
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::Stmt;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use wasm_bindgen::prelude::wasm_bindgen;
use yavashark_env::scope::Scope;
use yavashark_env::Realm;
use yavashark_interpreter::Interpreter;

#[wasm_bindgen(start)]
fn init() {
    console_error_panic_hook::set_once();
    console_log::init().expect("could not initialize logger");
}

fn parse(input: &str) -> Vec<Stmt> {
    if input.is_empty() {
        return Vec::new();
    }

    let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32 - 1));

    let c = EsSyntax::default();

    let mut p = Parser::new(Syntax::Es(c), input, None);

    let script = p.parse_script().unwrap();

    script.body
}

#[wasm_bindgen]
pub fn run_standalone(code: &str) -> String {
    let mut realm = Realm::new().unwrap();
    let mut scope = Scope::global(&realm, PathBuf::new());

    let res = Interpreter::run_in(&parse(code), &mut realm, &mut scope);

    match res {
        Ok(v) => v.to_string(&mut realm).unwrap(),
        Err(e) => e.to_string(),
    }
}
