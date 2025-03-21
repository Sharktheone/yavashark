use crate::instruction::generate_instruction_enum;

mod parse;
mod instruction;
mod set;

fn main() {
    generate_instruction_enum()
}
