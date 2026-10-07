# Yavashark

A new JavaScript engine, currently in development, written in Rust. 

## Project Status

The current focus is getting the engine faster. For that multiple POCs have been made. For example string v3, 
NaN-Boxing value type, the new object model and finally the standalone GC project.

## ECMA-262 Compliance

Yavashark currently passes ~81% of the entire test262 suite including intl, temporal, annexB and stating. 

## Contributing

Contributions to Yavashark are welcome! Whether it's reporting bugs, suggesting features, or contributing code, we
appreciate all forms of help in making Yavashark better.

## Compiling

Compile the engine via Cargo. Run

```shell
cargo build --release # for a binary in target/release/yavashark
cargo run --release # for running the engine directly
```

## Running test262 

To run a standalone test262 test, you can use the yavashark_test262 binary. 

```shell
cd crates/yavashark_test262
cargo r <path/to/test.js> # for running a single test
```

To run the whole test suite, first compile the test262 runner via

```sh
cd crates/yavashark_test262/runner
go build
```

To then run the testsuite, checkout the test262 repository in the repo root, so `yavashark/test262` or change the default test262 path
```shell
git clone https://github.com/tc39/test262 # from the repository root
```

afterwards run the suite with the runner

```shell
cd crates/yavashark_test262/
runner/yavashark_test262_runner -p rebuild --noskip # compiles the engine and runs every test
```

If you wish to run a faster version, use the fast profile, though make sure to have run `cargo b -r -p yavashark_test262 ` before 

```shell
cd crates/yavashark_test262/
cargo b -r
runner/yavashark_test262_runner -p fast
```

On my machine this can run a subset of around 97% of the testsuite in under 10 seconds!