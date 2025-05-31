use crate::{execute, execute_fmt};
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

#[wasm_bindgen]
pub fn run_standalone(code: &str) -> String {
    match execute_fmt(code) {
        Ok(v) => v,
        Err(e) => {
            format!("Error: {:?}", e)
        }
    }
}
