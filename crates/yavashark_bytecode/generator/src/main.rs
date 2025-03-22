use crate::constructors::generate_constructors;
use crate::execute::generate_execute;
use crate::instruction::generate_instruction_enum;

mod parse;
mod instruction;
mod set;
mod execute;
mod constructors;

fn main() {
    generate_instruction_enum();
    generate_execute();
    generate_constructors();
}
