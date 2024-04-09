

enum Statement { // Yield, Await, Return
    BlockStatement(BlockStatement), // ?Yield, ?Await, ?Return
    VariableStatement(VariableStatement), // ?Yield, ?Await
    EmptyStatement,
    ExpressionStatement(ExpressionStatement), // ?Yield, ?Await
    IfStatement(IfStatement), // ?Yield, ?Await, ?Return
    BreakableStatement(BreakableStatement), // ?Yield, ?Await, ?Return
    ContinueStatement(ContinueStatement), // ?Yield, ?Await
    BreakStatement(BreakStatement), // ?Yield, ?Await
    ReturnStatement(ReturnStatement), // +Return, ?Yield, ?Await
    WithStatement(WithStatement), // ?Yield, ?Await, ?Return
    LabelledStatement(LabelledStatement), // ?Yield, ?Await, ?Return
    ThrowStatement(ThrowStatement), // ?Yield, ?Await
    TryStatement(TryStatement), // ?Yield, ?Await, ?Return
    DebuggerStatement
}