use yavashark_bytecode::ConstValue;

pub struct DataSection {
    pub var_names: Vec<String>,
    pub constants: Vec<ConstValue>,
}

impl DataSection {
    pub fn new(var_names: Vec<String>, constants: Vec<ConstValue>) -> Self {
        Self {
            var_names,
            constants,
        }
    }
}
