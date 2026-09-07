mod conf;
#[cfg(not(feature = "minimal"))]
mod parse;
#[cfg(feature = "minimal")]
mod minimal;
#[cfg(feature = "vm")]
mod optimizer;
mod repl;
#[cfg(not(feature = "minimal"))]
mod run;

fn main() -> std::process::ExitCode {
    #[cfg(not(feature = "minimal"))]
    return run::main();

    #[cfg(feature = "minimal")]
    return minimal::main();
}
