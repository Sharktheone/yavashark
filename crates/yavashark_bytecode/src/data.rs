use crate::ConstValue;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataSection {
    pub var_names: Vec<String>,
    pub constants: Vec<ConstValue>,
}

impl DataSection {
    #[must_use]
    pub const fn new(var_names: Vec<String>, constants: Vec<ConstValue>) -> Self {
        Self {
            var_names,
            constants,
        }
    }
}
