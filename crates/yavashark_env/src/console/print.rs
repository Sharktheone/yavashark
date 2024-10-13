use colored::Colorize;

use yavashark_value::{Object, Value};

use crate::context::Context;

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

impl PrettyPrint for Object<Context> {
    fn pretty_print_key(&self) -> String {
        format!("'{self}'").green().to_string()
    }

    fn pretty_print_circular(&self, not: &mut Vec<usize>) -> String { let id = self.id();

        if not.contains(&id) {
            return "[Circular *1]".green().to_string()
        }

        not.push(id);


        let mut str = String::new();
        str.push_str("{ ");

        if let Ok(properties) = self.properties() {
            if properties.is_empty() {
                not.pop();
                return "{}".to_string();
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
        let id = self.id();
        
        if not.contains(&id) {
            return "[Circular *1]".green().to_string()
        }
        
        not.push(id);
        
        
        
        
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
                not.pop();
                return "{}".to_string();
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

impl PrettyPrint for Value<Context> {
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

    fn pretty_print_circular(&self, not: &mut Vec<usize>) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => s.to_string().green().to_string(),
            Self::Object(o) => o.pretty_print_circular(not),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
        }
    }

    fn pretty_print_circular_nl(&self, not: &mut Vec<usize>) -> String {
        match self {
            Self::Undefined => "undefined".bright_black().to_string(),
            Self::Null => "null".white().bold().to_string(),
            Self::Boolean(b) => b.to_string().yellow().to_string(),
            Self::Number(n) => n.to_string().bright_yellow().to_string(),
            Self::String(s) => s.to_string().green().to_string(),
            Self::Object(o) => o.pretty_print_circular_nl(not),
            Self::Symbol(s) => s.to_string().cyan().to_string(),
        }
    }
}
