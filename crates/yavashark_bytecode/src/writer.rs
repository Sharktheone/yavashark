use std::io::Write;

/// Own bytecode format, one compact one and one "normal" one
/// we probably also should use `bincode` so we have something that definetely works...
pub trait BytecodeWriter {
    fn write(&self, buffer: &mut impl Write);

    fn write_compact(&self, buffer: &mut impl Write);
}
