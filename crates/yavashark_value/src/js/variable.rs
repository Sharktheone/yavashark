use std::fmt::Debug;

use crate::Value;
use crate::{Error, Realm};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable<C: Realm> {
    pub value: Value<C>,
    pub properties: Attributes,
}

impl<C: Realm> Variable<C> {
    #[must_use]
    pub const fn new(value: Value<C>) -> Self {
        Self {
            value,
            properties: Attributes::new(),
        }
    }

    #[must_use]
    pub const fn new_read_only(value: Value<C>) -> Self {
        Self {
            value,
            properties: Attributes::new_read_only(),
        }
    }

    #[must_use]
    pub fn new_with_attributes(
        value: Value<C>,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    ) -> Self {
        Self {
            value,
            properties: Attributes::from_values(writable, enumerable, configurable),
        }
    }

    pub fn mutate(&mut self, value: Value<C>) -> Result<(), Error<C>> {
        if !self.properties.is_writable() {
            return Err(Error::new("Cannot assign to read-only variable"));
        }

        self.value = value;
        Ok(())
    }

    #[must_use]
    pub const fn get_value(&self) -> &Value<C> {
        &self.value
    }

    #[must_use]
    pub fn copy(&self) -> Value<C> {
        self.value.copy()
    }

    #[must_use]
    pub const fn is_writable(&self) -> bool {
        self.properties.is_writable()
    }

    #[must_use]
    pub const fn is_enumerable(&self) -> bool {
        self.properties.is_enumerable()
    }

    #[must_use]
    pub const fn is_configurable(&self) -> bool {
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

#[derive(PartialEq, Eq, Clone, Copy)]
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

        write!(f, "[{s}]")
    }
}

impl Attributes {
    const WRITABLE: u8 = 0b1;
    const ENUMERABLE: u8 = 0b10;
    const CONFIGURABLE: u8 = 0b100;

    #[must_use]
    pub const fn new() -> Self {
        Self(Self::WRITABLE)
    }

    #[must_use]
    pub const fn new_read_only() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn is_writable(&self) -> bool {
        self.0 & Self::WRITABLE != 0
    }

    #[must_use]
    pub const fn is_enumerable(&self) -> bool {
        self.0 & Self::ENUMERABLE != 0
    }

    #[must_use]
    pub const fn is_configurable(&self) -> bool {
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

    #[must_use]
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

impl<C: Realm, V: Into<Value<C>>> From<V> for Variable<C> {
    fn from(value: V) -> Self {
        Self::new(value.into())
    }
}
