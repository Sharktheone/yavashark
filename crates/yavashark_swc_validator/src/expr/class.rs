use swc_ecma_ast::{Class, ClassExpr, ClassMember, ParamOrTsParamProp};
use crate::Validator;

impl Validator {
    pub fn validate_class_expr(call: &ClassExpr) -> Result<(), String> {
        Self::validate_class(&call.class)
    }

    pub fn validate_class(class: &Class) -> Result<(), String> {
        for member in &class.body {
            Self::validate_class_member(member)?;
        }

        if let Some(super_class) = &class.super_class {
            Self::validate_expr(super_class)?;
        }

        Ok(())
    }

    fn validate_class_member(class: &ClassMember) -> Result<(), String> {
        match class {
            ClassMember::Constructor(constructor) => {
                Self::validate_prop_name(&constructor.key)?;

                for param in &constructor.params {
                    if let ParamOrTsParamProp::Param(param) = param {
                        Self::validate_pat(&param.pat)?;
                    }
                }

                if let Some(body) = &constructor.body {
                    Self::validate_block(body)?;
                }
            },
            ClassMember::Method(method) => {
                Self::validate_prop_name(&method.key)?;

                Self::validate_function(&method.function)?;
            },
            ClassMember::PrivateMethod(private_method) => {
                Self::validate_private_name_expr(&private_method.key)?;
                Self::validate_function(&private_method.function)?;
            },
            ClassMember::ClassProp(prop) => {
                Self::validate_prop_name(&prop.key)?;
                if let Some(value) = &prop.value {
                    Self::validate_expr(value)?;
                }
            },
            ClassMember::PrivateProp(private_prop) => {
                Self::validate_private_name_expr(&private_prop.key)?;
                if let Some(value) = &private_prop.value {
                    Self::validate_expr(value)?;
                }
            },
            ClassMember::StaticBlock(static_block) => {
                Self::validate_block(&static_block.body)?;
            },
            _ => {}
        }


        Ok(())

    }
}
