use crate::metadata::NegativePhase;
use crate::utils::parse_metadata;
use std::path::PathBuf;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yavashark_env::Error;

pub fn test_file(file: PathBuf) -> Result<String, Error> {
    #[cfg(not(feature = "oxc"))]
    test_parse_swc(file.clone());

    #[cfg(feature = "oxc")]
    test_parse_oxc(file);

    Ok(String::new())
}

pub fn test_parse_swc(file: PathBuf) {
    let input = std::fs::read_to_string(&file).unwrap();

    let metadata = parse_metadata(&input);

    let end = BytePos(input.len() as u32);

    let input = StringInput::new(&input, BytePos(0), end);

    let c =  EsSyntax {
            jsx: false,
            fn_bind: false,
            decorators: true,
            decorators_before_export: true,
            export_default_from: true,
            import_attributes: true,
            allow_super_outside_method: false,
            allow_return_outside_function: false,
            auto_accessors: true,
            explicit_resource_management: true,
        };

    let mut p = Parser::new(Syntax::Es(c), input, None);

    let _s = match p.parse_program() {
        Ok(s) => {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    println!("PARSE_SUCCESS_ERROR: Expected error but parsed successfully");
                    panic!()
                }
            }

            s
        }
        Err(e) => {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    return;
                }
            }

            println!("PARSE_ERROR:\n{e:?}");
            panic!()
        }
    };
}

#[cfg(feature = "oxc")]
pub fn test_parse_oxc(file: PathBuf) {
    oxc_parser::test_parse_oxc(file)
}

#[cfg(feature = "oxc")]
mod oxc_parser {
    use crate::metadata::NegativePhase;
    use crate::utils::parse_metadata;
    use oxc::allocator::Allocator;
    use oxc::span::SourceType;
    use std::path::PathBuf;

    pub fn test_parse_oxc(file: PathBuf) {
        let input = std::fs::read_to_string(&file).unwrap();

        let metadata = parse_metadata(&input);

        let alloc = Allocator::default();

        let parser = oxc::parser::Parser::new(&alloc, &input, SourceType::default());

        let res = parser.parse();

        if !res.panicked && res.errors.is_empty() {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    println!("PARSE_SUCCESS_ERROR: Expected error but parsed successfully");
                    panic!()
                }
            }
        } else {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    return;
                }
            }

            println!("PARSE_ERROR:\n{:?}", res.errors);
            panic!()
        }
    }
}
