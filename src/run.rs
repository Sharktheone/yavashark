use crate::conf;
use crate::repl::repl;
#[cfg(feature = "pprof")]
use flate2::{Compression, write::GzEncoder};
#[cfg(feature = "pprof")]
use pprof::ProfilerGuard;
#[cfg(feature = "pprof")]
use pprof::protos::Message;
#[cfg(feature = "pprof")]
use std::fs::File;
#[cfg(feature = "pprof")]
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use swc_ecma_ast::Program;
use tokio::runtime::Builder;
use yavashark_env::print::PrettyPrint;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm};
use yavashark_interpreter::eval::InterpreterEval;
use yavashark_swc_validator::Validator;

fn cli() -> clap::Command {
    clap::Command::new("yavashark")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A JavaScript interpreter written in Rust")
        .after_help(
            r"Examples:
yavashark                     Start the interactive REPL
yavashark script.js           Run a script
yavashark -e 'console.log(1)' Evaluate code
yavashark -s script.js        Run a script, then stay in the REPL",
        )
        .arg(
            clap::Arg::new("source")
                .help("The script to run (starts the REPL if omitted)")
                .value_name("SCRIPT")
                .index(1),
        )
        .arg(
            clap::Arg::new("eval")
                .help("Evaluate the provided JavaScript code")
                .short('e')
                .long("eval")
                .value_name("CODE")
                .conflicts_with("source"),
        )
        .arg(
            clap::Arg::new("shell")
                .help("Stay in the interactive REPL after running SCRIPT or --eval")
                .short('s')
                .short_alias('r')
                .long("repl")
                .visible_alias("shell")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("interpreter")
                .help("Run with the tree-walk-interpreter")
                .short('i')
                .long("interpreter")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("bytecode")
                .help("Run with the bytecode-interpreter")
                .short('b')
                .long("bytecode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("ast")
                .help("Print the AST")
                .short('a')
                .long("ast")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("instructions")
                .help("Print the instructions")
                .short('I')
                .long("instructions")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("profile-out")
                .help("Write JS profiler output to this path")
                .long("profile-out")
                .value_name("PATH"),
        )
        .arg(
            clap::Arg::new("native-profile")
                .help("Enable native pprof profiling")
                .long("native-profile")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("native-profile-out")
                .help("Write native pprof output to this path")
                .long("native-profile-out")
                .value_name("PATH")
                .requires("native-profile"),
        )
}

#[allow(clippy::unwrap_used)]
pub fn main() -> ExitCode {
    let matches = cli().get_matches();

    let mut interpreter = matches.get_flag("interpreter");
    let bytecode = matches.get_flag("bytecode");
    let ast = matches.get_flag("ast");
    let instructions = matches.get_flag("instructions");
    let shell = matches.get_flag("shell");
    let eval_code = matches.get_one::<String>("eval");
    let js_profile_out = matches.get_one::<String>("profile-out").cloned();
    let native_profile_out = matches.get_one::<String>("native-profile-out").cloned();
    let native_profile = matches.get_flag("native-profile");

    if !(interpreter || bytecode || ast || instructions) {
        interpreter = true;
    }

    let src = matches.get_one::<String>("source");

    // Either a script to run or code to evaluate - `conflicts_with` rules out both at once.
    let input = eval_code.map_or_else(
        || {
            src.map(|src| {
                let content = match std::fs::read_to_string(src) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Error reading {src}: {e}");
                        std::process::exit(1);
                    }
                };

                (content, PathBuf::from(src))
            })
        },
        |code| Some((code.clone(), PathBuf::from("<eval>"))),
    );

    let config = conf::Conf {
        ast,
        interpreter,
        bytecode,
        instructions,
    };

    if shell || input.is_none() {
        if let Err(e) = repl(config, input) {
            eprintln!("{e:?}");
            return ExitCode::FAILURE;
        }

        return ExitCode::SUCCESS;
    }

    if let Some((code, path)) = input {
        if code.is_empty() {
            return ExitCode::SUCCESS;
        }

        return run_code(
            &code,
            path,
            ast,
            interpreter,
            bytecode,
            instructions,
            js_profile_out.as_deref(),
            native_profile,
            native_profile_out.as_deref(),
        );
    }

    ExitCode::SUCCESS
}

