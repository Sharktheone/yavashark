use super::Checker;
use crate::Result;
use smallvec::SmallVec;
use swc_ecma_ast::{
    Callee, Decl, Expr, Ident, Lit, ModuleExportName, ObjectPatProp, Pat, PropName, Stmt,
    VarDeclKind,
};

impl Checker<'_, '_> {
    // ExpressionStatement forbids the token prefix `let [`. Parentheses and
    // escaped identifiers are distinct AST nodes/spellings and end this check.
    pub(super) fn starts_with_let_bracket(mut expression: &Expr) -> bool {
        use swc_ecma_ast::{AssignTarget, MemberProp, OptChainBase, SimpleAssignTarget};

        loop {
            let member = match expression {
                Expr::Member(member) => member,
                Expr::Assign(assign) => match &assign.left {
                    AssignTarget::Simple(SimpleAssignTarget::Member(member)) => member,
                    _ => return false,
                },
                Expr::OptChain(chain) => match &*chain.base {
                    OptChainBase::Member(member) if chain.optional => {
                        expression = &member.obj;
                        continue;
                    }
                    OptChainBase::Member(member) => member,
                    OptChainBase::Call(call) => {
                        expression = &call.callee;
                        continue;
                    }
                },
                Expr::Bin(binary) => {
                    expression = &binary.left;
                    continue;
                }
                Expr::Cond(conditional) => {
                    expression = &conditional.test;
                    continue;
                }
                Expr::Seq(sequence) => {
                    let Some(first) = sequence.exprs.first() else {
                        return false;
                    };
                    expression = first;
                    continue;
                }
                Expr::Call(call) => {
                    let Callee::Expr(callee) = &call.callee else {
                        return false;
                    };
                    expression = callee;
                    continue;
                }
                Expr::TaggedTpl(template) => {
                    expression = &template.tag;
                    continue;
                }
                Expr::Update(update) if !update.prefix => {
                    expression = &update.arg;
                    continue;
                }
                _ => return false,
            };

            if matches!(member.prop, MemberProp::Computed(_))
                && matches!(&*member.obj, Expr::Ident(ident) if ident.sym == *"let")
            {
                return true;
            }

            expression = &member.obj;
        }
    }

    // Size lexical tables before their first spill. Repeated var/function names
    // are legal, so cap that estimate instead of reserving for every repetition.
    pub(super) fn direct_binding_count(stmts: &[Stmt]) -> usize {
        let mut lexical = 0;
        let mut variable = 0;

        for stmt in stmts {
            match stmt {
                Stmt::Decl(Decl::Class(_)) => lexical += 1,
                Stmt::Decl(Decl::Var(decl)) if decl.kind != VarDeclKind::Var => {
                    lexical += decl.decls.len();
                }
                Stmt::Decl(Decl::Using(decl)) => lexical += decl.decls.len(),
                Stmt::Decl(Decl::Var(decl)) => variable += decl.decls.len(),
                Stmt::Decl(Decl::Fn(_)) => variable += 1,
                _ => {}
            }
        }

        lexical + variable.min(256)
    }

    pub(super) fn strict_directive(stmts: &[Stmt]) -> bool {
        for s in stmts {
            let Stmt::Expr(e) = s else { break };
            let Expr::Lit(Lit::Str(s)) = &*e.expr else {
                break;
            };

            if s.value == *"use strict"
                && s.raw
                    .as_ref()
                    .is_none_or(|r| matches!(r.as_str(), "\"use strict\"" | "'use strict'"))
            {
                return true;
            }
        }
        false
    }

