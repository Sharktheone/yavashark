use crate::expression::conditional::ConditionalExpression;
use crate::expression::r#yield::YieldExpression;

pub enum AssigmentExpression {
    ConditionalExpression(ConditionalExpression),
    YieldExpression(YieldExpression),
    ArrowFunction(ArrowFunction),
    AsyncArrowFunction(AsyncArrowFunction),
    LeftHandSideExpression(LeftHandSideExpression),
}


pub struct ArrowFunction {
}

pub struct AsyncArrowFunction {
}

pub enum LeftHandSideExpression {
    Equal(Box<AssigmentExpression>),                        // =
    Operator(AssignmentOperator, Box<AssigmentExpression>), // +=, -=, *=, /=, %=, **=, <<=, >>=, >>>=, &=, ^=, |=
    AndAndEqual(Box<AssigmentExpression>),                  // &&=
    OrOrEqual(Box<AssigmentExpression>),                    // ||=
    QuestionQuestionEqual(Box<AssigmentExpression>),        // ??=
}

pub enum AssignmentOperator {
    Asterisk,              // *=
    Slash,                 // /=
    Percent,               // %=
    Plus,                  // +=
    Minus,                 // -=
    LessLess,              // <<=
    GreaterGreater,        // >>=
    GreaterGreaterGreater, // >>>=
    And,                   // &=
    Caret,                 // ^=
    Or,                    // |=
    AsteriskAsterisk,      // **=
}
