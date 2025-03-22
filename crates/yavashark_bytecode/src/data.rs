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


pub enum DataType {
    Acc(Acc),
    Reg(Reg),
    Var(VarName),
    Const(ConstIdx),
    Stack(Stack)
}


pub enum OutputDataType {
    Acc(Acc),
    Reg(Reg),
    Var(VarName),
    Stack(Stack)
}


pub trait Data: Copy {
    fn acc(self) -> Option<Acc> {
        None
    }

    fn reg(self) -> Option<Reg> {
        None
    }

    fn var_name(self) -> Option<VarName> {
        None
    }

    fn const_idx(self) -> Option<ConstIdx> {
        None
    }

    fn stack(self) -> Option<Stack> {
        None
    }

    fn data_type(self) -> DataType;
}

pub trait OutputData: Copy {
    fn data_type(self) -> OutputDataType;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Acc;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reg(pub u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarName(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstIdx(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stack(pub u32);

impl Data for Acc {
    fn acc(self) -> Option<Acc> {
        Some(self)
    }

    fn data_type(self) -> DataType {
        DataType::Acc(self)
    }
}
impl Data for Reg {
    fn reg(self) -> Option<Reg> {
        Some(self)
    }

    fn data_type(self) -> DataType {
        DataType::Reg(self)
    }
}
impl Data for VarName {
    fn var_name(self) -> Option<VarName> {
        Some(self)
    }

    fn data_type(self) -> DataType {
        DataType::Var(self)
    }
}
impl Data for ConstIdx {
    fn const_idx(self) -> Option<ConstIdx> {
        Some(self)
    }

    fn data_type(self) -> DataType {
        DataType::Const(self)
    }
}
impl Data for Stack {
    fn stack(self) -> Option<Stack> {
        Some(self)
    }

    fn data_type(self) -> DataType {
        DataType::Stack(self)
    }
}