    pub(super) fn legacy_escape(raw: &str) -> bool {
        let b = raw.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'\\' {
                i += 1;

                if i < b.len()
                    && (matches!(b[i], b'1'..=b'9')
                        || b[i] == b'0' && b.get(i + 1).is_some_and(u8::is_ascii_digit))
                {
                    return true;
                }
            }
            i += 1;
        }
        false
    }

    pub(super) fn reserved(n: &str) -> bool {
        matches!(
            n,
            "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "enum"
                | "export"
                | "extends"
                | "false"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "import"
                | "in"
                | "instanceof"
                | "new"
                | "null"
                | "return"
                | "super"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
        )
    }

    pub(super) fn strict_reserved(n: &str) -> bool {
        matches!(
            n,
            "implements"
                | "interface"
                | "let"
                | "package"
                | "private"
                | "protected"
                | "public"
                | "static"
        )
    }

    pub(super) fn iteration(s: &Stmt) -> bool {
        match s {
            Stmt::For(_) | Stmt::ForIn(_) | Stmt::ForOf(_) | Stmt::While(_) | Stmt::DoWhile(_) => {
                true
            }
            Stmt::Labeled(l) => Self::iteration(&l.body),
            _ => false,
        }
    }

    pub(super) fn labelled_function(s: &Stmt) -> bool {
        match s {
            Stmt::Decl(Decl::Fn(_)) => true,
            Stmt::Labeled(l) => Self::labelled_function(&l.body),
            _ => false,
        }
    }

    pub(super) fn unparen(mut e: &Expr) -> &Expr {
        while let Expr::Paren(p) = e {
            e = &p.expr;
        }
        e
    }

    pub(super) fn import_chain(e: &Expr) -> bool {
        match e {
            Expr::Call(c) => matches!(c.callee, Callee::Import(_)),
            Expr::Member(m) => Self::import_chain(&m.obj),
            _ => false,
        }
    }

    pub(super) fn key_is(k: &PropName, n: &str) -> bool {
        match k {
            PropName::Ident(i) => i.sym == n,
            PropName::Str(s) => s.value == *n,
            _ => false,
        }
    }

    pub(super) fn export_str(n: &ModuleExportName) -> Option<&str> {
        match n {
            ModuleExportName::Ident(i) => Some(&i.sym),
            ModuleExportName::Str(s) => s.value.as_str(),
        }
    }

    pub(super) fn export_key(n: &ModuleExportName) -> &[u8] {
        match n {
            ModuleExportName::Ident(i) => i.sym.as_bytes(),
            ModuleExportName::Str(s) => s.value.as_bytes(),
        }
    }

    pub(super) fn each_name<'a>(
        p: &'a Pat,
        f: &mut impl FnMut(&'a Ident) -> Result<'a>,
    ) -> Result<'a> {
        let mut pending = SmallVec::<[&Pat; 16]>::new();
        pending.push(p);
        while let Some(p) = pending.pop() {
            match p {
                Pat::Ident(i) => f(&i.id)?,
                Pat::Array(a) => pending.extend(a.elems.iter().flatten()),
                Pat::Object(o) => {
                    for p in &o.props {
                        match p {
                            ObjectPatProp::KeyValue(k) => pending.push(&k.value),
                            ObjectPatProp::Assign(a) => f(&a.key.id)?,
                            ObjectPatProp::Rest(r) => pending.push(&r.arg),
                        }
                    }
                }
                Pat::Assign(a) => pending.push(&a.left),
                Pat::Rest(r) => pending.push(&r.arg),
                _ => {}
            }
        }

        Ok(())
    }

    pub(super) fn bound_contains(p: &Pat, name: &str) -> bool {
        let mut found = false;
        let _ = Self::each_name(p, &mut |i| {
            found |= i.sym == name;

            Ok(())
        });
        found
    }

    // SWC retains raw escapes in some IdentifierName atoms. Decode only the cursor;
    // no temporary String is needed to check the Unicode identifier grammar.
    pub(super) fn identifier_name(s: &str) -> std::result::Result<(), &'static str> {
        let mut chars = s.chars();
        let mut first = true;
        while let Some(mut c) = chars.next() {
            if c == '\\' {
                if chars.next() != Some('u') {
                    return Err("Invalid identifier escape");
                }

                let mut value = 0u32;
                let d = chars.next().ok_or("Incomplete identifier escape")?;

                if d == '{' {
                    let mut any = false;
                    loop {
                        let d = chars.next().ok_or("Unclosed identifier escape")?;

                        if d == '}' {
                            break;
                        }

                        let digit = d.to_digit(16).ok_or("Invalid identifier escape")?;
                        value = value
                            .checked_mul(16)
                            .and_then(|v| v.checked_add(digit))
                            .ok_or("Identifier escape out of range")?;
                        any = true;
                    }

                    if !any {
                        return Err("Empty identifier escape");
                    }
                } else {
                    value = d.to_digit(16).ok_or("Invalid identifier escape")?;

                    for _ in 0..3 {
                        value = value * 16
                            + chars
                                .next()
                                .and_then(|c| c.to_digit(16))
                                .ok_or("Invalid identifier escape")?;
                    }
                }
                c = char::from_u32(value).ok_or("Invalid identifier code point")?;
            }

            let valid = if first {
                matches!(c, '$' | '_') || unicode_id_start::is_id_start(c)
            } else {
                matches!(c, '$' | '_' | '\u{200c}' | '\u{200d}')
                    || unicode_id_start::is_id_continue(c)
            };

            if !valid {
                return Err("Invalid identifier character");
            }
            first = false;
        }

        if first {
            Err("Empty identifier")
        } else {
            Ok(())
        }
    }

    pub(super) fn valid_unicode_escapes(s: &str) -> bool {
        let b = s.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'\\' {
                i += 1;

                if b.get(i) == Some(&b'u') && b.get(i + 1) == Some(&b'{') {
                    i += 2;
                    let start = i;
                    while b.get(i).is_some_and(u8::is_ascii_hexdigit) {
                        i += 1;
                    }

                    if i == start || b.get(i) != Some(&b'}') {
                        return false;
                    }
                }
            }
            i += 1;
        }
        true
    }
}
