use std::path::Path;
use std::sync::LazyLock;
use swc_common::BytePos;
use swc_common::input::StringInput;
use swc_ecma_ast::Stmt;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::Realm;
use yavashark_env::scope::Scope;
use yavashark_interpreter::Interpreter;
use crate::TEST262_DIR;

const NON_RAW_HARNESS: [&str; 2] = ["harness/assert.js", "harness/sta.js"];


static COMPILED: LazyLock<Vec<Vec<Stmt>>> = LazyLock::new(|| {
    let p = Path::new(TEST262_DIR);
    
    
    NON_RAW_HARNESS.iter().map(|f| parse_file(
        p.join(
            Path::new(f)
        ).as_path())
    ).collect()
});


fn parse_file(f: &Path) -> Vec<Stmt> {
    let input = std::fs::read_to_string(f).unwrap();

    if input.is_empty() {
        return Vec::new()
    }

    let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32 - 1));

    let c = EsSyntax::default();

    let mut p = Parser::new(Syntax::Es(c), input, None);
    
    p.parse_script().unwrap().body
}



pub fn run_harness_in_realm(realm: &mut Realm, scope: &mut Scope) -> anyhow::Result<()> {
    for s in &*COMPILED {
        Interpreter::run_in(s, realm, scope)?;
    }
    
    Ok(())
}