use crate::metadata::{Metadata, NegativePhase};
use std::path::Path;
use swc_common::comments::{CommentKind, SingleThreadedComments};
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::Stmt;
use swc_ecma_parser::{EsSyntax, Parser, Syntax};
use yaml_rust2::yaml::YamlDecoder;

pub(crate) fn parse_file(f: &Path) -> Vec<Stmt> {
    let input = std::fs::read_to_string(f).unwrap();

    if input.is_empty() {
        return Vec::new();
    }
    
    let c = EsSyntax::default();

    let metadata;

    {
        let end = input.find("\n---*/\n").map(|x| x + 7).unwrap_or(input.len());
        

        let input = &input[..end];
        let max = BytePos(input.len() as u32);

        let input = StringInput::new(&input[..end], BytePos(0), max);


        let comments = SingleThreadedComments::default();

        let mut p = Parser::new(Syntax::Es(c), input, Some(&comments));

        let x = p.parse_script();

        let (leading, a) = comments.take_all();

        let meta = leading
            .borrow()
            .iter()
            .map(|(_, x)| x)
            .flatten()
            .filter(|comment| {
                if comment.kind != CommentKind::Block {
                    return false;
                }

                comment.text.starts_with("---\n")
            })
            .filter_map(|c| YamlDecoder::read(c.text.as_bytes()).decode().ok())
            .flatten()
            .collect::<Vec<_>>();


        metadata = meta.first().map(Metadata::parse).unwrap_or(Metadata::default());
    };
    
    
    let end = BytePos(input.len() as u32 - 1);
    
    let input = StringInput::new(&input, BytePos(0), end);
    
    let mut p = Parser::new(Syntax::Es(c), input, None);


    let s = match p.parse_script() {
        Ok(s) => s,
        Err(e) => {
            if let Some(neg) = &metadata.negative {
                if neg.phase == NegativePhase::Parse {
                    return Vec::new();
                }
            }
            
            println!("PARSE_ERROR:\n{e:?}");
            panic!()
        }
    };
    s.body
}
