mod helper;

use crate::conf::Conf;
use crate::repl::helper::ReplHelper;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, EditMode, Editor};
use std::path::PathBuf;
use std::time::Instant;
use tokio::runtime::{Builder, Runtime};
use yavashark_env::print::PrettyPrint;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res};
use yavashark_interpreter::eval::InterpreterEval;
use yavashark_swc_validator::Validator;

pub fn repl(conf: Conf, preload: Option<(String, PathBuf)>) -> Res {
    let (preload, path) = match preload {
        Some((code, path)) => (Some(code), path),
        None => (None, PathBuf::from("repl.js")),
    };

    let mut interpreter_realm = Realm::new()?;

    #[cfg(feature = "vm")]
    crate::optimizer::define_optimizer(&mut interpreter_realm)?;
    #[cfg(feature = "vm")]
    yavashark_vm::init(&mut interpreter_realm)?;
    interpreter_realm.set_eval(InterpreterEval, false)?;
    let mut interpreter_scope = Scope::global(&interpreter_realm, path.clone());

    let mut vm_realm = Realm::new()?;
    vm_realm.set_eval(InterpreterEval, false)?;
    #[cfg(feature = "vm")]
    yavashark_vm::init(&mut vm_realm)?;
    let vm_scope = Scope::global(&vm_realm, path);

    let mut old_vm_realm = Realm::new()?;
    old_vm_realm.set_eval(InterpreterEval, false)?;
    #[cfg(feature = "vm")]
    yavashark_vm::init(&mut old_vm_realm)?;

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .build();

    let mut rl = Editor::with_config(config)?;

    let h = ReplHelper::new(interpreter_scope.clone(), vm_scope.clone(), conf);

    rl.set_helper(Some(h));

    let mut count = 1u32;

    let rt = Builder::new_current_thread().enable_all().build()?;

    if let Some(code) = &preload {
        run_input(
            code,
            conf,
            false,
            &mut interpreter_realm,
            &mut interpreter_scope,
            &mut vm_realm,
            &vm_scope,
            &rt,
        );
    }

    let mut last_ctrl_c: Option<Instant> = None;

    loop {
        let p = format!("{count}> ");

        if let Some(helper) = rl.helper_mut() {
            helper.colored_prompt = format!("\x1b[1;32m{p}\x1b[0m");
        }
        let readline = rl.readline(&p);

        let mut input = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                if let Some(last) = last_ctrl_c && last.elapsed().as_secs() < 2 {
                        break;
                }

                println!("Please use `Ctrl+D` or press `Ctrl+C` again to exit");

                last_ctrl_c = Some(Instant::now());

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

            input = match std::fs::read_to_string(file) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading {file}: {e}");
                    continue;
                }
            };
        }

        run_input(
            &input,
            conf,
            true,
            &mut interpreter_realm,
            &mut interpreter_scope,
            &mut vm_realm,
            &vm_scope,
            &rt,
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_input(
    input: &str,
    conf: Conf,
    echo_result: bool,
    interpreter_realm: &mut Realm,
    interpreter_scope: &mut Scope,
    vm_realm: &mut Realm,
    vm_scope: &Scope,
    rt: &Runtime,
) {
    if input.is_empty() {
        return;
    }

    let Some(script) = crate::parse::parse_script(input, "repl.js") else {
        return;
    };

    if let Err(e) = Validator::new().validate_statements(&script.body) {
        eprintln!("SyntaxError: {e}");
        return;
    }

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
                eprintln!("Uncaught {}", e.pretty_print(interpreter_realm));
                return;
            }
        };

        if echo_result {
            if conf.bytecode {
                println!("Interpreter: {}", result.pretty_print(interpreter_realm));
            } else {
                println!("{}", result.pretty_print(interpreter_realm));
            }
        }

        rt.block_on(interpreter_realm.run_event_loop());
    }

    #[cfg(feature = "vm")]
    if conf.bytecode || conf.instructions {
        let bc = match yavashark_compiler::Compiler::compile(&script.body) {
            Ok(bc) => bc,
            Err(e) => {
                eprintln!("Failed to compile code: {e:?}");
                return;
            }
        };

        if conf.instructions {
            println!("{bc:#?}");
        }

        #[cfg(feature = "vm")]
        if conf.bytecode {
            use yavashark_vm::yavashark_bytecode::data::DataSection;
            use yavashark_vm::{BorrowedVM, VM};
            let data = DataSection::new(bc.variables, Vec::new(), bc.literals, bc.control);
            let mut vm =
                BorrowedVM::with_scope(&bc.instructions, &data, vm_realm, vm_scope.clone());

            if let Err(e) = vm.run() {
                eprintln!("Uncaught: {e:?}");
            }

            if echo_result {
                println!("Bytecode: {:?}", vm.acc());
            }

            rt.block_on(vm_realm.run_event_loop());
        }
    }
}
