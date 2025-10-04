use crate::error::ErrorKind;
use crate::print::PrettyPrint;
use crate::{Error, Realm};
use std::fmt::Write;

impl PrettyPrint for Error {
    fn pretty_print_key(&self, _: &mut Realm) -> String {
        self.name().to_string()
    }

    fn pretty_print_circular(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String {
        let msg = error_message_pretty_circular(self, not, realm);

        let mut buf = String::new();

        if msg.is_empty() {
            _ = write!(buf, "{}", self.name());
        } else {
            _ = write!(buf, "{}: {}\n{}", self.name(), msg, self.stacktrace);
        }

        buf
    }

    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String {
        let msg = error_message_pretty_circular_nl(self, not, realm);

        let mut buf = String::new();

        if msg.is_empty() {
            _ = writeln!(buf, "{}", self.name());
        } else {
            _ = writeln!(buf, "{}: {}\n{}", self.name(), msg, self.stacktrace);
        }

        buf
    }
}

fn error_message_pretty_circular(error: &Error, not: &mut Vec<usize>, realm: &mut Realm) -> String {
    match &error.kind {
        ErrorKind::Type(msg)
        | ErrorKind::Reference(msg)
        | ErrorKind::Range(msg)
        | ErrorKind::Internal(msg)
        | ErrorKind::Runtime(msg)
        | ErrorKind::Eval(msg)
        | ErrorKind::URI(msg)
        | ErrorKind::Aggregate(msg)
        | ErrorKind::Syntax(msg) => msg.to_string(),
        ErrorKind::Throw(val) => val.pretty_print_circular(not, realm),
        ErrorKind::Error(msg) => msg
            .as_ref()
            .map_or(String::new(), std::string::ToString::to_string),
    }
}

fn error_message_pretty_circular_nl(error: &Error, not: &mut Vec<usize>, realm: &mut Realm) -> String {
    match &error.kind {
        ErrorKind::Type(msg)
        | ErrorKind::Reference(msg)
        | ErrorKind::Range(msg)
        | ErrorKind::Internal(msg)
        | ErrorKind::Runtime(msg)
        | ErrorKind::Eval(msg)
        | ErrorKind::URI(msg)
        | ErrorKind::Aggregate(msg)
        | ErrorKind::Syntax(msg) => msg.to_string(),
        ErrorKind::Throw(val) => val.pretty_print_circular_nl(not, realm),
        ErrorKind::Error(msg) => msg
            .as_ref()
            .map_or(String::new(), std::string::ToString::to_string),
    }
}
