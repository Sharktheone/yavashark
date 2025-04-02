use swc_ecma_ast::{
    AssignTarget, AssignTargetPat, BlockStmt, Class, ClassDecl, ClassMember, DebuggerStmt, Decl,
    Expr, ExprStmt, ForHead, MemberProp, OptChainBase, Pat, Prop, PropName, PropOrSpread,
    SimpleAssignTarget, Stmt, SuperProp, UsingDecl, VarDeclOrExpr, WithStmt,
};

pub trait ASTNode {
    fn has_call(&self) -> bool;
}

impl ASTNode for Stmt {
    fn has_call(&self) -> bool {
        match self {
            Self::Block(block) => block.has_call(),
            Self::Empty(_) => false,
            Self::Debugger(d) => d.has_call(),
            Self::With(w) => w.has_call(),
            Self::Return(r) => r.has_call(),
            Self::Labeled(l) => l.has_call(),
            Self::Break(b) => b.has_call(),
            Self::Continue(c) => c.has_call(),
            Self::If(i) => i.has_call(),
            Self::Switch(s) => s.has_call(),
            Self::Throw(t) => t.has_call(),
            Self::Try(t) => t.has_call(),
            Self::While(w) => w.has_call(),
            Self::DoWhile(d) => d.has_call(),
            Self::For(f) => f.has_call(),
            Self::ForIn(f) => f.has_call(),
            Self::ForOf(f) => f.has_call(),
            Self::Decl(d) => d.has_call(),
            Self::Expr(expr) => expr.has_call(),
        }
    }
}

impl ASTNode for BlockStmt {
    fn has_call(&self) -> bool {
        self.stmts.iter().any(ASTNode::has_call)
    }
}

impl ASTNode for DebuggerStmt {
    fn has_call(&self) -> bool {
        false
    }
}

impl ASTNode for WithStmt {
    fn has_call(&self) -> bool {
        self.obj.has_call() || self.body.has_call()
    }
}

impl ASTNode for swc_ecma_ast::ReturnStmt {
    fn has_call(&self) -> bool {
        self.arg.as_deref().is_some_and(ASTNode::has_call)
    }
}

impl ASTNode for swc_ecma_ast::LabeledStmt {
    fn has_call(&self) -> bool {
        self.body.has_call()
    }
}

impl ASTNode for swc_ecma_ast::BreakStmt {
    fn has_call(&self) -> bool {
        false
    }
}

impl ASTNode for swc_ecma_ast::ContinueStmt {
    fn has_call(&self) -> bool {
        false
    }
}

impl ASTNode for swc_ecma_ast::IfStmt {
    fn has_call(&self) -> bool {
        self.test.has_call()
            || self.cons.has_call()
            || self.alt.as_deref().is_some_and(ASTNode::has_call)
    }
}

impl ASTNode for swc_ecma_ast::SwitchStmt {
    fn has_call(&self) -> bool {
        self.discriminant.has_call()
            || self.cases.iter().any(|c| {
                c.test.as_deref().is_some_and(ASTNode::has_call)
                    || c.cons.iter().any(ASTNode::has_call)
            })
    }
}

impl ASTNode for swc_ecma_ast::ThrowStmt {
    fn has_call(&self) -> bool {
        self.arg.has_call()
    }
}

impl ASTNode for swc_ecma_ast::TryStmt {
    fn has_call(&self) -> bool {
        self.block.has_call()
            || self.handler.as_ref().is_some_and(|h| h.body.has_call())
            || self.finalizer.as_ref().is_some_and(ASTNode::has_call)
    }
}

impl ASTNode for swc_ecma_ast::WhileStmt {
    fn has_call(&self) -> bool {
        self.test.has_call() || self.body.has_call()
    }
}

impl ASTNode for swc_ecma_ast::DoWhileStmt {
    fn has_call(&self) -> bool {
        self.test.has_call() || self.body.has_call()
    }
}

