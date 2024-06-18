use crate::{Ctx, Value};

#[derive(Debug, PartialEq, Eq)]
pub struct Error<C: Ctx> {
    pub kind: ErrorKind<C>,
    pub stacktrace: StackTrace,
}

impl<C: Ctx> Error<C> {
    #[must_use]
    pub fn new(error: &str) -> Self {
        Self {
            kind: ErrorKind::Runtime(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn new_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Runtime(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn reference(error: &str) -> Self {
        Self {
            kind: ErrorKind::Reference(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn reference_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Reference(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn syn(error: &str) -> Self {
        Self {
            kind: ErrorKind::Syntax(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn syn_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Syntax(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn unknown(error: Option<String>) -> Self {
        Self {
            kind: ErrorKind::Error(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn unknown_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Error(Some(error)),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn ty(error: &str) -> Self {
        Self {
            kind: ErrorKind::Type(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn ty_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Type(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub fn throw(val: Value<C>) -> Self {
        Self {
            kind: ErrorKind::Throw(val),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub const fn name(&self) -> &str {
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

    #[must_use]
    pub fn message(&self) -> String {
        match &self.kind {
            ErrorKind::Type(msg)
            | ErrorKind::Reference(msg)
            | ErrorKind::Range(msg)
            | ErrorKind::Internal(msg)
            | ErrorKind::Runtime(msg)
            | ErrorKind::Syntax(msg) => msg.clone(),
            ErrorKind::Throw(val) => val.to_string(),
            ErrorKind::Error(msg) => msg.clone().unwrap_or(String::new()),
        }
    }

    #[must_use]
    pub const fn stack(&self) -> &StackTrace {
        &self.stacktrace
    }

    #[must_use]
    pub fn file_name(&self) -> &str {
        self.stacktrace
            .frames
            .first()
            .map_or("", |f| f.file.as_str())
    }

    #[must_use]
    pub fn line_number(&self) -> u32 {
        self.stacktrace.frames.first().map_or(0, |f| f.line)
    }

    #[must_use]
    pub fn column_number(&self) -> u32 {
        self.stacktrace.frames.first().map_or(0, |f| f.column)
    }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StackFrame {
    pub function: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "anyhow")]
mod anyhow_impl {

    //TODO: Maybe we can integrate the JS stacktrace into the native Rust stacktrace?

    use std::fmt::Display;

    use super::*;

    #[derive(Debug)]
    struct SyncError {
        message: String,
        stack: StackTrace,
    }

    impl SyncError {
        fn new(message: String, stack: StackTrace) -> Self {
            Self { message, stack }
        }
    }

    impl Display for SyncError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for SyncError {}
}

impl<T: std::error::Error, C: Ctx> From<T> for Error<C> {
    fn from(value: T) -> Self {
        Self::new_error(value.to_string())
    }
}
