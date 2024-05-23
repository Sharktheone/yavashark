use block::BlockStatement;
use breakable::BreakableStatement;
use expression::ExpressionStatement;
use labelled::LabelledStatement;
use r#break::BreakStatement;
use r#continue::ContinueStatement;
use r#if::IfStatement;
use r#return::ReturnStatement;
use r#try::TryStatement;
use throw::ThrowStatement;
use variable::VariableStatement;
use with::WithStatement;

mod block;
mod r#break;
mod breakable;
mod r#continue;
mod expression;
mod r#if;
mod labelled;
mod r#return;
mod throw;
mod r#try;
mod variable;
mod with;

pub enum Statement {
    // Yield, Await, Return
    Block(BlockStatement),       // ?Yield, ?Await, ?Return
    Variable(VariableStatement), // ?Yield, ?Await
    Empty,
    Expression(ExpressionStatement), // ?Yield, ?Await
    If(IfStatement),                 // ?Yield, ?Await, ?Return
    Breakable(BreakableStatement),   // ?Yield, ?Await, ?Return
    Continue(ContinueStatement),     // ?Yield, ?Await
    Break(BreakStatement),           // ?Yield, ?Await
    Return(ReturnStatement),         // +Return, ?Yield, ?Await
    With(WithStatement),             // ?Yield, ?Await, ?Return
    Labelled(LabelledStatement),     // ?Yield, ?Await, ?Return
    Throw(ThrowStatement),           // ?Yield, ?Await
    Try(TryStatement),               // ?Yield, ?Await, ?Return
    Debugger,
}
