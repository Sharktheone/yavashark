use std::cell::RefCell;
use std::rc::Rc;
use swc_ecma_ast::{Function, Stmt};

pub use yavashark_bytecode as bytecode;
use yavashark_bytecode::BytecodeFunctionCode;
pub use yavashark_codegen as codegen;
pub use yavashark_vm as vm;

use yavashark_codegen::ByteCodegen;
use yavashark_compiler::Compiler;
use yavashark_env::optimizer::{FunctionCode, OptimFunction};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, ObjectHandle, Realm, Res, ValueResult};
use yavashark_vm::OldBorrowedVM;
use yavashark_vm::function_code::{BytecodeFunction};
use yavashark_vm::yavashark_bytecode::data::DataSection;

pub struct ByteCodeInterpreter;

impl ByteCodeInterpreter {
    pub fn run_in(script: &Vec<Stmt>, realm: &mut Realm, scope: &mut Scope) -> ValueResult {
        let code = ByteCodegen::compile(script)
            .map_err(|e| Error::new_error(format!("Failed to compile: {e:?}")))?;

        let ds = DataSection::new(code.variables, Vec::new(), code.literals);

        let mut vm = OldBorrowedVM::with_scope(&code.instructions, &ds, realm, scope.clone());

        vm.run_ret()
    }

    pub fn compile_fn(
        func: &Function,
        name: String,
        scope: Scope,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let mut compiled: Option<RefCell<Box<dyn FunctionCode + 'static>>> = None;
        if let Some(body) = &func.body {
            let code = Compiler::compile(&body.stmts)
                .map_err(|e| Error::syn_error(format!("Failed to compile: {e:?}")))?;

            let ds = DataSection::new(code.variables, Vec::new(), code.literals);

            compiled = Some(RefCell::new(Box::new(BytecodeFunction {
                code: Rc::new(BytecodeFunctionCode {
                    instructions: code.instructions,
                    ds,
                }),
                is_async: func.is_async,
                is_generator: func.is_generator,
            })));
        }

        OptimFunction::new(name, func.params.clone(), compiled, scope, realm)
    }
}
