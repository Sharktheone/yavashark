#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub stacktrace: StackTrace,
}

impl Error {
    pub fn new(error: String) -> Self {
        Self {
            kind: ErrorKind::Runtime(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn reference(error: String) -> Self {
        Self {
            kind: ErrorKind::Reference(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }
    
    pub fn syntax_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Syntax(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }
    
    pub fn syntax(error: &str) -> Self {
        Self::syntax_error(error.to_string())
    }

    pub fn ty(error: String) -> Self {
        Self {
            kind: ErrorKind::Type(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }
    
    pub fn name(&self) -> &str {
        match &self.kind {
            ErrorKind::Type(_) => "TypeError",
            ErrorKind::Reference(_) => "ReferenceError",
            ErrorKind::Range(_) => "RangeError",
            ErrorKind::Internal(_) => "InternalError",
            ErrorKind::Runtime(_) => "RuntimeError",
            ErrorKind::Syntax(_) => "SyntaxError",
        }
    }
    
    pub fn message(&self) -> &str {
        match &self.kind {
            ErrorKind::Type(msg) => msg,
            ErrorKind::Reference(msg) => msg,
            ErrorKind::Range(msg) => msg,
            ErrorKind::Internal(msg) => msg,
            ErrorKind::Runtime(msg) => msg,
            ErrorKind::Syntax(msg) => msg,
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
    Type(String),
    Reference(String),
    Range(String),
    Internal(String),
    Runtime(String),
    Syntax(String),
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
