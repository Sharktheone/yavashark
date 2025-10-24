mod conf;
#[cfg(feature = "vm")]
mod optimizer;
mod repl;
mod simplerepl;
mod run;
mod minimal;

fn main() {
    #[cfg(not(feature = "minimal"))]
    run::main();

    #[cfg(feature = "minimal")]
    minimal::main();
}