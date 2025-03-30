use crate::ConstValue;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataSection {
    pub var_names: Vec<String>,
    pub labels: Vec<String>,
    pub constants: Vec<ConstValue>,
}

impl DataSection {
    #[must_use]
    pub const fn new(var_names: Vec<String>, labels: Vec<String>, constants: Vec<ConstValue>) -> Self {
        Self { var_names, labels, constants }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Acc(Acc),
    Reg(Reg),
    Var(VarName),
    Const(ConstIdx),
    Stack(Stack),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputDataType {
    Acc(Acc),
    Reg(Reg),
    Var(VarName),
    Stack(Stack),
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

pub type TryIdx = ConstIdx;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stack(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Label(pub u32);

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

impl Data for DataType {
    
    fn acc(self) -> Option<Acc> {
        match self {
            Self::Acc(acc) => Some(acc),
            _ => None,
        }
    }
    
    fn reg(self) -> Option<Reg> {
        match self {
            Self::Reg(reg) => Some(reg),
            _ => None,
        }
    }
    
    fn var_name(self) -> Option<VarName> {
        match self {
            Self::Var(var) => Some(var),
            _ => None,
        }
    }
    
    fn const_idx(self) -> Option<ConstIdx> {
        match self {
            Self::Const(const_idx) => Some(const_idx),
            _ => None,
        }
    }
    
    fn stack(self) -> Option<Stack> {
        match self {
            Self::Stack(stack) => Some(stack),
            _ => None,
        }
    }
    
    fn data_type(self) -> DataType {
        self
    }
}

impl OutputData for Acc {
    fn data_type(self) -> OutputDataType {
        OutputDataType::Acc(self)
    }
}

impl OutputData for Reg {
    fn data_type(self) -> OutputDataType {
        OutputDataType::Reg(self)
    }
}

impl OutputData for VarName {
    fn data_type(self) -> OutputDataType {
        OutputDataType::Var(self)
    }
}

impl OutputData for Stack {
    fn data_type(self) -> OutputDataType {
        OutputDataType::Stack(self)
    }
}

impl OutputData for OutputDataType {
    fn data_type(self) -> OutputDataType {
        self
    }
}
