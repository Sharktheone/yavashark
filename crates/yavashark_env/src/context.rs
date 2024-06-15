use yavashark_value::Ctx;

mod prototypes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub proto: prototypes::Prototypes,
}

impl Context {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            proto: prototypes::Prototypes::new()?,
        })
    }
}

impl Ctx for Context {}
