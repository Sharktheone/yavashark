use crate::Res;
use crate::Value;
use yavashark_value::error::Error;

pub struct Variable {
    pub value: Value,
    pub properties: Attributes,
}

impl Variable {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            properties: Attributes::new(),
        }
    }

    pub fn new_read_only(value: Value) -> Self {
        Self {
            value,
            properties: Attributes::new_read_only(),
        }
    }

    pub fn mutate(&mut self, value: Value) -> Res {
        if !self.properties.is_writable() {
            return Err(Error::new(
                "Cannot assign to read-only variable".to_string(),
            ));
        }

        self.value = value;
        Ok(())
    }

    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn copy(&self) -> Value {
        self.value.copy()
    }

    pub fn is_writable(&self) -> bool {
        self.properties.is_writable()
    }

    pub fn is_enumerable(&self) -> bool {
        self.properties.is_enumerable()
    }

    pub fn is_configurable(&self) -> bool {
        self.properties.is_configurable()
    }

    pub fn make_writable(&mut self) {
        self.properties.make_writable();
    }

    pub fn make_enumerable(&mut self) {
        self.properties.make_enumerable();
    }

    pub fn make_configurable(&mut self) {
        self.properties.make_configurable();
    }
}

pub struct Attributes(u8);

impl Attributes {
    const WRITABLE: u8 = 0b1;
    const ENUMERABLE: u8 = 0b10;
    const CONFIGURABLE: u8 = 0b100;

    pub fn new() -> Self {
        Self(Self::WRITABLE)
    }

    pub fn new_read_only() -> Self {
        Self(0)
    }

    pub fn is_writable(&self) -> bool {
        self.0 & Self::WRITABLE != 0
    }

    pub fn is_enumerable(&self) -> bool {
        self.0 & Self::ENUMERABLE != 0
    }

    pub fn is_configurable(&self) -> bool {
        self.0 & Self::CONFIGURABLE != 0
    }

    pub fn make_writable(&mut self) {
        self.0 |= Self::WRITABLE;
    }

    pub fn make_enumerable(&mut self) {
        self.0 |= Self::ENUMERABLE;
    }

    pub fn make_configurable(&mut self) {
        self.0 |= Self::CONFIGURABLE;
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}
