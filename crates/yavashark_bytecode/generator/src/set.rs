use std::sync::LazyLock;
use convert_case::{Case, Casing};
use crate::parse::{instruction_def, InstructionDefinition, Type};

struct InstructionSet {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum ArgumentType {
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
    
    pub fn to_syn(&self) -> syn::Type {
        match self {
            ArgumentType::Variable => syn::parse_quote! { VarName },
            ArgumentType::Reg => syn::parse_quote! { Reg },
            ArgumentType::Acc => syn::parse_quote! { Acc },
            ArgumentType::Stack => syn::parse_quote! { Stack },
            ArgumentType::Const => syn::parse_quote! { ConstIdx },
            ArgumentType::Other(s) => syn::parse_str(s).unwrap(),
        }
    }
}


#[derive(Debug, Clone)]
pub enum ReturnType {
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

    pub fn to_syn(&self) -> syn::Type {
        match self {
            ReturnType::Variable => syn::parse_quote! { VarName },
            ReturnType::Reg => syn::parse_quote! { Reg },
            ReturnType::Acc => syn::parse_quote! { Acc },
            ReturnType::Stack => syn::parse_quote! { Stack },
            ReturnType::Other(s) => syn::parse_str(s).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub class: String,
    pub name: String,
    pub inputs: Vec<ArgumentType>,
    pub output: Option<ReturnType>,
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
        class: get_class(&def.name),
        name: def.name.clone(),
        inputs: Vec::new(),
        output: None,
    }];
    
    
    for input in &def.inputs {
        if input == &Type::Data {
            let len = inst.len();
            repeat_vec(&mut inst, ArgumentType::NUM_TYPES);

            inst.iter_mut().enumerate().for_each(|(idx, inst)| {
                let input = ArgumentType::TYPES[idx / len].clone();
                
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
            let len = inst.len();
            repeat_vec(&mut inst, ReturnType::NUM_TYPES);

            inst.iter_mut().enumerate().for_each(|(idx, inst)| {
                let out = ReturnType::TYPES[idx / len].clone();
                
                inst.name.push_str("To");
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

    for _ in 0..times - 1 {
        vec.extend_from_within(0..len);
    }
}

pub fn get_class(name: &str) -> String {
    let case = name.to_case(Case::Snake);
    
    match &*case {
        "mod" => "mod_".to_owned(),
        "in" => "in_".to_owned(),
        "move" => "move_".to_owned(),
        "return" => "return_".to_owned(),
        _ => case,
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