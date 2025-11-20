use yavashark_macro::data_object;
use crate::builtins::intl::Numeric;

#[data_object]
pub enum LocaleMatcher {
    Lookup,
    #[name("best fit")]
    BestFit,
}


#[data_object]
pub enum HourCycle {
    H11,
    H12,
    H23,
    H24,
}

#[data_object]
pub enum Style {
    Narrow,
    Short,
    Long,
}

#[data_object]
pub struct LocaleMatcherOptions {
    options: LocaleMatcher,
}
