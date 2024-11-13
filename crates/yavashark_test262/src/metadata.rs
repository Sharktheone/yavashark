use std::ops::Index;
use bitflags::bitflags;
use yaml_rust2::Yaml;
use clap::Parser;

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub negative: Option<Negative>,
    pub includes: Vec<String>,
    pub flags: Flags,
    pub locale: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Negative {
    pub phase: NegativePhase,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NegativePhase {
    Parse,
    Resolution,
    Runtime,
}

bitflags! {
    #[derive(Clone, Debug, Default)]
    pub struct Flags: u16 {
        const ONLY_STRICT = 1 << 0;
        const NO_STRICT = 1 << 1;
        const MODULE = 1 << 2;
        const RAW = 1 << 3;
        const ASYNC = 1 << 4;
        const GENERATED = 1 << 5;
        const CAN_BLOCK_IS_FALSE = 1 << 6;
        const CAN_BLOCK_IS_TRUE = 1 << 7;
        const NON_DETERMINISTIC = 1 << 8;
    }
}

impl Metadata {
    pub fn parse(yaml: &Yaml) -> Self {
        let negative = yaml.index("negative");

        let negative = Negative::parse(negative);


        let includes = if let Yaml::Array(includes) = yaml.index("includes") {
            includes.iter().filter_map(|v| {
                if let Yaml::String(v) = v {
                    Some(v.clone())
                } else {
                    None
                }
            }).collect()
        } else {
            Vec::new()
        };


        let flags = if let Yaml::Array(flag) = yaml.index("flags") {
            let mut flags = Flags::empty();

            for f in flag {

                if let Yaml::String(f) = f {
                    match f.as_str() {
                        "onlyStrict" => flags.insert(Flags::ONLY_STRICT),
                        "noStrict" => flags.insert(Flags::NO_STRICT),
                        "module" => flags.insert(Flags::MODULE),
                        "raw" => flags.insert(Flags::RAW),
                        "async" => flags.insert(Flags::ASYNC),
                        "generated" => flags.insert(Flags::GENERATED),
                        "CanBlockIsFalse" => flags.insert(Flags::CAN_BLOCK_IS_FALSE),
                        "CanBlockIsTrue" => flags.insert(Flags::CAN_BLOCK_IS_TRUE),
                        "non-deterministic" => flags.insert(Flags::NON_DETERMINISTIC),

                        _ => {}
                    }
                }
            }


            flags

        } else {
            Flags::empty()
        };



        let locale = if let Yaml::Array(locale) = yaml.index("locale") {
            locale.iter().filter_map(|v| {
                if let Yaml::String(v) = v {
                    Some(v.clone())
                } else {
                    None
                }
            }).collect()
        } else {
            Vec::new()
        };


        Self {
            negative,
            flags,
            includes,
            locale
        }


    }
}


impl Negative {
    fn parse(yaml: &Yaml) -> Option<Self> {
        let Yaml::String(ty) = yaml.index("type") else {
            return None
        };

        let Yaml::String(phase) = yaml.index("phase") else {
            return None
        };

        let phase = match phase.as_str() {
            "parse" => NegativePhase::Parse,
            "resolution" => NegativePhase::Resolution,
            "runtime" => NegativePhase::Runtime,
            _ => return None,
        };


        Some(Self {
            phase,
            ty: ty.clone(),
        })
    }
}
