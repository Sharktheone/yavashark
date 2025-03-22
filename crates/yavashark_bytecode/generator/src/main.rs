use crate::execute::generate_execute;
use crate::instruction::generate_instruction_enum;

mod parse;
mod instruction;
mod set;
mod execute;

fn main() {
    generate_instruction_enum();
    generate_execute();
}
