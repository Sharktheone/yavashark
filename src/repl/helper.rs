use crate::conf::Conf;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Context, Helper};
use rustyline_derive::{Completer, Hinter, Validator};
use std::borrow::Cow;
use yavashark_env::scope::Scope;

pub struct ScopeCompleter {
    filename: FilenameCompleter,
    scope: Scope,
}

impl Completer for ScopeCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        if let Some(line) = line.strip_prefix('!') {
            let (pos, pairs) = self.filename.complete(line, pos - 1, ctx)?;

            let pairs = pairs
                .into_iter()
                .map(|pair| Pair {
                    display: pair.display,
                    replacement: format!("!{}", pair.replacement),
                })
                .collect();

            return Ok((pos, pairs));
        }

        let Ok(names) = self.scope.get_variable_names() else {
            return Ok((0, vec![]));
        };

        let mut completions = vec![];
        for name in names {
            if name.starts_with(line) {
                completions.push(Pair {
                    display: name.to_string(),
                    replacement: name.to_string(),
                });
            }
        }

        Ok((0, completions))
    }
}

#[derive(Helper, Completer, Hinter, Validator)]
pub struct ReplHelper {
    #[rustyline(Completer)]
    pub completer: ScopeCompleter,
    pub highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    pub validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    pub hinter: HistoryHinter,
    pub colored_prompt: String,
}

impl ReplHelper {
    pub fn new(int: Scope, vm: Scope, conf: Conf) -> Self {
        let completer = if conf.interpreter {
            ScopeCompleter {
                filename: FilenameCompleter::new(),
                scope: vm,
            }
        } else {
            ScopeCompleter {
                filename: FilenameCompleter::new(),
                scope: int,
            }
        };

        Self {
            completer,
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            colored_prompt: String::new(),
        }
    }
}

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Cow::Borrowed(&self.colored_prompt)
        } else {
            Cow::Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}
