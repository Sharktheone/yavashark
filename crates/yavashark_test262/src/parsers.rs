use std::path::PathBuf;
use swc_common::BytePos;
use swc_common::input::StringInput;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use crate::metadata::NegativePhase;
use crate::utils::parse_metadata;

pub fn test_parse_swc(file: PathBuf) -> Result<(), String> {
    let input = std::fs::read_to_string(&file).unwrap();

    let metadata = parse_metadata(&input);

    let end = BytePos(input.len() as u32);

    let input = StringInput::new(&input, BytePos(0), end);

    let c = EsSyntax::default();

    let mut p = Parser::new(Syntax::Es(c), input, None);

    let _s = match p.parse_program() {
        Ok(s) => {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    println!("PARSE_ERROR: Expected error but parsed successfully");
                    panic!()
                }
            }

            s
        }
        Err(e) => {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    return Ok(())
                }
            }

            println!("PARSE_ERROR:\n{e:?}");
            panic!()
        }
    };


    Ok(())

}