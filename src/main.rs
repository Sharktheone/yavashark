mod repl;

use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_common::errors::{Handler, HANDLER};
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_codegen::ByteCodegen;
use yavashark_env::print::PrettyPrint;
use yavashark_env::scope::Scope;
use yavashark_env::Realm;
use yavashark_vm::yavashark_bytecode::data::DataSection;
use yavashark_vm::VM;
use crate::repl::Repl;

#[allow(clippy::unwrap_used)]
fn main() {
    let matches = clap::Command::new("yavashark")
        .version("0.1.0")
        .about("A JavaScript interpreter written in Rust")
        .arg(
            clap::Arg::new("source")
                .help("The source file to interpret")
                .required(false)
                .index(1),
        )
        .arg(
            clap::Arg::new("interpreter")
                .help("Run with the tree-walk-interpreter")
                .short('i')
                .required(false)
                .default_value("false")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("bytecode")
                .help("Run with the bytecode-interpreter")
                .short('b')
                .required(false)
                .default_value("false")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("ast")
                .help("Print the AST")
                .short('a')
                .required(false)
                .default_value("false")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("instructions")
                .help("Print the instructions")
                .short('I')
                .required(false)
                .default_value("false")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("shell")
                .help("Interactive shell (repl)")
                .short('s')
                .short_alias('r')
                .alias("repl")
                .required(false)
                .default_value("false")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let interpreter = matches.get_flag("interpreter");
    let bytecode = matches.get_flag("bytecode");
    let ast = matches.get_flag("ast");
    let instructions = matches.get_flag("instructions");
    let shell = matches.get_flag("shell");

    if !(interpreter || bytecode || ast || instructions) {
        println!("No interpreter specified");
        return;
    }

    let src = matches.get_one::<String>("source");

    if shell && src.is_some() {
        println!("Cannot run src file and shell");
        return;
    }

    if let Some(src) = src {
        let path = PathBuf::from(src);

        let input = std::fs::read_to_string(src).unwrap();

        if input.is_empty() {
            return;
        }

        let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32 - 1));

        let c = EsSyntax::default();

        let mut p = Parser::new(Syntax::Es(c), input, None);

        let script = p.parse_script().unwrap();

        if ast {
            println!("AST:\n{script:#?}");
        }

        if interpreter {
            let result =
                yavashark_interpreter::Interpreter::run(&script.body, path.clone()).unwrap();
            println!("Interpreter: {result:?}");
        }

        if bytecode || instructions {
            let bc = ByteCodegen::compile(&script.body).unwrap();

            if instructions {
                println!("{bc:#?}");
            }

            if bytecode {
                let data = DataSection::new(bc.variables, bc.literals);

                let mut vm = VM::new(bc.instructions, data, path).unwrap();

                vm.run().unwrap();

                println!("Bytecode: {:?}", vm.acc());
            }
        }
    }

    if shell {
        let path = Path::new("repl.js");

        let mut interpreter_realm = Realm::new().unwrap();
        let mut interpreter_scope = Scope::global(&interpreter_realm, path.to_path_buf());

        let mut vm_realm = Realm::new().unwrap();
        let vm_scope = Scope::global(&vm_realm, path.to_path_buf());


        let syn = Syntax::Es(EsSyntax::default());
        
        let mut repl = Repl::new(Box::new(move |input| {

            if input.is_empty() {
                return;
            }

            let input = StringInput::new(&input, BytePos(0), BytePos(input.len() as u32 - 1));

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
                    
                    eprintln!("{:?}", e);
                    
                    return
                } 
            };

            if ast {
                println!("AST:\n{script:#?}");
            }

            if interpreter {
                let result = yavashark_interpreter::Interpreter::run_in(
                    &script.body,
                    &mut interpreter_realm,
                    &mut interpreter_scope,
                )
                    .unwrap();

                if bytecode {
                    println!("Interpreter: {}", result.pretty_print())
                } else {
                    println!("{}", result.pretty_print())
                }
            }

            if bytecode || instructions {
                let bc = ByteCodegen::compile(&script.body).unwrap();

                if instructions {
                    println!("{bc:#?}");
                }

                if bytecode {
                    let data = DataSection::new(bc.variables, bc.literals);

                    let mut vm = VM::with_realm_scope(
                        bc.instructions,
                        data,
                        vm_realm.clone(),
                        vm_scope.clone(),
                        path.to_path_buf(),
                    );

                    vm.run().unwrap();

                    println!("Bytecode: {:?}", vm.acc());
                }
            }
        }));
        
        
        repl.run();
    }
}
