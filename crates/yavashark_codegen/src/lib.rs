#![allow(unused, clippy::needless_pass_by_ref_mut)] //pass by ref mut is just temporary until all functions are implemented

use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::LabeledStmt;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};

use yavashark_bytecode::{ConstValue, Instruction};

pub type CompileError = anyhow::Error;
pub type Res = Result<(), CompileError>;

mod statement;
mod utils;
mod labels;

pub use labels::*;

struct ByteCodegen {
    instructions: Vec<Instruction>,
    variables: Vec<String>,
    literals: Vec<ConstValue>,
    labels: Vec<(String, usize)>,
    loop_label: Option<usize>,
    label_backpatch: Vec<(LabelName, usize)>,
}


#[test]
fn test_compile() {
    let src = r#"
    console.log("Hello, World!");
    if (true) {
        console.log("True");
    } else {
        console.log("False");
    }
 "#;

    let input = StringInput::new(src.into(), BytePos(0), BytePos(src.len() as u32));

    let c = EsSyntax::default();

    let mut p = Parser::new(Syntax::Es(c), input, None);
    let script = p.parse_script().unwrap();

    let mut bc = ByteCodegen {
        instructions: vec![],
        variables: vec![],
        literals: vec![],
        labels: vec![],
        loop_label: None,
        label_backpatch: vec![]
    };

    bc.compile_statements(&script.body);
}
