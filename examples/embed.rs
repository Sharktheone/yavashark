use std::path::PathBuf;
use swc_common::BytePos;
use swc_common::input::StringInput;
use swc_ecma_parser::{Parser, Syntax};
use yavashark_env::Realm;
use yavashark_env::scope::Scope;
use yavashark_interpreter::Interpreter;

const SCRIPT: &str = r#"
function fib(n) {
    return n < 2 ? n : fib(n - 1) + fib(n - 2);
}

console.log("fib(20) =", fib(20));

"hello from " + ["yava", "shark"].join("")
"#;

fn main() {
    let input = StringInput::new(SCRIPT, BytePos(0), BytePos(SCRIPT.len() as u32));
    let script = Parser::new(Syntax::Es(Default::default()), input, None)
        .parse_script()
        .expect("failed to parse script");

    let mut realm = Realm::new().expect("failed to create realm");
    let mut scope = Scope::global(&realm, PathBuf::from("embed.js"));

    let result = Interpreter::run_in(&script.body, &mut realm, &mut scope)
        .expect("script threw an error");

    println!("result: {}", result.to_string(&mut realm).expect("failed to stringify"));
}
