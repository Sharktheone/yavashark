use std::io::Write;
use bincode::Encode;

/// Own bytecode format, one compact one and one "normal" one, it can also make use of the bincode crate
pub trait BytecodeWriter: Encode {
    fn write(&self, buffer: &mut impl Write);

    fn write_compact(&self, buffer: &mut impl Write);
}
