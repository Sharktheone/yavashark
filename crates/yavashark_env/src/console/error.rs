use crate::print::PrettyPrint;
use crate::Error;
use std::fmt::Write;
use yavashark_value::ErrorKind;

impl PrettyPrint for Error {
    fn pretty_print_key(&self) -> String {
        self.name().to_string()
    }

    fn pretty_print_circular(&self, not: &mut Vec<usize>) -> String {
        let msg = error_message_pretty_circular(self, not);

        let mut buf = String::new();

        if msg.is_empty() {
            _ = write!(buf, "{}", self.name());
        } else {
            _ = write!(buf, "{}: {}\n{}", self.name(), msg, self.stacktrace);
        }

        buf
    }

    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>) -> String {
        let msg = error_message_pretty_circular_nl(self, not);

        let mut buf = String::new();

        if msg.is_empty() {
            _ = writeln!(buf, "{}", self.name());
        } else {
            _ = writeln!(buf, "{}: {}\n{}", self.name(), msg, self.stacktrace);
        }

        buf
    }
}

fn error_message_pretty_circular(error: &Error, not: &mut Vec<usize>) -> String {
    match &error.kind {
        ErrorKind::Type(msg)
        | ErrorKind::Reference(msg)
        | ErrorKind::Range(msg)
        | ErrorKind::Internal(msg)
        | ErrorKind::Runtime(msg)
        | ErrorKind::Syntax(msg) => msg.clone(),
        ErrorKind::Throw(val) => val.pretty_print_circular(not),
        ErrorKind::Error(msg) => msg.clone().unwrap_or(String::new()),
    }
}

fn error_message_pretty_circular_nl(error: &Error, not: &mut Vec<usize>) -> String {
    match &error.kind {
        ErrorKind::Type(msg)
        | ErrorKind::Reference(msg)
        | ErrorKind::Range(msg)
        | ErrorKind::Internal(msg)
        | ErrorKind::Runtime(msg)
        | ErrorKind::Syntax(msg) => msg.clone(),
        ErrorKind::Throw(val) => val.pretty_print_circular_nl(not),
        ErrorKind::Error(msg) => msg.clone().unwrap_or(String::new()),
    }
}
