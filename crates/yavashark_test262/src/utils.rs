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
use yavashark_swc_validator::Validator;

pub(crate) fn parse_file(f: &Path) -> (Program, Metadata) {
    let input = match std::fs::read_to_string(f) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            panic!("File not found: {f:?}");
        }
        Err(e) => {
            panic!("Error reading file {f:?}: {e}");
        }
    };

    parse_code(&input)
}
pub(crate) fn parse_code(input: &str) -> (Program, Metadata) {
    if input.is_empty() {
        return (Program::Script(Script::dummy()), Metadata::default());
    }

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

    let metadata = parse_metadata(input);

    if metadata.flags.contains(Flags::ONLY_STRICT) && !metadata.negative.as_ref().is_some_and(|neg| neg.phase == NegativePhase::Parse) {
        println!("SKIP");
        return (Program::Script(Script::dummy()), Metadata::default());
    }

    let end = BytePos(input.len() as u32);

    let input = StringInput::new(input, BytePos(0), end);

    let mut p = Parser::new(Syntax::Es(c), input, None);

    let s = match p.parse_program() {
        Ok(s) => {
            match &s {
                Program::Script(script) => {
                    if !p.take_errors().is_empty() {
                        if let Some(neg) = &metadata.negative {
                            if neg.phase == NegativePhase::Parse {
                                return (Program::Script(Script::dummy()), Metadata::default());
                            }
                        }

                        println!("PARSE_ERROR:\n");
                        panic!()
                    }

                    let mut validator = Validator::new();

                    if metadata.flags.contains(Flags::ONLY_STRICT) {
                        validator.enable_script_strict_mode();
                    }

                    if let Err(e) = validator.validate_statements(&script.body) {
                        if let Some(neg) = &metadata.negative {
                            if neg.phase == NegativePhase::Parse {
                                return (Program::Script(Script::dummy()), Metadata::default());
                            }
                        }

                        println!("PARSE_SUCCESS_ERROR:\n{e:?}");
                        panic!()
                    }
                }
                Program::Module(module) => {
                    if !p.take_errors().is_empty() {
                        if let Some(neg) = &metadata.negative {
                            if neg.phase == NegativePhase::Parse {
                                return (Program::Script(Script::dummy()), Metadata::default());
                            }
                        }

                        println!("PARSE_ERROR:\n");
                        panic!()
                    }

                    if let Err(e) = Validator::new().validate_module_items(&module.body) {
                        if let Some(neg) = &metadata.negative {
                            if neg.phase == NegativePhase::Parse {
                                return (Program::Script(Script::dummy()), Metadata::default());
                            }
                        }

                        println!("PARSE_SUCCESS_ERROR:\n{e:?}");
                        panic!()
                    }
                }
            }

            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    println!("PARSE_SUCCESS_ERROR: Expected error but parsed successfully");
                    panic!()
                }
            }

            // if s.is_module() && !metadata.flags.contains(Flags::MODULE) {
            //     println!("PARSE_ERROR: Expected script but parsed module");
            //     panic!()
            // }

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

pub fn parse_metadata(input: &str) -> Metadata {
    let Some(start) = input.find("\n/*---\n").map(|x| x + 7) else {
        return parse_metadata_comments(input);
    };
    let Some(end) = input.find("\n---*/\n") else {
        return parse_metadata_comments(input);
    };

    let input = &input[start..end];

    YamlDecoder::read(input.as_bytes())
        .decode()
        .ok()
        .as_ref()
        .and_then(|x| x.first())
        .map(Metadata::parse)
        .unwrap_or_default()
}

fn parse_metadata_comments(input: &str) -> Metadata {
    let end = input
        .find("\n---*/\n")
        .map(|x| x + 7)
        .unwrap_or(input.len());

    let input = &input[..end];
    let max = BytePos(input.len() as u32);

    let input = StringInput::new(&input[..end], BytePos(0), max);

    let comments = SingleThreadedComments::default();

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

    let mut p = Parser::new(Syntax::Es(c), input, Some(&comments));

    _ = p.parse_script();

    let (leading, trailing) = comments.take_all();

    let mut meta = process_comments(leading);
    let mut trailing = process_comments(trailing);

    meta.append(&mut trailing);

    meta.first().map(Metadata::parse).unwrap_or_default()
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
