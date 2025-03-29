mod conf;
mod optimizer;
mod repl;
mod simplerepl;
#[cfg(target_arch = "wasm32")]
mod wasm;

use crate::repl::{old_repl, repl};
use std::path::PathBuf;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_codegen::ByteCodegen;
use yavashark_env::print::PrettyPrint;
use yavashark_vm::yavashark_bytecode::data::DataSection;
use yavashark_vm::OwnedVM;

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
        .arg(
            clap::Arg::new("shellold")
                .help("Old interactive shell (repl)")
                .short('S')
                .short_alias('R')
                .alias("replold")
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
    let shellold = matches.get_flag("shellold");

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
            let result = match yavashark_interpreter::Interpreter::run(&script.body, path.clone()) {
                Ok(v) => v,
                Err(e) => {
                    println!("Error: {}", e.pretty_print());
                    return;
                }
            };
            println!("Interpreter: {result:?}");
        }

        if bytecode || instructions {
            let bc = ByteCodegen::compile(&script.body).unwrap();

            if instructions {
                println!("{bc:#?}");
            }

            if bytecode {
                let data = DataSection::new(bc.variables, Vec::new(), bc.literals);

                let mut vm = OwnedVM::new(bc.instructions, data, path).unwrap();

                vm.run().unwrap();

                println!("Bytecode: {:?}", vm.acc());
            }
        }
    }

    let config = conf::Conf {
        ast,
        interpreter,
        bytecode,
        instructions,
    };

    if shell && shellold {
        println!("Cannot run both shells");
        return;
    }

    if shell {
        repl(config).unwrap();
    }

    if shellold {
        old_repl(config).unwrap();
    }
}
