use std::path::PathBuf;
use yavashark_env::Error;

pub fn test_file(file: PathBuf) -> Result<String, Error> {
    #[cfg(not(feature = "oxc"))]
    test_parse_swc(file);

    #[cfg(feature = "oxc")]
    test_parse_oxc(file);

    Ok(String::new())
}

pub fn test_parse_swc(file: PathBuf) {
    crate::utils::ParsedTest::from_file(&file);
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
    use oxc::diagnostics::{OxcDiagnostic, Severity};
    use oxc::span::SourceType;
    use std::path::PathBuf;

    pub fn test_parse_oxc(file: PathBuf) {
        let input = std::fs::read_to_string(&file).unwrap();

        let metadata = parse_metadata(&input);

        let alloc = Allocator::default();

        let parser = oxc::parser::Parser::new(&alloc, &input, SourceType::default());

        let res = parser.parse();

        let sem = oxc_semantic::SemanticBuilder::new()
            .with_check_syntax_error(false)
            .build(&res.program);

        fn has_errors(dia: &[OxcDiagnostic]) -> bool {
            dia.iter().any(|d| d.severity == Severity::Error)
        }

        fn is_err_free(dia: &[OxcDiagnostic]) -> bool {
            dia.iter().all(|d| d.severity != Severity::Error)
        }

        if !res.panicked && is_err_free(&res.errors) && is_err_free(&sem.errors) {
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

            println!("PARSE_ERROR:\n{:#?}\n{:#?}", res.errors, sem.errors);
            panic!()
        }
    }
}
