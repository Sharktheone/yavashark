mod conf;
#[cfg(feature = "minimal")]
mod minimal;
#[cfg(feature = "vm")]
mod optimizer;
mod repl;
#[cfg(not(feature = "minimal"))]
mod run;

fn main() {
    #[cfg(not(feature = "minimal"))]
    run::main();

    #[cfg(feature = "minimal")]
    minimal::main();
}
