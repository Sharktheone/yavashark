use crate::error::ErrorKind;
use crate::error_obj::ErrorObj;
use crate::print::{PrettyObjectOverride, PrettyPrint};
use crate::{Error, Realm};
use std::fmt::Write;

impl PrettyObjectOverride for ErrorObj {
    fn pretty_inline(
        &self,
        _obj: &crate::value::Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        let inner = self.inner.try_borrow().ok()?;

        Some(inner.error.pretty_print_circular(not, realm))
    }
}

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
            _ = write!(buf, "{}: {}", self.name(), msg);
        }

        if !self.stacktrace.frames.is_empty() {
            _ = write!(buf, "\n{}", self.stacktrace);
            buf.pop();
        }

        buf
    }

    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String {
        let msg = error_message_pretty_circular_nl(self, not, realm);

        let mut buf = String::new();

        if msg.is_empty() {
            _ = writeln!(buf, "{}", self.name());
        } else {
            _ = writeln!(buf, "{}: {}", self.name(), msg);
        }

        _ = write!(buf, "{}", self.stacktrace);

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
        | ErrorKind::Suppressed(msg)
        | ErrorKind::Syntax(msg) => msg.to_string(),
        ErrorKind::Throw(val) => val.pretty_print_circular(not, realm),
        ErrorKind::Error(msg) => msg
            .as_ref()
            .map_or(String::new(), std::string::ToString::to_string),
    }
}

fn error_message_pretty_circular_nl(
    error: &Error,
    not: &mut Vec<usize>,
    realm: &mut Realm,
) -> String {
    match &error.kind {
        ErrorKind::Type(msg)
        | ErrorKind::Reference(msg)
        | ErrorKind::Range(msg)
        | ErrorKind::Internal(msg)
        | ErrorKind::Runtime(msg)
        | ErrorKind::Eval(msg)
        | ErrorKind::URI(msg)
        | ErrorKind::Aggregate(msg)
        | ErrorKind::Suppressed(msg)
        | ErrorKind::Syntax(msg) => msg.to_string(),
        ErrorKind::Throw(val) => val.pretty_print_circular_nl(not, realm),
        ErrorKind::Error(msg) => msg
            .as_ref()
            .map_or(String::new(), std::string::ToString::to_string),
    }
}
