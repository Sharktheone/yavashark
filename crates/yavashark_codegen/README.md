The `yavashark_codegen` crate contains the code generation for the YavaShark engine. It is used to generate the bytecode
defined in `yavashark_bytecode`. It can be executed via the `yavashark_vm` crate.

It does not know how to execute it, just how to generate the bytecode from a swc AST