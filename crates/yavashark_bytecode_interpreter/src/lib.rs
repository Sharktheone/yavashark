use std::rc::Rc;
use swc_ecma_ast::{Function, Stmt};

pub use yavashark_bytecode as bytecode;
use yavashark_bytecode::{BytecodeFunctionCode, BytecodeFunctionParams};
pub use yavashark_codegen as codegen;
pub use yavashark_vm as vm;

use yavashark_codegen::ByteCodegen;
use yavashark_compiler::Compiler;
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use yavashark_env::{Error, ObjectHandle, Realm, Res, ValueResult};
use yavashark_vm::OldBorrowedVM;
use yavashark_vm::async_bytecode_function::AsyncBytecodeFunction;
use yavashark_vm::async_generator::AsyncGeneratorFunction;
use yavashark_vm::bytecode_function::BytecodeFunction;
use yavashark_vm::generator::GeneratorFunction;
use yavashark_vm::yavashark_bytecode::data::DataSection;

pub struct ByteCodeInterpreter;

impl ByteCodeInterpreter {
    pub fn run_in(script: &Vec<Stmt>, realm: &mut Realm, scope: &mut Scope) -> ValueResult {
        let code = ByteCodegen::compile(script)
            .map_err(|e| Error::new_error(format!("Failed to compile: {e:?}")))?;

        let ds = DataSection::new(code.variables, Vec::new(), code.literals, Vec::new());

        let mut vm = OldBorrowedVM::with_scope(&code.instructions, &ds, realm, scope.clone());

        vm.run_ret()
    }

    pub fn compile_fn(
        func: &Function,
        name: String,
        scope: Scope,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let mut compiled: Option<Rc<BytecodeFunctionCode>> = None;
        if let Some(body) = &func.body {
            let code = Compiler::compile(&body.stmts)
                .map_err(|e| Error::syn_error(format!("Failed to compile: {e:?}")))?;

            let ds = DataSection::new(code.variables, Vec::new(), code.literals, code.control);

            compiled = Some(Rc::new(BytecodeFunctionCode {
                instructions: code.instructions,
                ds,
            }));
        }

        let len = func.params.last().map_or(0, |last| {
            if last.pat.is_rest() {
                func.params.len() - 1
            } else {
                func.params.len()
            }
        });

        let params = {
            let (params_code, params_defs) =
                Compiler::compile_params(func.params.iter().map(|p| &p.pat))
                    .map_err(|e| Error::syn_error(format!("Failed to compile: {e:?}")))?;

            let ds = DataSection::new(
                params_code.variables,
                Vec::new(),
                params_code.literals,
                params_code.control,
            );

            BytecodeFunctionParams {
                instructions: params_code.instructions,
                ds,
                defs: params_defs,
            }
        };

        if func.is_generator && !func.is_async {
            let g = GeneratorFunction::new(compiled.unwrap_or_default(), scope, realm, params)?;

            g.define_property_attributes("length".into(), len.into(), realm)?;
            g.define_property_attributes("name".into(), name.into(), realm)?;

            return Ok(g.into_object());
        }

        if func.is_generator && func.is_async {
            let g =
                AsyncGeneratorFunction::new(compiled.unwrap_or_default(), scope, realm, params)?;

            g.define_property_attributes("length".into(), len.into(), realm)?;
            g.define_property_attributes("name".into(), name.into(), realm)?;

            return Ok(g.into_object());
        }

        if func.is_async {
            let f = AsyncBytecodeFunction::new(compiled.unwrap_or_default(), scope, realm, params);

            f.define_property_attributes("length".into(), len.into(), realm)?;
            f.define_property_attributes("name".into(), name.into(), realm)?;

            return Ok(f.into_object());
        }

        let f = BytecodeFunction::new(compiled.unwrap_or_default(), scope, realm, params);

        f.define_property_attributes("length".into(), len.into(), realm)?;
        f.define_property_attributes("name".into(), name.into(), realm)?;

        Ok(f.into_object())
    }
}