#[allow(
    clippy::unwrap_used,
    clippy::too_many_arguments,
    clippy::fn_params_excessive_bools
)]
fn run_code(
    input: &str,
    path: PathBuf,
    ast: bool,
    interpreter: bool,
    #[allow(unused_variables)] bytecode: bool,
    #[allow(unused_variables)] instructions: bool,
    #[allow(unused_variables)] js_profile_out: Option<&str>,
    native_profile: bool,
    #[allow(unused_variables)] native_profile_out: Option<&str>,
) -> ExitCode {
    let Some(prog) = crate::parse::parse_program(input, &path.display().to_string()) else {
        return ExitCode::FAILURE;
    };

    if ast {
        println!("AST:\n{prog:#?}");
    }

    let mut validator = Validator::new();

    match &prog {
        Program::Script(script) => {
            if let Err(e) = validator.validate_statements(&script.body) {
                println!("SyntaxError: {e}");
                return ExitCode::FAILURE;
            }
        }
        Program::Module(module) => {
            if let Err(e) = validator.validate_module_items(&module.body) {
                println!("SyntaxError: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    #[cfg(feature = "pprof")]
    let native_guard = if native_profile {
        Some(ProfilerGuard::new(1_000_000).unwrap())
    } else {
        None
    };

    #[cfg(not(feature = "pprof"))]
    if native_profile {
        eprintln!(
            "Native profiling requested but not enabled at compile time. Rebuild with --features pprof."
        );
    }

    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    if interpreter {
        let mut realm = Realm::new().unwrap();
        let mut scope = Scope::global(&realm, path.clone());
        realm.set_eval(InterpreterEval, false).unwrap();
        #[cfg(feature = "profiler")]
        if let Some(profile_out) = js_profile_out {
            let p = match yavashark_profiler::FileProfileWriter::from_path(
                PathBuf::from(profile_out).as_path(),
            ) {
                Ok(k) => k,
                Err(e) => {
                    println!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            realm.set_profile_writer(p);
        }
        yavashark_vm::init(&mut realm).unwrap();

        let result =
            match yavashark_interpreter::Interpreter::run_program_in(&prog, &mut realm, &mut scope)
            {
                Ok(v) => v,
                Err(e) => {
                    println!("{}", e.pretty_print(&mut realm));
                    return ExitCode::FAILURE;
                }
            };
        if !result.is_undefined() {
            if bytecode && interpreter {
                println!("Interpreter: {result:?}");
            } else {
                println!("{result:?}");
            }
        }

        rt.block_on(realm.run_event_loop());

        #[cfg(feature = "profiler")]
        if let Some(profile_out) = js_profile_out {
            match realm.write_profile() {
                Ok(_) => {
                    eprintln!("wrote JS profile to {}", profile_out);
                }
                Err(e) => {
                    eprintln!("Error writing JS profile: {e}");
                }
            }
        }
    }

    #[cfg(feature = "pprof")]
    if let Some(guard) = native_guard {
        write_native_profile(native_profile_out.as_deref(), path.as_path(), guard);
    }

    #[cfg(feature = "vm")]
    if bytecode {
        use yavashark_vm::yavashark_bytecode::data::DataSection;
        use yavashark_vm::{OwnedVM, VM};

        let Some(script) = prog.as_script() else {
            eprintln!("Only scripts are supported in bytecode mode currently");
            return ExitCode::FAILURE;
        };

        let bc = yavashark_compiler::Compiler::compile(&script.body).unwrap();

        if instructions {
            println!("{bc:#?}");
        }

        let data = DataSection::new(bc.variables, Vec::new(), bc.literals, bc.control);
        let mut vm = OwnedVM::new(bc.instructions, data, path).unwrap();

        match vm.run() {
            Ok(()) => {}
            Err(ControlFlow::Continue(_)) => {
                println!("Error: Unexpected continue");
                return ExitCode::FAILURE;
            }
            Err(ControlFlow::Break(_)) => {
                println!("Error: Unexpected break");
                return ExitCode::FAILURE;
            }
            Err(ControlFlow::Return(_)) => {
                println!("Error: Unexpected return");
                return ExitCode::FAILURE;
            }
            Err(ControlFlow::Error(err)) => {
                println!("{}", err.pretty_print(vm.get_realm()));
                return ExitCode::FAILURE;
            }
            Err(ControlFlow::Yield(_) | ControlFlow::YieldStar(_)) => {
                println!("Error: Unexpected yield");
                return ExitCode::FAILURE;
            }
            Err(ControlFlow::Await(_)) => {
                println!("Error: Unexpected await");
                return ExitCode::FAILURE;
            }
            Err(ControlFlow::OptChainShortCircuit) => {
                println!("Error: Unexpected optional chaining short-circuit");
                return ExitCode::FAILURE;
            }
        }

        rt.block_on(vm.get_realm().run_event_loop());

        let ret = vm.acc();

        if !ret.is_undefined() {
            if bytecode && interpreter {
                println!("Bytecode: {ret:?}");
            } else {
                println!("{ret:?}");
            }
        }
    }

    #[cfg(feature = "vm")]
    if instructions {
        let Some(script) = prog.as_script() else {
            eprintln!("Only scripts are supported in bytecode mode currently");
            return ExitCode::FAILURE;
        };

        let bc = yavashark_codegen::ByteCodegen::compile(&script.body).unwrap();

        if instructions {
            println!("{bc:#?}");
        }
    }

    ExitCode::SUCCESS
}

#[cfg(feature = "pprof")]
#[allow(clippy::unwrap_used)]
fn write_native_profile(
    profile_out: Option<&str>,
    path: &std::path::Path,
    guard: ProfilerGuard<'_>,
) {
    let Ok(report) = guard.report().build() else {
        return;
    };
    let Ok(profile) = report.pprof() else {
        return;
    };

    let mut buf = Vec::new();
    if profile.encode(&mut buf).is_err() {
        return;
    }

    let out = profile_out
        .map(PathBuf::from)
        .unwrap_or_else(|| default_native_profile_path(path));
    if let Some(parent) = out.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let Ok(file) = File::create(&out) else {
        return;
    };
    let mut encoder = GzEncoder::new(file, Compression::default());
    if encoder.write_all(&buf).is_err() {
        return;
    }
    if encoder.finish().is_err() {
        return;
    }

    eprintln!("wrote native profile to {}", out.display());
}

#[cfg(feature = "pprof")]
fn default_native_profile_path(path: &std::path::Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("profile");
    PathBuf::from(format!("profiles/{stem}.native.pb.gz"))
}