impl ASTNode for swc_ecma_ast::ForStmt {
    fn has_call(&self) -> bool {
        self.init.as_ref().is_some_and(ASTNode::has_call)
            || self.test.as_deref().is_some_and(ASTNode::has_call)
            || self.update.as_deref().is_some_and(ASTNode::has_call)
            || self.body.has_call()
    }
}

impl ASTNode for swc_ecma_ast::ForInStmt {
    fn has_call(&self) -> bool {
        self.left.has_call() || self.right.has_call() || self.body.has_call()
    }
}

impl ASTNode for swc_ecma_ast::ForOfStmt {
    fn has_call(&self) -> bool {
        self.left.has_call() || self.right.has_call() || self.body.has_call()
    }
}

impl ASTNode for Decl {
    fn has_call(&self) -> bool {
        match self {
            Self::Class(c) => c.has_call(),
            Self::Fn(f) => f.has_call(),
            Self::Var(v) => v.has_call(),
            Self::Using(u) => u.has_call(),

            _ => false,
        }
    }
}

impl ASTNode for ClassDecl {
    fn has_call(&self) -> bool {
        self.class.has_call()
    }
}

impl ASTNode for swc_ecma_ast::FnDecl {
    fn has_call(&self) -> bool {
        false
    }
}

impl ASTNode for UsingDecl {
    fn has_call(&self) -> bool {
        self.decls
            .iter()
            .any(|d| d.init.as_deref().is_some_and(ASTNode::has_call) || d.name.has_call())
    }
}

impl ASTNode for Class {
    fn has_call(&self) -> bool {
        self.super_class.as_deref().is_some_and(ASTNode::has_call)
            || self.body.iter().any(|m| match m {
                ClassMember::StaticBlock(s) => s.body.has_call(),
                _ => false,
            })
    }
}

impl ASTNode for ExprStmt {
    fn has_call(&self) -> bool {
        self.expr.has_call()
    }
}

impl ASTNode for VarDeclOrExpr {
    fn has_call(&self) -> bool {
        match self {
            Self::VarDecl(v) => v.has_call(),
            Self::Expr(e) => e.has_call(),
        }
    }
}

impl ASTNode for swc_ecma_ast::VarDecl {
    fn has_call(&self) -> bool {
        self.decls
            .iter()
            .any(|d| d.init.as_deref().is_some_and(ASTNode::has_call) || d.name.has_call())
    }
}

impl ASTNode for ForHead {
    fn has_call(&self) -> bool {
        match self {
            Self::VarDecl(v) => v.has_call(),
            Self::UsingDecl(u) => u.has_call(),
            Self::Pat(e) => e.has_call(),
        }
    }
}

impl ASTNode for Pat {
    fn has_call(&self) -> bool {
        match self {
            Self::Ident(_) | Self::Invalid(_) => false,
            Self::Array(array) => array.has_call(),
            Self::Rest(rest) => rest.has_call(),
            Self::Object(obj) => obj.has_call(),
            Self::Assign(assign) => assign.has_call(),
            Self::Expr(e) => e.has_call(),
        }
    }
}

impl ASTNode for swc_ecma_ast::ArrayPat {
    fn has_call(&self) -> bool {
        self.elems
            .iter()
            .any(|e| e.as_ref().is_some_and(ASTNode::has_call))
    }
}

impl ASTNode for swc_ecma_ast::RestPat {
    fn has_call(&self) -> bool {
        self.arg.has_call()
    }
}

impl ASTNode for swc_ecma_ast::ObjectPat {
    fn has_call(&self) -> bool {
        self.props.iter().any(|p| match p {
            swc_ecma_ast::ObjectPatProp::KeyValue(kv) => kv.key.has_call() || kv.value.has_call(),
            swc_ecma_ast::ObjectPatProp::Assign(a) => {
                a.value.as_deref().is_some_and(ASTNode::has_call)
            }
            swc_ecma_ast::ObjectPatProp::Rest(r) => r.arg.has_call(),
        })
    }
}

