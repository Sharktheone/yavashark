#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub stacktrace: StackTrace,
}

impl Error {
    pub fn new(error: String) -> Self {
        Self {
            kind: ErrorKind::RuntimeError(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn reference(error: String) -> Self {
        Self {
            kind: ErrorKind::ReferenceError(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn ty(error: String) -> Self {
        Self {
            kind: ErrorKind::TypeError(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    TypeError(String),
    ReferenceError(String),
    RangeError(String),
    InternalError(String),
    RuntimeError(String),
}

#[derive(Debug)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}

#[derive(Debug)]
pub struct StackFrame {
    pub function: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}
