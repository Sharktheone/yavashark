use crate::metadata::{Flags, Metadata, NegativePhase};
use std::path::Path;
use swc_common::BytePos;
use swc_common::comments::{CommentKind, SingleThreadedComments, SingleThreadedCommentsMap};
use swc_common::input::StringInput;
use swc_common::util::take::Take;
use swc_ecma_ast::{Program, Script};
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yaml_rust2::Yaml;
use yaml_rust2::yaml::YamlDecoder;
use yavashark_swc_validator::Validator;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Variant {
    Sloppy,
    Strict,
    Module,
    Raw,
}

impl Variant {
    fn for_metadata(metadata: &Metadata) -> &'static [Self] {
        if metadata.flags.contains(Flags::MODULE) {
            &[Self::Module]
        } else if metadata.flags.contains(Flags::RAW) {
            &[Self::Raw]
        } else if metadata.flags.contains(Flags::ONLY_STRICT) {
            &[Self::Strict]
        } else if metadata.flags.contains(Flags::NO_STRICT) {
            &[Self::Sloppy]
        } else {
            &[Self::Sloppy, Self::Strict]
        }
    }

    fn parse(self, source: &str) -> Result<Program, String> {
        // Strictness must reach the parser as well as the validator. Raw tests
        // retain their exact source; modules are inherently strict.
        let strict_source;
        let source = if self == Self::Strict {
            strict_source = format!("\"use strict\";\n{source}");
            strict_source.as_str()
        } else {
            source
        };

        let syntax = EsSyntax {
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

        let input = StringInput::new(source, BytePos(0), BytePos(source.len() as u32));
        let mut parser = Parser::new(Syntax::Es(syntax), input, None);
        let program = if self == Self::Module {
            parser.parse_module().map(Program::Module)
        } else {
            parser.parse_script().map(Program::Script)
        }
        .map_err(|error| format!("{error:?}"))?;

        if let Some(error) = parser.take_errors().first() {
            return Err(format!("{error:?}"));
        }

        Validator::new()
            .validate(&program)
            .map_err(|error| error.to_string())?;

        Ok(program)
    }
}

#[derive(Debug)]
pub(crate) enum ParseFailure {
    Rejected { variant: Variant, message: String },
    Accepted { variant: Variant },
}

impl ParseFailure {
    pub(crate) fn report(self) -> ! {
        match self {
            Self::Rejected { variant, message } => {
                println!("PARSE_ERROR: {variant:?}\n{message}");
            }
            Self::Accepted { variant } => {
                println!(
                    "PARSE_SUCCESS_ERROR: {variant:?}: Expected error but parsed successfully"
                );
            }
        }

        std::process::exit(1);
    }
}

pub(crate) struct ParsedTest {
    pub(crate) metadata: Metadata,
    pub(crate) programs: [Option<(Program, Variant)>; 2],
}

impl ParsedTest {
    pub(crate) fn from_file(path: &Path) -> Self {
        let input = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("Cannot read {}: {error}", path.display()));

        Self::parse(&input).unwrap_or_else(|error| error.report())
    }

    fn parse(input: &str) -> Result<Self, ParseFailure> {
        let metadata = parse_metadata(input);
        let mut programs = [None, None];

        for (slot, &variant) in programs.iter_mut().zip(Variant::for_metadata(&metadata)) {
            *slot =
                Self::parse_variant(input, &metadata, variant)?.map(|program| (program, variant));
        }

        Ok(Self { metadata, programs })
    }

    fn parse_variant(
        input: &str,
        metadata: &Metadata,
        variant: Variant,
    ) -> Result<Option<Program>, ParseFailure> {
        let negative = metadata
            .negative
            .as_ref()
            .is_some_and(|negative| negative.phase == NegativePhase::Parse);

        match (variant.parse(input), negative) {
            (Ok(_), true) => Err(ParseFailure::Accepted { variant }),
            (Err(_), true) => Ok(None),
            (Ok(program), false) => Ok(Some(program)),
            (Err(message), false) => Err(ParseFailure::Rejected { variant, message }),
        }
    }
}

// Harness includes are parsed once in their own context, not expanded into
// test variants. The test entry points above handle the variant matrix.
pub(crate) fn parse_file(path: &Path) -> (Program, Metadata) {
    let input = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("Cannot read {}: {error}", path.display()));

    parse_code(&input)
}

pub(crate) fn parse_code(input: &str) -> (Program, Metadata) {
    let metadata = parse_metadata(input);
    let variant = Variant::for_metadata(&metadata)[0];
    let program = ParsedTest::parse_variant(input, &metadata, variant)
        .unwrap_or_else(|error| error.report())
        .unwrap_or_else(|| Program::Script(Script::dummy()));

    (program, metadata)
}

pub fn parse_metadata(input: &str) -> Metadata {
    let Some(start) = input.find("/*---").map(|offset| offset + 5) else {
        return parse_metadata_comments(input);
    };
    let Some(end) = input[start..].find("---*/").map(|offset| start + offset) else {
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

    let c = EsSyntax {
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
    map.borrow().values().flatten()
        .filter(|comment| {
            if comment.kind != CommentKind::Block {
                return false;
            }

            comment.text.starts_with("---\n") || comment.text.starts_with("---\r\n")
        })
        .filter_map(|c| YamlDecoder::read(c.text.as_bytes()).decode().ok())
        .flatten()
        .collect()
}
