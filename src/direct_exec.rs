use std::path::PathBuf;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::Stmt;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, ValueResult};
use yavashark_interpreter::Interpreter;

fn parse(input: &str) -> Res<Vec<Stmt>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32));

    let c = EsSyntax::default();

    let mut p = Parser::new(Syntax::Es(c), input, None);

    let script = p
        .parse_script()
        .map_err(|e| Error::syn_error(format!("{e:?}")))?;

    Ok(script.body)
}

pub fn execute(code: &str) -> ValueResult {
    let mut realm = Realm::new()?;
    let mut scope = Scope::global(&realm, PathBuf::new());

    Interpreter::run_in(&parse(code)?, &mut realm, &mut scope)
}

pub fn execute_fmt(code: &str) -> Res<String> {
    let mut realm = Realm::new()?;
    let mut scope = Scope::global(&realm, PathBuf::new());

    Interpreter::run_in(&parse(code)?, &mut realm, &mut scope)
        .and_then(|v| Ok(v.to_string(&mut realm)?.to_string()))
}
