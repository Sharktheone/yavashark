use std::path::PathBuf;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use tokio::runtime::Builder;
use yavashark_env::print::PrettyPrint;
use yavashark_env::scope::Scope;
use yavashark_env::Realm;
use yavashark_interpreter::eval::InterpreterEval;

#[allow(clippy::expect_used)]
pub fn main() {
    let path = std::env::args().nth(1).expect("Please provide a file path");

    let file = std::fs::File::open(&path).expect("Failed to open file");
    let mmap = unsafe { memmap2::Mmap::map(&file).expect("Failed to map file") };
    let input = str::from_utf8(&mmap).expect("Failed to read file as UTF-8");

    if input.is_empty() {
        return;
    }

    let input = StringInput::new(input, BytePos(0), BytePos(input.len() as u32));

    let c = EsSyntax {
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

    let script = p.parse_script().expect("Failed to parse script");

    let mut realm = Realm::new().expect("Failed to create realm");

    let mut scope = Scope::global(&realm, PathBuf::from(path));
    realm
        .set_eval(InterpreterEval, false)
        .expect("Failed to set eval");

    // #[cfg(feature = "vm")]
    // yavashark_vm::init(&mut realm).expect("Failed to init VM");

    let _result =
        match yavashark_interpreter::Interpreter::run_in(&script.body, &mut realm, &mut scope) {
            Ok(v) => v,
            Err(e) => {
                println!("Error: {}", e.pretty_print(&mut realm));
                return;
            }
        };

    if realm.has_pending_jobs() {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime");
        rt.block_on(realm.run_event_loop());
    }
}
