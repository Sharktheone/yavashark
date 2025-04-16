use crate::ConstValue;
use crate::control::ControlBlock;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataSection {
    pub var_names: Vec<String>,
    pub labels: Vec<String>,
    pub constants: Vec<ConstValue>,
    pub control: Vec<ControlBlock>
}

impl DataSection {
    #[must_use]
    pub const fn new(
        var_names: Vec<String>,
        labels: Vec<String>,
        constants: Vec<ConstValue>,
        control: Vec<ControlBlock>,
    ) -> Self {
        Self {
            var_names,
            labels,
            constants,
            control,
        }
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

impl From<OutputDataType> for DataType {
    fn from(val: OutputDataType) -> Self {
        match val {
            OutputDataType::Acc(acc) => Self::Acc(acc),
            OutputDataType::Reg(reg) => Self::Reg(reg),
            OutputDataType::Var(variable) => Self::Var(variable),
            OutputDataType::Stack(stack) => Self::Stack(stack),
        }
    }
}

impl From<Acc> for DataType {
    fn from(val: Acc) -> Self {
        Self::Acc(val)
    }
}

impl From<Reg> for DataType {
    fn from(val: Reg) -> Self {
        Self::Reg(val)
    }
}

impl From<VarName> for DataType {
    fn from(val: VarName) -> Self {
        Self::Var(val)
    }
}

impl From<ConstIdx> for DataType {
    fn from(val: ConstIdx) -> Self {
        Self::Const(val)
    }
}

impl From<Stack> for DataType {
    fn from(val: Stack) -> Self {
        Self::Stack(val)
    }
}

impl TryFrom<DataType> for OutputDataType {
    type Error = ();

    fn try_from(value: DataType) -> Result<Self, Self::Error> {
        match value {
            DataType::Acc(acc) => Ok(Self::Acc(acc)),
            DataType::Reg(reg) => Ok(Self::Reg(reg)),
            DataType::Var(var) => Ok(Self::Var(var)),
            DataType::Stack(stack) => Ok(Self::Stack(stack)),
            DataType::Const(_) => Err(()),
        }
    }
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
pub struct ControlIdx(pub u32);

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

impl Data for OutputDataType {
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
        None
    }

    fn stack(self) -> Option<Stack> {
        match self {
            Self::Stack(stack) => Some(stack),
            _ => None,
        }
    }

    fn data_type(self) -> DataType {
        self.into()
    }
}
