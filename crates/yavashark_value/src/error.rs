use crate::{Realm, Value};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Range;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error<C: Realm> {
    pub kind: ErrorKind<C>,
    pub stacktrace: StackTrace,
}

impl<C: Realm> Error<C> {
    #[must_use]
    pub fn new(error: &str) -> Self {
        Self {
            kind: ErrorKind::Runtime(error.to_string()),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub const fn new_error(error: String) -> Self {
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
    pub const fn reference_error(error: String) -> Self {
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
    pub const fn syn_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Syntax(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub const fn unknown(error: Option<String>) -> Self {
        Self {
            kind: ErrorKind::Error(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub const fn unknown_error(error: String) -> Self {
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
    pub const fn ty_error(error: String) -> Self {
        Self {
            kind: ErrorKind::Type(error),
            stacktrace: StackTrace { frames: vec![] },
        }
    }

    #[must_use]
    pub const fn throw(val: Value<C>) -> Self {
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

    pub fn message(&self, realm: &mut C) -> Result<String, Self> {
        Ok(match &self.kind {
            ErrorKind::Type(msg)
            | ErrorKind::Reference(msg)
            | ErrorKind::Range(msg)
            | ErrorKind::Internal(msg)
            | ErrorKind::Runtime(msg)
            | ErrorKind::Syntax(msg) => msg.clone(),
            ErrorKind::Throw(val) => val.to_string(realm)?,
            ErrorKind::Error(msg) => msg.clone().unwrap_or(String::new()),
        })
    }

    #[must_use]
    pub fn message_internal(&self) -> String {
        match &self.kind {
            ErrorKind::Type(msg)
            | ErrorKind::Reference(msg)
            | ErrorKind::Range(msg)
            | ErrorKind::Internal(msg)
            | ErrorKind::Runtime(msg)
            | ErrorKind::Syntax(msg) => msg.clone(),
            ErrorKind::Throw(val) => format!("{val}"),
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
            .map_or("", |f| f.loc.file())
    }

    #[must_use]
    pub fn line_number(&self) -> u32 {
        self.stacktrace.frames.first().map_or(0, |f| f.loc.line())
    }

    #[must_use]
    pub fn column_number(&self) -> u32 {
        self.stacktrace.frames.first().map_or(0, |f| f.loc.column())
    }


    pub fn attach_location(&mut self, loc: Location) {
        self.stacktrace.attach_location(loc)
    }

    pub fn attach_function_stack(&mut self, function: String, loc: Location) {
        self.stacktrace.attach_function_stack(function, loc)
    }
}

impl<C: Realm> Display for Error<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = self.message_internal();

        if msg.is_empty() {
            write!(f, "{}", self.name())
        } else {
            write!(f, "{}: {}\n{}", self.name(), msg, self.stacktrace)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorKind<C: Realm> {
    Type(String),
    Reference(String),
    Range(String),
    Internal(String),
    Runtime(String),
    Syntax(String),
    Throw(Value<C>),
    Error(Option<String>),
}





#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}




impl Display for StackTrace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for t in &self.frames {
            Display::fmt(t, f)?
        }
        
        Ok(())
    }
    
}



impl StackTrace {
    fn attach_location(&mut self, loc: Location) {
        if self.frames.is_empty() {
            self.frames.push(StackFrame {
                loc,
                function: String::new(),
            })
        }
    }


    fn attach_function_stack(&mut self, function: String, loc: Location) {
        self.frames.push(StackFrame {
            loc,
            function,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackFrame {
    pub function: String,
    pub loc: Location,
}


impl Display for StackFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "    at {} ({}:{})", self.function, self.loc.file(), self.loc.line()) 
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Location {
    Source {
        range: Range<u32>,
        path: PathBuf,
    },
    SourceRange {
        //path is unknown
        range: Range<u32>
    },
    Native {
        path: PathBuf,
        line: u32,
        column: u32,
    },
    NativeUnknown,
}

impl Location {
    fn file(&self) -> &str {
        match self {
            Self::Source { path, ..} => {
                path.to_str().unwrap_or("<unknown>")
            }
            Self::SourceRange {..} => {
                "<unknown>"
            }
            Self::Native { path, ..} => {
                path.to_str().unwrap_or("<unknown>")
            }

            Location::NativeUnknown => {
                "<unknown>"
            }
        }
    }

    fn line(&self) -> u32 {
        match self {
            Self::Source { range, path} => {
                line_of_range(range.clone(), path)
            }
            Self::SourceRange { .. } => {
                0
            }
            Self::Native { line, ..} => {
                *line
            }

            Location::NativeUnknown => {
                0
            }
        }
    }

    fn column(&self) -> u32 {
        match self {
            Self::Source { range, path} => {
                col_of_range(range.clone(), path.as_path())
            }
            Self::SourceRange { .. } => {
                0
            }
            Self::Native { column, ..} => {
                *column
            }

            Location::NativeUnknown => {
                0
            }
        }
    }
}


fn line_of_range(range: Range<u32>, path: &Path) -> u32 {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut total_chars = 0u32;
    for (line_number, line) in reader.lines().enumerate() {
        let line = line.expect("Failed to read line");
        let line_length = line.len() as u32 + 1;

        if total_chars + line_length > range.start {
            return (line_number + 1) as u32;
        }

        total_chars += line_length;
    }

    0
}

fn col_of_range(range: Range<u32>, path: &Path) -> u32 {
    let file = File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut total_chars = 0u32;
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line_length = line.len() as u32 + 1;

        if total_chars + line_length > range.start {
            return range.start - total_chars + 1;
        }

        total_chars += line_length;
    }

    0 
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

impl<T: std::error::Error, C: Realm> From<T> for Error<C> {
    fn from(value: T) -> Self {
        Self::new_error(value.to_string())
    }
}
