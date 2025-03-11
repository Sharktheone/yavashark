use crate::metadata::{Flags, Metadata, NegativePhase};
use std::path::Path;
use swc_common::comments::{CommentKind, SingleThreadedComments, SingleThreadedCommentsMap};
use swc_common::input::StringInput;
use swc_common::util::take::Take;
use swc_common::BytePos;
use swc_ecma_ast::{Program, Script};
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yaml_rust2::yaml::YamlDecoder;
use yaml_rust2::Yaml;

pub(crate) fn parse_file(f: &Path) -> (Program, Metadata) {
    let input = std::fs::read_to_string(f).unwrap();

    if input.is_empty() {
        return (Program::Script(Script::dummy()), Metadata::default());
    }

    let c = EsSyntax::default();

    let metadata;

    {
        let end = input
            .find("\n---*/\n")
            .map(|x| x + 7)
            .unwrap_or(input.len());

        let input = &input[..end];
        let max = BytePos(input.len() as u32);

        let input = StringInput::new(&input[..end], BytePos(0), max);

        let comments = SingleThreadedComments::default();

        let mut p = Parser::new(Syntax::Es(c), input, Some(&comments));

        _ = p.parse_script();

        let (leading, trailing) = comments.take_all();

        let mut meta = process_comments(leading);
        let mut trailing = process_comments(trailing);

        meta.append(&mut trailing);

        metadata = meta.first().map(Metadata::parse).unwrap_or_default();
    };

    let end = BytePos(input.len() as u32 - 1);

    let input = StringInput::new(&input, BytePos(0), end);

    let mut p = Parser::new(Syntax::Es(c), input, None);

    let s = match p.parse_program() {
        Ok(s) => {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    println!("PARSE_ERROR: Expected error but parsed successfully");
                    panic!()
                }
            }

            if s.is_module() && !metadata.flags.contains(Flags::MODULE) {
                println!("PARSE_ERROR: Expected script but parsed module");
                panic!()
            }

            s
        }
        Err(e) => {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    return (Program::Script(Script::dummy()), Metadata::default());
                }
            }

            println!("PARSE_ERROR:\n{e:?}");
            panic!()
        }
    };

    (s, metadata)
}

fn process_comments(map: SingleThreadedCommentsMap) -> Vec<Yaml> {
    map.borrow()
        .iter()
        .flat_map(|(_, x)| x)
        .filter(|comment| {
            if comment.kind != CommentKind::Block {
                return false;
            }

            comment.text.starts_with("---\n")
        })
        .filter_map(|c| YamlDecoder::read(c.text.as_bytes()).decode().ok())
        .flatten()
        .collect()
}
