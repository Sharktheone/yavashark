use yavashark::Engine;

const SCRIPT: &str = r#"
function fib(n) {
    return n < 2 ? n : fib(n - 1) + fib(n - 2);
}

console.log("fib(20) =", fib(20));

"hello from " + ["yava", "shark"].join("")
"#;

fn main() {
    let mut engine = Engine::new();

    let result = engine
        .run(SCRIPT, "examples/embed.rs")
        .expect("failed to run script");

    println!(
        "result: {}",
        result
            .to_string(engine.realm())
            .expect("failed to stringify")
    );
}
