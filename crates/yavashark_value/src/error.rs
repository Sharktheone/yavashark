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
    
    pub fn name(&self) -> &str {
        match &self.kind {
            ErrorKind::TypeError(_) => "TypeError",
            ErrorKind::ReferenceError(_) => "ReferenceError",
            ErrorKind::RangeError(_) => "RangeError",
            ErrorKind::InternalError(_) => "InternalError",
            ErrorKind::RuntimeError(_) => "RuntimeError",
        }
    }
    
    pub fn message(&self) -> &str {
        match &self.kind {
            ErrorKind::TypeError(msg) => msg,
            ErrorKind::ReferenceError(msg) => msg,
            ErrorKind::RangeError(msg) => msg,
            ErrorKind::InternalError(msg) => msg,
            ErrorKind::RuntimeError(msg) => msg,
        }
    }
    
    pub fn stack(&self) -> &StackTrace {
        &self.stacktrace
    }
    
    pub fn file_name(&self) -> &str {
        self.stacktrace.frames.first().map(|f| f.file.as_str()).unwrap_or("")
    }
    
    pub fn line_number(&self) -> u32 {
        self.stacktrace.frames.first().map(|f| f.line).unwrap_or(0)
    }
    
    pub fn column_number(&self) -> u32 {
        self.stacktrace.frames.first().map(|f| f.column).unwrap_or(0)
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
