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

impl Default for ByteCodegen {
    fn default() -> Self {
        Self::new()
    }
}

impl ByteCodegen {
    #[must_use]
    pub const fn new() -> Self {
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
        let mut bc = Self::new();

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

    let input = StringInput::new(src, BytePos(0), BytePos(src.len() as u32));

    let c =  EsSyntax {
            jsx: false,
            fn_bind: false,
            decorators: true,
            decorators_before_export: true,
            export_default_from: true,
            import_attributes: true,
            allow_super_outside_method: false,
            allow_return_outside_function: false,
            auto_accessors: true,
            explicit_resource_management: true,
        };

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

    println!("{bc:?}");
}
