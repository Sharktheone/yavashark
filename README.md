# Yavashark

A new TypeScript and JavaScript engine, currently in development, written in Rust. Yavashark is designed to be
TypeScript-first, providing a way to run your TypeScript projects at native speeds, without the need for transpiling to
JavaScript.

## Project Status

Currently, I'm working on the JS-Tree-Walk-Interpreter. This is more a test to see, how to implement an interpreter in
Rust.
Specifically on the JS Garbage Collector.

Next up would be an JS-Bytecode-Interpreter, which will be a test for the TS-Bytecode-Interpreter.

After that, I will start with the TS-Bytecode-Interpreter, which will be the first step to run TypeScript natively.

## Why Yavashark?

The goal of Yavashark is to provide an engine that is optimized for TypeScript. This means you can write your code in
TypeScript and run it directly, without the need to transpile it to JavaScript first. This can lead to performance
improvements and a simpler development process.

## ECMA-262 Compliance

While Yavashark is currently not compliant with the ECMA-262 standard, it is a goal for the future. As the project grows
and more developers contribute, It will full compliance with the standard.

## Contributing

Contributions to Yavashark are welcome! Whether it's reporting bugs, suggesting features, or contributing code, we
appreciate all forms of help in making Yavashark better.

### Interpreters and Compilers

So, I thought of a few interpreters and compilers for this project. Probably they are too many, but if we have the first
few, the next aren't that hard to implement.

- [ ] JS-Tree-Walk-Interpreter [In Progress]
- [ ] JS-Bytecode-Interpreter  [In Progress] (Stack based, TODO: Register based)
- [ ] TS-Bytecode-Interpreter
- [ ] TS-JIT-Compiler (Cranelift)
- [ ] JS-JIT-Compiler (Cranelift)
- [ ] TS-AOT-Compiler (Cranelift)
- [ ] TS-AOT-Compiler (LLVM)
- [ ] TS-JIT-Compiler (LLVM)
- [ ] JS-JIT-Compiler (LLVM)
- [ ] TS JIT-Compiler (Custom)
- [ ] JS JIT-Compiler (Custom)

The cranelift compilers should be relatively easy to implement, once we can compile to Cranelift IR (maybe except the
JS-JIT)
For LLVM it is the same as for Cranelift, but we have to compile to LLVM IR.

The custom JIT-Compilers are just for fun, because they are too risky for production.