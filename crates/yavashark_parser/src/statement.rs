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

pub enum Statement {
    // Yield, Await, Return
    BlockStatement(BlockStatement),       // ?Yield, ?Await, ?Return
    VariableStatement(VariableStatement), // ?Yield, ?Await
    EmptyStatement,
    ExpressionStatement(ExpressionStatement), // ?Yield, ?Await
    IfStatement(IfStatement),                 // ?Yield, ?Await, ?Return
    BreakableStatement(BreakableStatement),   // ?Yield, ?Await, ?Return
    ContinueStatement(ContinueStatement),     // ?Yield, ?Await
    BreakStatement(BreakStatement),           // ?Yield, ?Await
    ReturnStatement(ReturnStatement),         // +Return, ?Yield, ?Await
    WithStatement(WithStatement),             // ?Yield, ?Await, ?Return
    LabelledStatement(LabelledStatement),     // ?Yield, ?Await, ?Return
    ThrowStatement(ThrowStatement),           // ?Yield, ?Await
    TryStatement(TryStatement),               // ?Yield, ?Await, ?Return
    DebuggerStatement,
}
