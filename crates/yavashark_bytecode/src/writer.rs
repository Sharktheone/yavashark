use std::io::Write;
use bincode::Encode;

/// Own bytecode format, one compact one and one "normal" one, it can also make use of the bincode crate
pub trait BytecodeWriter: Encode {
    fn write(&self, buffer: &mut impl Write);

    fn write_tight(&self, buffer: &mut impl Write);
}





///adc2-243d
const NORMAL_HEADER: [u8; 8] = *b"adc2243d";


///9961-c49c
const TIGHT_HEADER: [u8; 8] = *b"9961c49c";
//Who can find out for what these stand for? Open an issue if you do! (It's a hidden message!)

