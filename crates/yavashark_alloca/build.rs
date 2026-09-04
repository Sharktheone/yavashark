fn main() {
    println!("cargo::rustc-check-cfg=cfg(has_c_alloca)");
    println!("cargo::rerun-if-changed=src/alloca.c");
    println!("cargo::rerun-if-env-changed=PATH");

    if std::env::var_os("CARGO_FEATURE_HEAP_ALLOCA").is_some() {
        println!(
            "cargo::warning=yavashark_shonk: heap-alloca enabled; using heap allocation instead of C alloca"
        );
        return;
    }

    // Building the helper also checks target support, headers and archive tools.
    match cc::Build::new()
        .file("src/alloca.c")
        .flag_if_supported("-fstack-clash-protection")
        .try_compile("yavashark_alloca")
    {
        Ok(()) => println!("cargo::rustc-cfg=has_c_alloca"),
        Err(error) => {
            let reason = error.to_string().replace(['\n', '\r'], " ");
            println!(
                "cargo::warning=yavashark_shonk: C alloca helper unavailable; using heap allocation instead: {reason}"
            );
        }
    }
}
