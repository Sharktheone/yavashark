mod block;
mod variable;
mod expression;
mod r#if;
mod breakable;
mod r#continue;
mod r#break;
mod r#return;
mod with;
mod labelled;
mod throw;
mod r#try;

use block::BlockStatement;
use variable::VariableStatement;
use expression::ExpressionStatement;
use r#if::IfStatement;
use breakable::BreakableStatement;
use r#continue::ContinueStatement;
use r#break::BreakStatement;
use r#return::ReturnStatement;
use with::WithStatement;
use labelled::LabelledStatement;
use throw::ThrowStatement;
use r#try::TryStatement;

enum Statement {
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
