use colored::Colorize;

use yavashark_value::{Object, Value};

use crate::context::Context;

pub trait PrettyPrint {
    fn pretty_print(&self) -> String;

    fn pretty_print_nl(&self) -> String;

    fn pretty_print_key(&self) -> String;
}

impl PrettyPrint for Object<Context> {
    fn pretty_print(&self) -> String {
        let mut str = String::new();
        str.push_str("{ ");

        if let Ok(properties) = self.properties() {
            if properties.is_empty() {
                return "{}".to_string();
            }

            for (key, value) in properties {
                str.push_str(&key.pretty_print_key());
                str.push_str(": ");
                str.push_str(&value.pretty_print());
                str.push_str(", ");
            }
            str.pop();
            str.pop();
        } else {
            str.push_str("{}");
        };

        str.push_str(" }");
        str
    }

    fn pretty_print_nl(&self) -> String {
        let mut str = if self.is_function() {
            format!("[Function: {}] ", self.name())
                .bright_green()
                .to_string()
        } else {
            String::new()
        };

        str.push_str("{\n");

        if let Ok(properties) = self.properties() {
            if properties.is_empty() {
                return "{}".to_string();
            }

            for (key, value) in properties {
                str.push_str("  ");
                str.push_str(&key.pretty_print_key());
                str.push_str(": ");
                str.push_str(&value.pretty_print_nl());
                str.push_str(",\n");
            }
            str.pop();
            str.pop();
        } else {
            str.push_str("{}");
        };

        str.push_str("\n}");
        str
    }

    fn pretty_print_key(&self) -> String {
        format!("'{self}'").green().to_string()
    }
}

impl PrettyPrint for Value<Context> {
    fn pretty_print(&self) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => s.to_string().green().to_string(),
            Self::Object(o) => o.pretty_print(),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
        }
    }

    fn pretty_print_nl(&self) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => s.to_string().green().to_string(),
            Self::Object(o) => o.pretty_print_nl(),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
        }
    }

    fn pretty_print_key(&self) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => s.to_string().green().to_string(),
            Self::Object(o) => o.pretty_print_key(),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
        }
    }
}
