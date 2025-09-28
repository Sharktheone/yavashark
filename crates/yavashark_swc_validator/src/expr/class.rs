use std::collections::{HashMap, HashSet};

use crate::{Validator};
use swc_ecma_ast::{
    Callee, Class, ClassExpr, ClassMember, Expr, Ident, IdentName, MethodKind, ParamOrTsParamProp,
    PropName, Super,
};
use swc_ecma_visit::{Visit, VisitWith};

#[derive(Default)]
struct PrivateNameEntry {
    has_field: bool,
    has_method: bool,
    has_getter: bool,
    has_setter: bool,
}

impl PrivateNameEntry {
    const fn has_any(&self) -> bool {
        self.has_field || self.has_method || self.has_getter || self.has_setter
    }

    const fn add_field(&mut self) -> bool {
        if self.has_any() {
            return false;
        }
        self.has_field = true;
        true
    }

    const fn add_method(&mut self) -> bool {
        if self.has_any() {
            return false;
        }
        self.has_method = true;
        true
    }

    const fn add_getter(&mut self) -> bool {
        if self.has_field || self.has_method || self.has_getter {
            return false;
        }
        self.has_getter = true;
        true
    }

    const fn add_setter(&mut self) -> bool {
        if self.has_field || self.has_method || self.has_setter {
            return false;
        }
        self.has_setter = true;
        true
    }
}

impl<'a> Validator<'a> {
    pub fn validate_class_expr(&mut self, call: &'a ClassExpr) -> Result<(), String> {
        self.validate_class(&call.class)
    }

    pub fn validate_class(&mut self, class: &'a Class) -> Result<(), String> {
        let mut private_registry: HashMap<&'a str, PrivateNameEntry> = HashMap::new();
        let mut has_constructor = false;

        for member in &class.body {
            match member {
                ClassMember::Constructor(_) => {
                    if has_constructor {
                        return Err("Class cannot have multiple constructors".to_string());
                    }
                    has_constructor = true;
                }
                ClassMember::PrivateMethod(private_method) => {
                    let name = private_method.key.name.as_str();
                    let entry = private_registry.entry(name).or_default();
                    let added = match private_method.kind {
                        MethodKind::Method => entry.add_method(),
                        MethodKind::Getter => entry.add_getter(),
                        MethodKind::Setter => entry.add_setter(),
                    };

                    if !added {
                        return Err(format!("Duplicate private name: #{name}"));
                    }
                }
                ClassMember::PrivateProp(private_prop) => {
                    let name = private_prop.key.name.as_str();
                    let entry = private_registry.entry(name).or_default();

                    if !entry.add_field() {
                        return Err(format!("Duplicate private name: #{name}"));
                    }
                }
                _ => {}
            }
        }

        if let Some(super_class) = &class.super_class {
            self.validate_expr(super_class)?;
        }

        let private_names: HashSet<&str> = private_registry.keys().copied().collect();
        let scope = self.enter_private_name_scope(private_names);

        for member in &class.body {
            if let Err(e) = self.validate_class_member(member) {
                scope.exit(self);
                return Err(e);
            }
        }

        scope.exit(self);

        Ok(())
    }

    fn validate_class_member(&mut self, class: &'a ClassMember) -> Result<(), String> {
        match class {
            ClassMember::Constructor(constructor) => {
                self.validate_prop_name(&constructor.key)?;

                for param in &constructor.params {
                    if let ParamOrTsParamProp::Param(param) = param {
                        self.validate_pat(&param.pat)?;
                    }
                }

                if let Some(body) = &constructor.body {
                    self.validate_block(body)?;
                }
            }
            ClassMember::Method(method) => {
                self.validate_prop_name(&method.key)?;

                self.validate_function(&method.function)?;
            }
            ClassMember::PrivateMethod(private_method) => {
                self.validate_private_name_expr(&private_method.key)?;
                self.validate_function(&private_method.function)?;
            }
            ClassMember::ClassProp(prop) => {
                self.validate_prop_name(&prop.key)?;
                if prop.is_static {
                    if let Some(name) = prop_name_to_string(&prop.key) {
                        if matches!(name.as_str(), "prototype" | "constructor") {
                            return Err(format!(
                                "Static field cannot be named '{name}'"
                            ));
                        }
                    }
                }

                if let Some(value) = &prop.value {
                    validate_field_initializer(value)?;
                    self.validate_expr(value)?;
                }
            }
            ClassMember::PrivateProp(private_prop) => {
                self.validate_private_name_expr(&private_prop.key)?;
                if let Some(value) = &private_prop.value {
                    validate_field_initializer(value)?;
                    self.validate_expr(value)?;
                }
            }
            ClassMember::StaticBlock(static_block) => {
                self.validate_block(&static_block.body)?;
            }
            _ => {}
        }

        Ok(())
    }
}

fn validate_field_initializer(expr: &Expr) -> Result<(), String> {
    if contains_arguments(expr) {
        return Err("Field initializer cannot contain 'arguments'".to_string());
    }

    if contains_super(expr) {
        return Err("Field initializer cannot contain 'super'".to_string());
    }

    Ok(())
}

fn contains_arguments(expr: &Expr) -> bool {
    contains_identifier(expr, "arguments")
}

fn contains_identifier(expr: &Expr, target: &str) -> bool {
    let mut finder = ContainsIdentifier {
        target,
        found: false,
    };
    expr.visit_with(&mut finder);
    finder.found
}

fn contains_super(expr: &Expr) -> bool {
    let mut finder = ContainsSuper { found: false };
    expr.visit_with(&mut finder);
    finder.found
}

fn prop_name_to_string(prop_name: &PropName) -> Option<String> {
    match prop_name {
        PropName::Ident(ident) => Some(ident.sym.to_string()),
        PropName::Str(str_lit) => Some(str_lit.value.to_string()),
        _ => None,
    }
}

struct ContainsIdentifier<'a> {
    target: &'a str,
    found: bool,
}

impl Visit for ContainsIdentifier<'_> {
    fn visit_ident(&mut self, ident: &Ident) {
        if self.found {
            return;
        }

        if ident.sym.as_ref() == self.target {
            self.found = true;
        }
    }

    fn visit_ident_name(&mut self, ident: &IdentName) {
        if self.found {
            return;
        }

        if ident.sym.as_ref() == self.target {
            self.found = true;
        }
    }
}

struct ContainsSuper {
    found: bool,
}

impl Visit for ContainsSuper {
    fn visit_call_expr(&mut self, call: &swc_ecma_ast::CallExpr) {
        if self.found {
            return;
        }

        if matches!(call.callee, Callee::Super(_)) {
            self.found = true;
        }

        call.visit_children_with(self);
    }

    fn visit_super(&mut self, _: &Super) {
        if self.found {
            return;
        }

        self.found = true;
    }

    fn visit_super_prop_expr(&mut self, expr: &swc_ecma_ast::SuperPropExpr) {
        if self.found {
            return;
        }

        self.found = true;
        expr.visit_children_with(self);
    }

}
