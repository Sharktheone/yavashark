mod conf;
#[cfg(feature = "minimal")]
mod minimal;
#[cfg(feature = "vm")]
mod optimizer;
#[cfg(not(feature = "minimal"))]
mod parse;
mod repl;
#[cfg(not(feature = "minimal"))]
mod run;

fn main() -> std::process::ExitCode {
    #[cfg(not(feature = "minimal"))]
    return run::main();

    #[cfg(feature = "minimal")]
    return minimal::main();
}
