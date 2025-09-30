use swc_common::Span;
use yavashark_env::error::Location;
use yavashark_env::scope::Scope;
use yavashark_env::Res;

pub fn get_location(span: Span, scope: &Scope) -> Location {
    let range = span.lo.0..span.hi.0;

    let Ok(file) = scope.get_current_path() else {
        return Location::SourceRange { range };
    };

    Location::Source { path: file, range }
}
