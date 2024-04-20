pub struct Error {
    pub kind: ErrorKind,
    pub stacktrace: StackTrace,
}

pub enum ErrorKind {
    TypeError(String),
    ReferenceError(String),
    RangeError(String),
    InternalError(String),
}

pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}

pub struct StackFrame {
    pub function: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}
