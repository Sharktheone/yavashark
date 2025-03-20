static INSTRUCTION_SET: &str = include_str!("../../set.instruct");



#[derive(Debug, Clone)]
pub enum Type {
    Data,
    Offset,
    Other(String)
}

impl Type {
    fn parse(s: &str) -> Self {
        match s.trim() {
            "Data" => Type::Data,
            "Offset" => Type::Offset,
            _ => Type::Other(s.to_string())
        }
    }
}


#[derive(Debug, Clone)]
pub struct InstructionDefinition {
    pub name: String,
    pub inputs: Vec<Type>,
    pub output: Option<Type>,
}



fn parse_instruction_set() -> Vec<InstructionDefinition> {
    let mut instructions = Vec::new();
    for line in INSTRUCTION_SET.lines() {
        if line.is_empty() {
            continue;
        }

        let InstructionLine { name, inputs, output } = split_in_parts(line);

        
        let inputs = if let Some(inputs) = inputs {
            inputs.split(',').map(Type::parse).collect()
        } else {
            Vec::new()
        };




        instructions.push(InstructionDefinition {
            name: name.to_string(),
            inputs,
            output: output.map(Type::parse),
        });
    }
    instructions
}


pub struct InstructionLine<'a> {
    name: &'a str,
    inputs: Option<&'a str>,
    output: Option<&'a str>,
}

fn split_in_parts(line: &str) -> InstructionLine {
    let parts = line.split_once(['(', ' ']);
    let name = parts.map(|x| x.0).unwrap_or(line);
    
    let in_ret = parts.map(|x| x.1).map(parse_args_and_ret);
    
    let inputs = in_ret.and_then(|(i, _)| i);
    let output = in_ret.and_then(|(_, o)| o);

    InstructionLine {
        name,
        inputs,
        output,
    }
}

fn parse_args_and_ret(rest: &str) -> (Option<&str>, Option<&str>) {
    if rest.is_empty() {
        return (None, None);
    }
    
    
    if let Some(parts) = rest.split_once(')') {
        let input = parts.0;
        let output = parts.1.trim().strip_prefix("=>").map(str::trim);
        
        (Some(input), output)
    } else {
        let ret = rest.trim().strip_prefix("=>").map(str::trim);
        
        (None, ret)
    }
}


#[test]
fn parse() {
    let set = parse_instruction_set();


    dbg!(set);
}
