use crate::{Ctx, Value};

#[derive(Debug, PartialEq)]
pub struct Error<C: Ctx> {
    pub kind: ErrorKind<C>,
    pub stacktrace: StackTrace,
}

impl<C: Ctx> Error<C> {
    pub fn new(error: &str) -> Self {
        Self {
            kind: ErrorKind::Runtime(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn new_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Runtime(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn reference(error: &str) -> Self {
        Self {
            kind: ErrorKind::Reference(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn reference_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Reference(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn syn(error: &str) -> Self {
        Self {
            kind: ErrorKind::Syntax(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn syn_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Syntax(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn unknown(error: Option<String>) -> Self {
        Self {
            kind: ErrorKind::Error(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn unknown_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Error(Some(error)),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn ty(error: &str) -> Self {
        Self {
            kind: ErrorKind::Type(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn ty_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Type(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    pub fn throw(val: Value<C>) -> Self {
        Self {
            kind: ErrorKind::Throw(val),
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
            ErrorKind::Error(_) => "Error",
            ErrorKind::Throw(_) => "TODO: Throw",
        }
    }

    pub fn message(&self) -> String {
        match &self.kind {
            ErrorKind::Type(msg) => msg.clone(),
            ErrorKind::Reference(msg) => msg.clone(),
            ErrorKind::Range(msg) => msg.clone(),
            ErrorKind::Internal(msg) => msg.clone(),
            ErrorKind::Runtime(msg) => msg.clone(),
            ErrorKind::Syntax(msg) => msg.clone(),
            ErrorKind::Throw(val) => val.to_string(),
            ErrorKind::Error(msg) => msg.clone().unwrap_or(String::new()),
        }
    }

    pub fn stack(&self) -> &StackTrace {
        &self.stacktrace
    }

    pub fn file_name(&self) -> &str {
        self.stacktrace
            .frames
            .first()
            .map(|f| f.file.as_str())
            .unwrap_or("")
    }

    pub fn line_number(&self) -> u32 {
        self.stacktrace.frames.first().map(|f| f.line).unwrap_or(0)
    }

    pub fn column_number(&self) -> u32 {
        self.stacktrace
            .frames
            .first()
            .map(|f| f.column)
            .unwrap_or(0)
    }
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind<C: Ctx> {
    Type(String),
    Reference(String),
    Range(String),
    Internal(String),
    Runtime(String),
    Syntax(String),
    Throw(Value<C>),
    Error(Option<String>),
}

#[derive(Debug, PartialEq)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}

#[derive(Debug, PartialEq)]
pub struct StackFrame {
    pub function: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}
