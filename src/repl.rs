mod helper;

use crate::conf;
use crate::conf::Conf;
use crate::repl::helper::ReplHelper;
use crate::simplerepl::Repl;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, EditMode, Editor};
use std::path::Path;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_codegen::ByteCodegen;
use yavashark_env::print::PrettyPrint;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res};
use yavashark_vm::yavashark_bytecode::data::DataSection;
use yavashark_vm::VM;

pub fn repl(conf: Conf) -> Res {
    let path = Path::new("repl.js");

    let mut interpreter_realm = Realm::new()?;
    let mut interpreter_scope = Scope::global(&interpreter_realm, path.to_path_buf());

    let vm_realm = Realm::new()?;
    let vm_scope = Scope::global(&vm_realm, path.to_path_buf());

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .build();

    let mut rl = Editor::with_config(config)?;

    let h = ReplHelper::new(interpreter_scope.clone(), vm_scope.clone(), conf);

    rl.set_helper(Some(h));

    let mut count = 1;

    loop {
        let p = format!("{count}> ");

        if let Some(helper) = rl.helper_mut() {
            helper.colored_prompt = format!("\x1b[1;32m{p}\x1b[0m");
        }
        let readline = rl.readline(&p);

        let mut input = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                println!("Please use `Ctrl-D` to exit");
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }

            Err(err) => {
                eprintln!("Error: {err}");
                break;
            }
        };

        rl.add_history_entry(input.as_str())?;
        count += 1;

        if let Some(file) = input.strip_prefix('!') {
            let file = file.trim();
            input = std::fs::read_to_string(file)?;
        }

        run_input(
            &input,
            conf,
            &mut interpreter_realm,
            &mut interpreter_scope,
            &vm_realm,
            &vm_scope,
        );
    }

    Ok(())
}

fn run_input(
    input: &str,
    conf: Conf,
    interpreter_realm: &mut Realm,
    interpreter_scope: &mut Scope,
    vm_realm: &Realm,
    vm_scope: &Scope,
) {
    if input.is_empty() {
        return;
    }

    let input = StringInput::new(input, BytePos(0), BytePos(input.len() as u32 - 1));
    let syn = Syntax::Es(EsSyntax::default());

    let mut p = Parser::new(syn, input, None);

    let script = match p.parse_script() {
        Ok(s) => s,
        Err(e) => {
            // HANDLER.with(|h| {
            //     let mut diagnostic = e.into_diagnostic(h);
            //
            //     diagnostic.emit();
            //
            //
            // });

            eprintln!("{e:?}");

            return;
        }
    };

    if conf.ast {
        println!("AST:\n{script:#?}");
    }

    if conf.interpreter {
        let result = match yavashark_interpreter::Interpreter::run_in(
            &script.body,
            interpreter_realm,
            interpreter_scope,
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Uncaught {}", e.pretty_print());
                return;
            }
        };

        if conf.bytecode {
            println!("Interpreter: {}", result.pretty_print());
        } else {
            println!("{}", result.pretty_print());
        }
    }

    if conf.bytecode || conf.instructions {
        let bc = match ByteCodegen::compile(&script.body) {
            Ok(bc) => bc,
            Err(e) => {
                eprintln!("Failed to compile code: {e:?}");
                return;
            }
        };

        if conf.instructions {
            println!("{bc:#?}");
        }

        if conf.bytecode {
            let data = DataSection::new(bc.variables, bc.literals);

            let mut vm =
                VM::with_realm_scope(bc.instructions, data, vm_realm.clone(), vm_scope.clone());

            if let Err(e) = vm.run() {
                eprintln!("Uncaught: {e:?}");
            }

            println!("Bytecode: {:?}", vm.acc());
        }
    }
}

pub fn old_repl(conf: conf::Conf) -> Res {
    let path = Path::new("repl.js");

    let mut interpreter_realm = Realm::new()?;
    let mut interpreter_scope = Scope::global(&interpreter_realm, path.to_path_buf());

    let vm_realm = Realm::new()?;
    let vm_scope = Scope::global(&vm_realm, path.to_path_buf());

    let mut repl = Repl::new(Box::new(move |input| {
        run_input(
            input,
            conf,
            &mut interpreter_realm,
            &mut interpreter_scope,
            &vm_realm,
            &vm_scope,
        );
    }));

    repl.run();
}
