use yavashark_macro::data_object;

#[derive(Clone, Copy, Debug)]
#[data_object]
pub enum LocaleMatcher {
    Lookup,
    #[name("best fit")]
    BestFit,
}

#[derive(Clone, Copy, Debug)]
#[data_object]
pub enum HourCycle {
    H11,
    H12,
    H23,
    H24,
}

#[derive(Clone, Copy, Debug)]
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
