use yavashark_bytecode::ConstValue;

pub struct DataSection {
    pub var_names: Vec<String>,
    pub constants: Vec<ConstValue>,
}