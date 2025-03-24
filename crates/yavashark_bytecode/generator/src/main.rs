use crate::constructors::generate_constructors;
use crate::execute::generate_execute;
use crate::instruction::generate_instruction_enum;

mod constructors;
mod execute;
mod instruction;
mod parse;
mod set;

fn main() {
    generate_instruction_enum();
    generate_execute();
    generate_constructors();
}
