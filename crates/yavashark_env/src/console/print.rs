use colored::Colorize;

use crate::array::Array;
use crate::builtins::RegExp;
use crate::realm::Realm;
use yavashark_value::{Object, Value};

pub trait PrettyObjectOverride {
    fn pretty_inline(&self, obj: &Object<Realm>, not: &mut Vec<usize>) -> Option<String>;

    fn pretty_multiline(&self, obj: &Object<Realm>, not: &mut Vec<usize>) -> Option<String> {
        self.pretty_inline(obj, not)
    }
}

pub trait PrettyPrint {
    fn pretty_print(&self) -> String {
        self.pretty_print_circular(&mut Vec::new())
    }

    #[allow(unused)]
    fn pretty_print_nl(&self) -> String {
        self.pretty_print_circular_nl(&mut Vec::new())
    }

    fn pretty_print_key(&self) -> String;

    fn pretty_print_circular(&self, not: &mut Vec<usize>) -> String;
    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>) -> String;
}

impl PrettyPrint for Object<Realm> {
    fn pretty_print_key(&self) -> String {
        format!("'{self}'").green().to_string()
    }

    fn pretty_print_circular(&self, not: &mut Vec<usize>) -> String {
        if let Some(array) = self.downcast::<Array>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*array, self, not) {
                return s;
            }
        }
        if let Some(re) = self.downcast::<RegExp>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*re, self, not) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::Date>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::Duration>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::Instant>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not) {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainDate>() {
            if let Some(s) = PrettyObjectOverride::pretty_inline(&*date, self, not) {
                return s;
            }
        }


        let id = self.id();

        if not.contains(&id) {
            return "[Circular *1]".bright_green().to_string();
        }

        not.push(id);

        let mut str = if self.is_function() {
            format!("[Function: {}] ", self.name())
                .bright_green()
                .to_string()
        } else if let Some(prim) = self.primitive() {
            match prim {
                Value::Null => "[Null] ".bright_green().to_string(),
                Value::Undefined => "[Undefined] ".bright_green().to_string(),
                Value::String(s) => format!("[String: \"{s}\"] ").bright_green().to_string(),
                Value::Number(n) => format!("[Number: {n}] ").bright_green().to_string(),
                Value::Boolean(b) => format!("[Boolean: {b}] ").bright_green().to_string(),
                _ => "[Primitive] ".bright_green().to_string(),
            }
        } else {
            String::new()
        };

        str.push_str("{ ");

        if let Ok(properties) = self.properties() {
            if properties.is_empty() {
                not.pop();
                if str.len() == 2 {
                    return "{}".to_string();
                }
                str.pop();
                str.pop();
                return str;
            }

            for (key, value) in properties {
                str.push_str(&key.pretty_print_key());
                str.push_str(": ");
                str.push_str(&value.pretty_print_circular(not));
                str.push_str(", ");
            }
            str.pop();
            str.pop();
        } else {
            str.push_str("{}");
        };

        str.push_str(" }");
        not.pop();
        str
    }

    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>) -> String {
        // Try type-specific overrides first
        if let Some(array) = self.downcast::<crate::object::array::Array>() {
            if let Some(s) =
                crate::console::print::PrettyObjectOverride::pretty_multiline(&*array, self, not)
            {
                return s;
            }
        }
        if let Some(re) = self.downcast::<crate::builtins::RegExp>() {
            if let Some(s) =
                crate::console::print::PrettyObjectOverride::pretty_multiline(&*re, self, not)
            {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::Date>() {
            if let Some(s) =
                crate::console::print::PrettyObjectOverride::pretty_multiline(&*date, self, not)
            {
                return s;
            }
        }


        if let Some(date) = self.downcast::<crate::builtins::temporal::Duration>() {
            if let Some(s) =
                crate::console::print::PrettyObjectOverride::pretty_multiline(&*date, self, not)
            {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::Instant>() {
            if let Some(s) =
                crate::console::print::PrettyObjectOverride::pretty_multiline(&*date, self, not)
            {
                return s;
            }
        }

        if let Some(date) = self.downcast::<crate::builtins::temporal::PlainDate>() {
            if let Some(s) =
                crate::console::print::PrettyObjectOverride::pretty_multiline(&*date, self, not)
            {
                return s;
            }
        }


        let id = self.id();

        if not.contains(&id) {
            return "[Circular *1]".bright_green().to_string();
        }

        not.push(id);

        let mut str = if self.is_function() {
            format!("[Function: {}] ", self.name())
                .bright_green()
                .to_string()
        } else if let Some(prim) = self.primitive() {
            match prim {
                Value::Null => "[Null]".bright_green().to_string(),
                Value::Undefined => "[Undefined]".bright_green().to_string(),
                Value::String(s) => format!("[String: \"{s}\"]").bright_green().to_string(),
                Value::Number(n) => format!("[Number: {n}]").bright_green().to_string(),
                Value::Boolean(b) => format!("[Boolean: {b}]").bright_green().to_string(),
                _ => "[Primitive]".bright_green().to_string(),
            }
        } else {
            String::new()
        };

        str.push_str("{\n");

        if let Ok(properties) = self.properties() {
            if properties.is_empty() {
                not.pop();
                if str.len() == 2 {
                    return "{}".to_string();
                }
                str.pop();
                str.pop();
                return str;
            }

            for (key, value) in properties {
                str.push_str("  ");
                str.push_str(&key.pretty_print_key());
                str.push_str(": ");
                str.push_str(&value.pretty_print_circular_nl(not));
                str.push_str(",\n");
            }
            str.pop();
            str.pop();
        } else {
            str.push_str("{}");
        };

        str.push_str("\n}");

        not.pop();

        str
    }
}

impl PrettyPrint for Value<Realm> {
    fn pretty_print_key(&self) -> String {
        match self {
            Self::Undefined => "undefined".to_string(),
            Self::Null => "null".to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Number(n) => n.to_string(),
            Self::String(s) => s.to_string(),
            Self::Object(o) => o.pretty_print_key(),
            Self::Symbol(s) => s.to_string(),
            Self::BigInt(b) => b.to_string(),
        }
    }

    fn pretty_print_circular(&self, not: &mut Vec<usize>) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => format!("\"{s}\"").green().to_string(),
            Self::Object(o) => o.pretty_print_circular(not),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
            Self::BigInt(b) => {
                let mut str = b.to_string();
                str.push('n');

                str.bright_yellow().to_string()
            }
        }
    }

    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => format!("\"{s}\"").green().to_string(),
            Self::Object(o) => o.pretty_print_circular_nl(not),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
            Self::BigInt(b) => {
                let mut str = b.to_string();
                str.push('n');

                str.bright_yellow().to_string()
            }
        }
    }
}

pub fn fmt_properties_to(obj: &Object<Realm>, str: &mut String, not: &mut Vec<usize>) {
    let Ok(properties) = obj.properties() else {
        return;
    };

    if properties.is_empty() {
        return;
    }

    str.push_str(" { ");

    for (key, value) in properties {
        str.push_str(&key.pretty_print_key());
        str.push_str(": ");
        str.push_str(&value.pretty_print_circular(not));
        str.push_str(", ");
    }
    str.pop();
    str.pop();

    str.push_str(" }");
}
