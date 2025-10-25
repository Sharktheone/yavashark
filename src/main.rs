mod conf;
mod minimal;
#[cfg(feature = "vm")]
mod optimizer;
mod repl;
mod run;
mod simplerepl;

fn main() {
    #[cfg(not(feature = "minimal"))]
    run::main();

    #[cfg(feature = "minimal")]
    minimal::main();
}
