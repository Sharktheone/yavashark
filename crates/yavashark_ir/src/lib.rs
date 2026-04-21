/// This IR will be used for many optimizations in the YS interpreter and compiler.
pub struct YavasharkIr {}

trait IRTranslator {
    fn translate(&self, input: &str) -> YavasharkIr;
}
