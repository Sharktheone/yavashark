//! Temporal option enums with automatic parsing from JavaScript values.
//!
//! These wrapper enums use the `#[data_object]` macro to provide automatic
//! conversion from JavaScript values (strings) to the corresponding Rust types.

use yavashark_macro::data_object;

// ============================================================================
// Overflow
// ============================================================================

/// How to handle overflow when constraining values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum Overflow {
    #[default]
    Constrain,
    Reject,
}

impl From<Overflow> for temporal_rs::options::Overflow {
    fn from(value: Overflow) -> Self {
        match value {
            Overflow::Constrain => Self::Constrain,
            Overflow::Reject => Self::Reject,
        }
    }
}

// ============================================================================
// DisplayCalendar
// ============================================================================

/// How to display the calendar annotation in toString output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum DisplayCalendar {
    #[default]
    Auto,
    Always,
    Never,
    Critical,
}

impl From<DisplayCalendar> for temporal_rs::options::DisplayCalendar {
    fn from(value: DisplayCalendar) -> Self {
        match value {
            DisplayCalendar::Auto => Self::Auto,
            DisplayCalendar::Always => Self::Always,
            DisplayCalendar::Never => Self::Never,
            DisplayCalendar::Critical => Self::Critical,
        }
    }
}

// ============================================================================
// DisplayOffset
// ============================================================================

/// How to display the offset in toString output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum DisplayOffset {
    #[default]
    Auto,
    Never,
}

impl From<DisplayOffset> for temporal_rs::options::DisplayOffset {
    fn from(value: DisplayOffset) -> Self {
        match value {
            DisplayOffset::Auto => Self::Auto,
            DisplayOffset::Never => Self::Never,
        }
    }
}

// ============================================================================
// DisplayTimeZone
// ============================================================================

/// How to display the time zone in toString output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum DisplayTimeZone {
    #[default]
    Auto,
    Never,
    Critical,
}

impl From<DisplayTimeZone> for temporal_rs::options::DisplayTimeZone {
    fn from(value: DisplayTimeZone) -> Self {
        match value {
            DisplayTimeZone::Auto => Self::Auto,
            DisplayTimeZone::Never => Self::Never,
            DisplayTimeZone::Critical => Self::Critical,
        }
    }
}

// ============================================================================
// Disambiguation
// ============================================================================

/// How to resolve ambiguous times (e.g., during DST transitions).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum Disambiguation {
    #[default]
    Compatible,
    Earlier,
    Later,
    Reject,
}

impl From<Disambiguation> for temporal_rs::options::Disambiguation {
    fn from(value: Disambiguation) -> Self {
        match value {
            Disambiguation::Compatible => Self::Compatible,
            Disambiguation::Earlier => Self::Earlier,
            Disambiguation::Later => Self::Later,
            Disambiguation::Reject => Self::Reject,
        }
    }
}

// ============================================================================
// OffsetDisambiguation
// ============================================================================

/// How to handle offset disambiguation when creating ZonedDateTime.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum OffsetDisambiguation {
    #[default]
    Use,
    Prefer,
    Ignore,
    Reject,
}

impl From<OffsetDisambiguation> for temporal_rs::options::OffsetDisambiguation {
    fn from(value: OffsetDisambiguation) -> Self {
        match value {
            OffsetDisambiguation::Use => Self::Use,
            OffsetDisambiguation::Prefer => Self::Prefer,
            OffsetDisambiguation::Ignore => Self::Ignore,
            OffsetDisambiguation::Reject => Self::Reject,
        }
    }
}

// ============================================================================
// RoundingMode
// ============================================================================

/// The rounding mode for rounding operations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum RoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    #[default]
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

impl From<RoundingMode> for temporal_rs::options::RoundingMode {
    fn from(value: RoundingMode) -> Self {
        match value {
            RoundingMode::Ceil => Self::Ceil,
            RoundingMode::Floor => Self::Floor,
            RoundingMode::Expand => Self::Expand,
            RoundingMode::Trunc => Self::Trunc,
            RoundingMode::HalfCeil => Self::HalfCeil,
            RoundingMode::HalfFloor => Self::HalfFloor,
            RoundingMode::HalfExpand => Self::HalfExpand,
            RoundingMode::HalfTrunc => Self::HalfTrunc,
            RoundingMode::HalfEven => Self::HalfEven,
        }
    }
}

// ============================================================================
// Unit
// ============================================================================

/// A temporal unit for duration and rounding operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum Unit {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl From<Unit> for temporal_rs::options::Unit {
    fn from(value: Unit) -> Self {
        match value {
            Unit::Year => Self::Year,
            Unit::Month => Self::Month,
            Unit::Week => Self::Week,
            Unit::Day => Self::Day,
            Unit::Hour => Self::Hour,
            Unit::Minute => Self::Minute,
            Unit::Second => Self::Second,
            Unit::Millisecond => Self::Millisecond,
            Unit::Microsecond => Self::Microsecond,
            Unit::Nanosecond => Self::Nanosecond,
        }
    }
}

// ============================================================================
// TransitionDirection
// ============================================================================

/// The direction to search for time zone transitions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[data_object(error = "range")]
pub enum TransitionDirection {
    #[default]
    Next,
    Previous,
}

impl From<TransitionDirection> for temporal_rs::provider::TransitionDirection {
    fn from(value: TransitionDirection) -> Self {
        match value {
            TransitionDirection::Next => Self::Next,
            TransitionDirection::Previous => Self::Previous,
        }
    }
}
