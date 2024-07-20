The `yavashark_bytecode` crate contains the bytecode representation for the YavaShark engine, it cannot execute it nor generate it, it just defines the structure.

It was split up, so we can have a later on JIT, that does not rely on the codegen or vm part, but only on its definitions.