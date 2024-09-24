#![allow(unused, clippy::needless_pass_by_ref_mut)] //pass by ref mut is just temporary until all functions are implemented

use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::{LabeledStmt, Stmt};
use swc_ecma_parser::{EsSyntax, Parser, Syntax};

use yavashark_bytecode::{ConstValue, Instruction};

pub type CompileError = anyhow::Error;
pub type Res = Result<(), CompileError>;

mod labels;
mod statement;
mod utils;

pub use labels::*;

#[derive(Debug)]
pub struct ByteCodegen {
    pub instructions: Vec<Instruction>,
    pub variables: Vec<String>,
    pub literals: Vec<ConstValue>,
    labels: Vec<(String, usize)>,
    loop_label: Option<usize>,
    label_backpatch: Vec<(LabelName, usize)>,
}


impl ByteCodegen {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            variables: vec![],
            literals: vec![],
            labels: vec![],
            loop_label: None,
            label_backpatch: vec![],
        }
    }
    
    
    pub fn compile(script: &Vec<Stmt>) -> Result<Self, CompileError> {
        let mut bc = ByteCodegen::new();
        
        bc.compile_statements(script)?;
        
        Ok(bc)
    }
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
        label_backpatch: vec![],
    };

    bc.compile_statements(&script.body);
    
    println!("{:?}", bc);
}