impl ASTNode for swc_ecma_ast::AssignPat {
    fn has_call(&self) -> bool {
        self.left.has_call() || self.right.has_call()
    }
}

impl ASTNode for PropName {
    fn has_call(&self) -> bool {
        match self {
            Self::Computed(c) => c.expr.has_call(),
            _ => false,
        }
    }
}

impl ASTNode for Expr {
    fn has_call(&self) -> bool {
        match self {
            Self::Array(a) => a
                .elems
                .iter()
                .any(|i| i.as_ref().is_some_and(|expr| expr.expr.has_call())),
            Self::Object(o) => o.props.iter().any(ASTNode::has_call),
            Self::Unary(u) => u.arg.has_call(),
            Self::Update(u) => u.arg.has_call(),
            Self::Bin(b) => b.left.has_call() || b.right.has_call(),
            Self::Assign(a) => a.left.has_call() || a.right.has_call(),
            Self::Member(m) => m.obj.has_call(),
            Self::SuperProp(s) => s.prop.has_call(),
            Self::Cond(c) => c.test.has_call() || c.cons.has_call() || c.alt.has_call(),
            Self::Seq(s) => s.exprs.iter().any(|e| e.has_call()),
            Self::Tpl(tpl) => tpl.exprs.iter().any(|e| e.has_call()),
            Self::TaggedTpl(_) => true,
            Self::Class(c) => c.class.has_call(),
            Self::Yield(y) => y.arg.as_deref().is_some_and(ASTNode::has_call),
            Self::Await(a) => a.arg.has_call(),
            Self::Paren(p) => p.expr.has_call(),
            Self::OptChain(o) => o.base.has_call(),

            _ => false,
        }
    }
}

impl ASTNode for PropOrSpread {
    fn has_call(&self) -> bool {
        match self {
            Self::Spread(s) => s.expr.has_call(),
            Self::Prop(e) => e.has_call(),
        }
    }
}

impl ASTNode for Prop {
    fn has_call(&self) -> bool {
        match self {
            Self::Shorthand(_) => false,
            Self::KeyValue(kv) => kv.key.has_call() || kv.value.has_call(),
            Self::Assign(a) => a.value.has_call(),
            Self::Getter(g) => g.key.has_call(),
            Self::Setter(s) => s.key.has_call(),
            Self::Method(m) => m.key.has_call(),
        }
    }
}

impl ASTNode for AssignTarget {
    fn has_call(&self) -> bool {
        match self {
            Self::Simple(s) => s.has_call(),
            Self::Pat(p) => p.has_call(),
        }
    }
}

impl ASTNode for SimpleAssignTarget {
    fn has_call(&self) -> bool {
        match self {
            Self::Member(m) => m.obj.has_call() | m.prop.has_call(),
            Self::SuperProp(s) => s.prop.has_call(),
            Self::Paren(p) => p.expr.has_call(),
            Self::OptChain(o) => o.base.has_call(),
            _ => false,
        }
    }
}

impl ASTNode for AssignTargetPat {
    fn has_call(&self) -> bool {
        match self {
            Self::Array(a) => a.has_call(),
            Self::Object(o) => o.has_call(),
            Self::Invalid(_) => false,
        }
    }
}

impl ASTNode for SuperProp {
    fn has_call(&self) -> bool {
        match self {
            Self::Ident(_) => false,
            Self::Computed(c) => c.expr.has_call(),
        }
    }
}

impl ASTNode for OptChainBase {
    fn has_call(&self) -> bool {
        match self {
            Self::Member(m) => m.obj.has_call() || m.prop.has_call(),
            Self::Call(_) => true,
        }
    }
}

impl ASTNode for MemberProp {
    fn has_call(&self) -> bool {
        match self {
            Self::Ident(_) | Self::PrivateName(_) => false,
            Self::Computed(c) => c.expr.has_call(),
        }
    }
}
