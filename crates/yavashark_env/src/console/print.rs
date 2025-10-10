use colored::Colorize;

use crate::array::Array;
use crate::builtins::RegExp;
use crate::value::{Object, Value};
use crate::{PrimitiveValue, PropertyKey, Realm};

pub trait PrettyObjectOverride {
    fn pretty_inline(
        &self,
        obj: &Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String>;

    fn pretty_multiline(
        &self,
        obj: &Object,
        not: &mut Vec<usize>,
        realm: &mut Realm,
    ) -> Option<String> {
        self.pretty_inline(obj, not, realm)
    }
}

pub trait PrettyPrint {
    fn pretty_print(&self, realm: &mut Realm) -> String {
        self.pretty_print_circular(&mut Vec::new(), realm)
    }

    #[allow(unused)]
    fn pretty_print_nl(&self, realm: &mut Realm) -> String {
        self.pretty_print_circular_nl(&mut Vec::new(), realm)
    }

    fn pretty_print_key(&self, realm: &mut Realm) -> String;

    fn pretty_print_circular(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String;
    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String;
}

impl PrettyPrint for Object {
    fn pretty_print_key(&self, _realm: &mut Realm) -> String {
        format!("'{self}'").green().to_string()
    }

    fn pretty_print_circular(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String {
        if let Some(array) = self.downcast::<Array>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*array, self, not, realm) {
                return s;
            }
        }
        if let Some(re) = self.downcast::<RegExp>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*re, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::Date>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::Duration>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::Instant>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainDate>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainDateTime>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainMonthDay>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainTime>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainYearMonth>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::ZonedDateTime>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not, realm) {
                return s;
            }
        }

        let id = self.id();

        if not.contains(&id) {
            return "[Circular *1]".bright_green().to_string();
        }

        not.push(id);

        let mut str = if self.is_callable() {
            format!("[Function: {}] ", self.name())
                .bright_green()
                .to_string()
        } else if let Some(prim) = self.primitive(realm).ok().flatten() {
            match prim {
                PrimitiveValue::Null => "[Null] ".bright_green().to_string(),
                PrimitiveValue::Undefined => "[Undefined] ".bright_green().to_string(),
                PrimitiveValue::String(s) => {
                    format!("[String: \"{s}\"] ").bright_green().to_string()
                }
                PrimitiveValue::Number(n) => format!("[Number: {n}] ").bright_green().to_string(),
                PrimitiveValue::Boolean(b) => format!("[Boolean: {b}] ").bright_green().to_string(),
                _ => "[Primitive] ".bright_green().to_string(),
            }
        } else {
            let name = self.name();
            if name == "Object" {
                String::new()
            } else {
                format!("[{name}] ").bright_green().to_string()
            }
        };

        str.push_str("{ ");

        if let Ok(properties) = self.properties(realm) {
            if properties.is_empty() {
                not.pop();

                str.pop();
                str.push('}');
                return str;
            }

            for (key, value) in properties {
                str.push_str(&key.pretty_print_key(realm));
                str.push_str(": ");
                str.push_str(&value.pretty_print_circular(not, realm));
                str.push_str(", ");
            }
            str.pop();
            str.pop();
        } else {
            str.pop();
            str.push('}');
        }

        str.push_str(" }");
        not.pop();
        str
    }

    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String {
        // Try type-specific overrides first
        if let Some(array) = self.downcast::<crate::object::array::Array>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*array, self, not, realm,
            ) {
                return s;
            }
        }
        if let Some(re) = self.downcast::<crate::builtins::RegExp>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*re, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::Date>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::Duration>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::Instant>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainDate>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainDateTime>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainMonthDay>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainTime>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainYearMonth>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::ZonedDateTime>() {
            if let Some(s) = crate::console::print::PrettyObjectOverride::pretty_multiline(
                &*date, self, not, realm,
            ) {
                return s;
            }
        }

        let id = self.id();

        if not.contains(&id) {
            return "[Circular *1]".bright_green().to_string();
        }

        not.push(id);

        let mut str = if self.is_callable() {
            format!("[Function: {}] ", self.name())
                .bright_green()
                .to_string()
        } else if let Some(prim) = self.primitive(realm).ok().flatten() {
            match prim {
                PrimitiveValue::Null => "[Null]".bright_green().to_string(),
                PrimitiveValue::Undefined => "[Undefined]".bright_green().to_string(),
                PrimitiveValue::String(s) => {
                    format!("[String: \"{s}\"]").bright_green().to_string()
                }
                PrimitiveValue::Number(n) => format!("[Number: {n}]").bright_green().to_string(),
                PrimitiveValue::Boolean(b) => format!("[Boolean: {b}]").bright_green().to_string(),
                _ => "[Primitive]".bright_green().to_string(),
            }
        } else {
            let name = self.name();
            if name == "Object" {
                String::new()
            } else {
                format!("[{name}] ").bright_green().to_string()
            }
        };

        str.push_str("{\n");

        if let Ok(properties) = self.properties(realm) {
            if properties.is_empty() {
                not.pop();
                str.pop();
                str.push('}');
                return str;
            }

            for (key, value) in properties {
                str.push_str("  ");
                str.push_str(&key.pretty_print_key(realm));
                str.push_str(": ");
                str.push_str(&value.pretty_print_circular_nl(not, realm));
                str.push_str(",\n");
            }
            str.pop();
            str.pop();
        } else {
            str.pop();
            str.push('}');
            return str;
        }

        str.push_str("\n}");

        not.pop();

        str
    }
}

impl PrettyPrint for Value {
    fn pretty_print_key(&self, realm: &mut Realm) -> String {
        match self {
            Self::Undefined => "undefined".to_string(),
            Self::Null => "null".to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Number(n) => n.to_string(),
            Self::String(s) => s.to_string(),
            Self::Object(o) => o.pretty_print_key(realm),
            Self::Symbol(s) => s.to_string(),
            Self::BigInt(b) => b.to_string(),
        }
    }

    fn pretty_print_circular(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => format!("\"{s}\"").green().to_string(),
            Self::Object(o) => o.pretty_print_circular(not, realm),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
            Self::BigInt(b) => {
                let mut str = b.to_string();
                str.push('n');

                str.bright_yellow().to_string()
            }
        }
    }

    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>, realm: &mut Realm) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => format!("\"{s}\"").green().to_string(),
            Self::Object(o) => o.pretty_print_circular_nl(not, realm),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
            Self::BigInt(b) => {
                let mut str = b.to_string();
                str.push('n');

                str.bright_yellow().to_string()
            }
        }
    }
}

pub fn fmt_properties_to(obj: &Object, str: &mut String, not: &mut Vec<usize>, realm: &mut Realm) {
    let Ok(properties) = obj.properties(realm) else {
        return;
    };

    if properties.is_empty() {
        return;
    }

    str.push_str(" { ");

    for (key, value) in properties {
        str.push_str(&key.pretty_print_key(realm));
        str.push_str(": ");
        str.push_str(&value.pretty_print_circular(not, realm));
        str.push_str(", ");
    }
    str.pop();
    str.pop();

    str.push_str(" }");
}

impl PrettyPrint for PropertyKey {
    fn pretty_print_key(&self, _realm: &mut Realm) -> String {
        match self {
            Self::String(s) => format!("\"{s}\"").green().to_string(),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
        }
    }

    fn pretty_print_circular(&self, _not: &mut Vec<usize>, realm: &mut Realm) -> String {
        self.pretty_print_key(realm)
    }

    fn pretty_print_circular_nl(&self, _not: &mut Vec<usize>, realm: &mut Realm) -> String {
        self.pretty_print_key(realm)
    }
}
