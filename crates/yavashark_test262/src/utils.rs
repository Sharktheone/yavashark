use std::path::Path;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::Stmt;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};

pub(crate) fn parse_file(f: &Path) -> Vec<Stmt> {
    let input = std::fs::read_to_string(f).unwrap();

    if input.is_empty() {
        return Vec::new();
    }

    let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32 - 1));

    let c = EsSyntax::default();

    let mut p = Parser::new(Syntax::Es(c), input, None);

    p.parse_script().unwrap().body
}
