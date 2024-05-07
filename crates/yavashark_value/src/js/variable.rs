use crate::{Ctx, Error};
use crate::Value;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq)]
pub struct Variable<C: Ctx> {
    pub value: Value<C>,
    pub properties: Attributes,
}

impl<C: Ctx> Variable<C> {
    pub fn new(value: Value<C>) -> Self {
        Self {
            value,
            properties: Attributes::new(),
        }
    }

    pub fn new_read_only(value: Value<C>) -> Self {
        Self {
            value,
            properties: Attributes::new_read_only(),
        }
    }
    
    pub fn new_with_attributes(value: Value<C>, writable: bool, enumerable: bool, configurable: bool) -> Self {
        Self {
            value,
            properties: Attributes::from_values(writable, enumerable, configurable)
        }
    }

    pub fn mutate(&mut self, value: Value<C>) -> Result<(), Error<C>> {
        if !self.properties.is_writable() {
            return Err(Error::new("Cannot assign to read-only variable"));
        }

        self.value = value;
        Ok(())
    }

    pub fn get_value(&self) -> &Value<C> {
        &self.value
    }

    pub fn copy(&self) -> Value<C> {
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

#[derive(PartialEq, Eq)]
pub struct Attributes(u8);

impl Debug for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.is_writable() {
            s.push_str("writable, ");
        }
        if self.is_enumerable() {
            s.push_str("enumerable, ");
        }
        if self.is_configurable() {
            s.push_str("configurable, ");
        }

        s.pop();
        s.pop();

        write!(f, "[{}]", s)
    }
}

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
    
    pub fn from_values(writable: bool, enumerable: bool, configurable: bool) -> Self {
        let mut attributes = Self::new_read_only();
        if writable {
            attributes.make_writable();
        }
        if enumerable {
            attributes.make_enumerable();
        }
        if configurable {
            attributes.make_configurable();
        }

        attributes
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}


impl<C: Ctx, V: Into<Value<C>>> From<V> for Variable<C> {
    fn from(value: V) -> Self {
        Self::new(value.into())
    }
}