use std::sync::LazyLock;
use crate::parse::{instruction_def, InstructionDefinition, Type};

struct InstructionSet {
    instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
enum ArgumentType {
    Variable,
    Reg,
    Acc,
    Stack,
    Const,
    Other(String),
}

impl ArgumentType {
    const NUM_TYPES: usize = 5;
    const TYPES: [ArgumentType; Self::NUM_TYPES] = [
        ArgumentType::Variable,
        ArgumentType::Reg,
        ArgumentType::Acc,
        ArgumentType::Stack,
        ArgumentType::Const,
    ];
    
    fn to_str(&self) -> &str {
        match self {
            ArgumentType::Variable => "Var",
            ArgumentType::Reg => "Reg",
            ArgumentType::Acc => "Acc",
            ArgumentType::Stack => "Stack",
            ArgumentType::Const => "Const",
            ArgumentType::Other(s) => s.as_str(),
        }
    }
}


#[derive(Debug, Clone)]
enum ReturnType {
    Variable,
    Reg,
    Acc,
    Stack,
    Other(String),
}

impl ReturnType {
    const NUM_TYPES: usize = 4;
    const TYPES: [ReturnType; Self::NUM_TYPES] = [
        ReturnType::Variable,
        ReturnType::Reg,
        ReturnType::Acc,
        ReturnType::Stack,
    ];
    
    fn to_str(&self) -> &str {
        match self {
            ReturnType::Variable => "Var",
            ReturnType::Reg => "Reg",
            ReturnType::Acc => "Acc",
            ReturnType::Stack => "Stack",
            ReturnType::Other(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
struct Instruction {
    name: String,
    inputs: Vec<ArgumentType>,
    output: Option<ReturnType>,
}


static INSTRUCTIONS: LazyLock<Vec<Instruction>> = LazyLock::new(expand_definitions);

pub fn instructions() -> &'static [Instruction] {
    &INSTRUCTIONS
}


fn expand_definitions() -> Vec<Instruction> {
    let instructions = instruction_def();
    
    let mut inst = Vec::new();
    
    for def in instructions {
        inst.extend(expand_definition(def));
    }

    println!("{:#?}", inst.len());
    
    inst
}

fn expand_definition(def: &InstructionDefinition) -> Vec<Instruction> {
    let mut inst = vec![Instruction {
        name: def.name.clone(),
        inputs: Vec::new(),
        output: None,
    }];
    
    
    for input in &def.inputs {
        if input == &Type::Data {
            repeat_vec(&mut inst, ArgumentType::NUM_TYPES);

            inst.iter_mut().zip(ArgumentType::TYPES.iter().cycle()).for_each(|(inst, input)| {
                inst.name.push_str(input.to_str());
                inst.inputs.push(input.clone());
            });
        } else {
            let ty = input.type_str();
            
            inst.iter_mut().for_each(|inst| {
                inst.inputs.push(ArgumentType::Other(ty.to_owned()));
            });
        }
    }
    
    if let Some(output) = &def.output {
        if output == &Type::Data {
            repeat_vec(&mut inst, ReturnType::NUM_TYPES);

            inst.iter_mut().zip(ReturnType::TYPES.iter().cycle()).for_each(|(inst, out)| {
                inst.name.push_str(out.to_str());
                inst.output = Some(out.clone());
            });
        } else {
            let ty = output.type_str();

            inst.iter_mut().for_each(|inst| {
                inst.output = Some(ReturnType::Other(ty.to_owned()));
            });
        }
    }
    
    inst
}




fn repeat_vec<T: Clone>(vec: &mut Vec<T>, times: usize) {
    let len = vec.len();
    vec.reserve(len * times);

    for _ in 0..times {
        vec.extend_from_within(0..len);
    }
}



#[test]
fn expand() {
    let insts = instructions();
    
    for inst in insts {
        println!("{:#?}", inst);
    }

    println!("{}", insts.len());
}