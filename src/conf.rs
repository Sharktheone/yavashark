#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct Conf {
    pub ast: bool,
    pub interpreter: bool,
    pub bytecode: bool,
    pub old_bytecode: bool,
    pub instructions: bool,
}
