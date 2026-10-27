<picture>
  <source media="(prefers-color-scheme: dark)" srcset=".github/assets/logo-dark.svg">
  <img src=".github/assets/logo-light.svg" alt="Yavashark, a pink shark" width="160" align="right">
</picture>

# Yavashark

A new JavaScript engine, currently in development, written in Rust. 🦈

<br clear="right">

## Project Status

The current focus is getting the engine faster. For that multiple POCs have been made. For example string v3, 
NaN-Boxing value type, the new object model and finally the standalone GC project.

## ECMA-262 Compliance

Yavashark currently passes ~81% of the entire test262 suite including intl, temporal, annexB and staging. 

## Contributing

Contributions to Yavashark are welcome! Whether it's reporting bugs, suggesting features, or contributing code, we
appreciate all forms of help in making Yavashark better.

## Compiling

Compile the engine via Cargo. Run

```shell
cargo build --release # for a binary in target/release/yavashark
cargo run --release # for running the engine directly
```

## Usage

```shell
yavashark                      # start the REPL
yavashark script.js            # run a script
yavashark -e 'console.log(1)'  # evaluate code
yavashark -s script.js         # run a script, then stay in the REPL
```

| Option | Description |
| --- | --- |
| `-e, --eval <CODE>` | Evaluate code instead of a script |
| `-s, --repl` | Stay in the REPL after the script or `--eval` |
| `-i, --interpreter` | Use the tree-walk interpreter (default) |
| `-b, --bytecode` | Use the bytecode VM |
| `-a, --ast` | Print the AST |
| `-I, --instructions` | Print the bytecode |
| `--profile-out <PATH>` | Write a JS profile (needs `--features profiler`) |
| `--native-profile` | Profile the engine itself (needs `--features pprof`) |
| `--native-profile-out <PATH>` | Where to write the native profile |

Run `yavashark --help` for the full list.
