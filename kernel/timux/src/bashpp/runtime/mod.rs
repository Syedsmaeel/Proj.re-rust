//! Bash++ Runtime — executes AST nodes
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use crate::bashpp::parser::{Ast, Expr, Value, Command};

pub type ExitCode = i32;

/// Variable environment — a stack of scopes
pub struct Env {
    scopes: Vec<BTreeMap<String, Value>>,
    pub last_exit: ExitCode,
}

impl Env {
    pub fn new() -> Self {
        let mut e = Self { scopes: Vec::new(), last_exit: 0 };
        e.push_scope();
        // Built-in vars
        e.set("SHELL",   Value::Str(String::from("bashpp")));
        e.set("VERSION", Value::Str(String::from("0.1.0")));
        e.set("PS1",     Value::Str(String::from("bashpp> ")));
        e
    }

    pub fn push_scope(&mut self) { self.scopes.push(BTreeMap::new()); }
    pub fn pop_scope(&mut self)  { self.scopes.pop(); }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) { return Some(v); }
        }
        None
    }

    pub fn set(&mut self, name: &str, val: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(String::from(name), val);
        }
    }

    pub fn unset(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.remove(name).is_some() { return; }
        }
    }

    pub fn export(&mut self, name: &str, val: Option<Value>) {
        let v = val.unwrap_or_else(|| self.get(name).cloned().unwrap_or(Value::Null));
        self.set(name, v);
    }
}

/// Output sink — collects text output from the runtime
pub struct Output {
    pub lines: Vec<String>,
}

impl Output {
    pub fn new() -> Self { Self { lines: Vec::new() } }
    pub fn push(&mut self, line: String) { self.lines.push(line); }
    pub fn push_str(&mut self, s: &str) { self.lines.push(String::from(s)); }
    pub fn drain(&mut self) -> Vec<String> { core::mem::take(&mut self.lines) }
}

/// Control flow signal
#[derive(Debug)]
pub enum Flow {
    Normal,
    Return(Option<Value>),
    Break,
    Continue,
    Exit(ExitCode),
}

/// The Bash++ runtime interpreter
pub struct Runtime {
    pub env:    Env,
    pub output: Output,
    fns:        BTreeMap<String, (Vec<String>, Vec<Ast>)>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            env:    Env::new(),
            output: Output::new(),
            fns:    BTreeMap::new(),
        }
    }

    /// Execute a list of AST nodes
    pub fn exec_block(&mut self, nodes: &[Ast]) -> (Flow, Option<Value>) {
        for node in nodes {
            let (flow, val) = self.exec(node);
            match flow {
                Flow::Normal => {}
                other => return (other, val),
            }
        }
        (Flow::Normal, None)
    }

    /// Execute a single AST node
    pub fn exec(&mut self, node: &Ast) -> (Flow, Option<Value>) {
        match node {
            Ast::Expr(expr) => {
                let val = self.eval(expr);
                (Flow::Normal, Some(val))
            }
            Ast::If { cond, then, else_ } => {
                let cv = self.eval(cond);
                if self.is_truthy(&cv) {
                    self.env.push_scope();
                    let r = self.exec_block(then);
                    self.env.pop_scope();
                    r
                } else if let Some(eb) = else_ {
                    self.env.push_scope();
                    let r = self.exec_block(eb);
                    self.env.pop_scope();
                    r
                } else {
                    (Flow::Normal, None)
                }
            }
            Ast::For { var, iter, body } => {
                let items = self.eval_iter(iter);
                for item in items {
                    self.env.push_scope();
                    self.env.set(var, item);
                    let (flow, val) = self.exec_block(body);
                    self.env.pop_scope();
                    match flow {
                        Flow::Break    => break,
                        Flow::Continue => continue,
                        Flow::Normal   => {}
                        other          => return (other, val),
                    }
                }
                (Flow::Normal, None)
            }
            Ast::While { cond, body } => {
                loop {
                    let cv = self.eval(cond);
                    if !self.is_truthy(&cv) { break; }
                    self.env.push_scope();
                    let (flow, val) = self.exec_block(body);
                    self.env.pop_scope();
                    match flow {
                        Flow::Break    => break,
                        Flow::Continue => continue,
                        Flow::Normal   => {}
                        other          => return (other, val),
                    }
                }
                (Flow::Normal, None)
            }
            Ast::Loop { body } => {
                loop {
                    self.env.push_scope();
                    let (flow, val) = self.exec_block(body);
                    self.env.pop_scope();
                    match flow {
                        Flow::Break    => break,
                        Flow::Normal | Flow::Continue => continue,
                        other => return (other, val),
                    }
                }
                (Flow::Normal, None)
            }
            Ast::FnDef { name, params, body } => {
                self.fns.insert(name.clone(), (params.clone(), body.clone()));
                (Flow::Normal, None)
            }
            Ast::Return(val) => {
                let v = val.as_ref().map(|e| self.eval(e));
                (Flow::Return(v), None)
            }
            Ast::Break    => (Flow::Break, None),
            Ast::Continue => (Flow::Continue, None),
            Ast::Export(name, val) => {
                let v = val.as_ref().map(|e| self.eval(e));
                self.env.export(name, v);
                (Flow::Normal, None)
            }
            Ast::Unset(name) => { self.env.unset(name); (Flow::Normal, None) }
            Ast::Source(path) => {
                self.output.push(alloc::format!("[source {}]", path));
                (Flow::Normal, None)
            }
            Ast::Match { val, arms } => {
                let v = self.eval(val);
                for (pat, body) in arms {
                    let pv = self.eval(pat);
                    if self.values_match(&v, &pv) {
                        self.env.push_scope();
                        let r = self.exec_block(body);
                        self.env.pop_scope();
                        return r;
                    }
                }
                (Flow::Normal, None)
            }
            Ast::Block(nodes) => {
                self.env.push_scope();
                let r = self.exec_block(nodes);
                self.env.pop_scope();
                r
            }
        }
    }

    /// Evaluate an expression to a value
    pub fn eval(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Lit(v) => v.clone(),
            Expr::Var(name) => self.env.get(name).cloned().unwrap_or(Value::Null),
            Expr::Interp(name) => {
                let v = self.env.get(name).cloned().unwrap_or(Value::Null);
                Value::Str(self.to_str(&v))
            }
            Expr::Assign { name, val } => {
                let v = self.eval(val);
                self.env.set(name, v.clone());
                v
            }
            Expr::Cmd(cmd) => self.exec_cmd(cmd),
            Expr::Pipeline(parts) => {
                let mut last = Value::Null;
                let mut prev_out: Option<Value> = None;
                for (i, part) in parts.iter().enumerate() {
                    let _ = prev_out;
                    last = self.eval(part);
                    prev_out = Some(last.clone());
                }
                last
            }
            Expr::And(l, r) => {
                let lv = self.eval(l);
                if self.is_truthy(&lv) { self.eval(r) } else { lv }
            }
            Expr::Or(l, r) => {
                let lv = self.eval(l);
                if self.is_truthy(&lv) { lv } else { self.eval(r) }
            }
            Expr::RenderExpr { text, color } => {
                let colored = if let Some(c) = color {
                    alloc::format!("[{}|{}]", c, text)
                } else {
                    text.clone()
                };
                self.output.push(colored.clone());
                Value::Str(colored)
            }
            Expr::IconExpr(icon) => {
                self.output.push(icon.clone());
                Value::Str(icon.clone())
            }
            Expr::ThemeExpr(name) => {
                self.output.push(alloc::format!("[theme:{}]", name));
                Value::Str(name.clone())
            }
            Expr::PaneExpr(_) => Value::Str(String::from("[pane]")),
            Expr::TabExpr(tab) => Value::Str(alloc::format!("[tab:{}]", tab.name)),
            Expr::WidgetExpr(_) => Value::Str(String::from("[widget]")),
            Expr::BinOp { op, lhs, rhs } => {
                let l = self.eval(lhs);
                let r = self.eval(rhs);
                self.apply_binop(op, &l, &r)
            }
            Expr::SubShell(nodes) => {
                self.env.push_scope();
                let (_, val) = self.exec_block(nodes);
                self.env.pop_scope();
                val.unwrap_or(Value::Null)
            }
            Expr::Redirect { expr, .. } => self.eval(expr),
        }
    }

    fn exec_cmd(&mut self, cmd: &Command) -> Value {
        let args: Vec<Value> = cmd.args.iter().map(|a| self.eval(a)).collect();
        let str_args: Vec<String> = args.iter().map(|v| self.to_str(v)).collect();

        match cmd.name.as_str() {
            "echo"    => { let out = str_args.join(" "); self.output.push(out.clone()); Value::Str(out) }
            "print"   => { let out = str_args.join(" "); self.output.push(out.clone()); Value::Str(out) }
            "println" => { let out = alloc::format!("{}\n", str_args.join(" ")); self.output.push(out.clone()); Value::Str(out) }
            "cd"      => { self.env.set("PWD", Value::Str(str_args.first().cloned().unwrap_or_default())); Value::Null }
            "pwd"     => { let p = self.to_str(self.env.get("PWD").unwrap_or(&Value::Str(String::from("/")))); self.output.push(p.clone()); Value::Str(p) }
            "export"  => { for a in &str_args { self.env.export(a, None); } Value::Null }
            "unset"   => { for a in &str_args { self.env.unset(a); } Value::Null }
            "true"    => Value::Bool(true),
            "false"   => Value::Bool(false),
            "not"     => { let v = args.first().cloned().unwrap_or(Value::Null); Value::Bool(!self.is_truthy(&v)) }
            "len"     => { match args.first() { Some(Value::List(l))=>Value::Int(l.len() as i64), Some(Value::Str(s))=>Value::Int(s.len() as i64), _=>Value::Int(0) } }
            "str"     => { Value::Str(str_args.join("")) }
            "int"     => { Value::Int(str_args.first().and_then(|s|s.parse().ok()).unwrap_or(0)) }
            "list"    => { Value::List(args) }
            other => {
                // try calling a user-defined function
                if let Some((params, body)) = self.fns.get(other).cloned() {
                    self.env.push_scope();
                    for (p, a) in params.iter().zip(args.iter()) {
                        self.env.set(p, a.clone());
                    }
                    let (_, val) = self.exec_block(&body);
                    self.env.pop_scope();
                    val.unwrap_or(Value::Null)
                } else {
                    self.output.push(alloc::format!("[exec: {}]", other));
                    Value::Null
                }
            }
        }
    }

    fn eval_iter(&mut self, expr: &Expr) -> Vec<Value> {
        match self.eval(expr) {
            Value::List(l) => l,
            Value::Str(s)  => s.split_whitespace().map(|w| Value::Str(String::from(w))).collect(),
            other           => alloc::vec![other],
        }
    }

    pub fn is_truthy(&self, v: &Value) -> bool {
        match v {
            Value::Bool(b)  => *b,
            Value::Int(n)   => *n != 0,
            Value::Str(s)   => !s.is_empty() && s != "false" && s != "0",
            Value::List(l)  => !l.is_empty(),
            Value::Null     => false,
        }
    }

    fn values_match(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Str(x),  Value::Str(y))  => x == y,
            (Value::Int(x),  Value::Int(y))  => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Null,    Value::Null)    => true,
            _ => false,
        }
    }

    fn apply_binop(&self, op: &crate::bashpp::parser::BinOp, l: &Value, r: &Value) -> Value {
        use crate::bashpp::parser::BinOp::*;
        match (op, l, r) {
            (Add, Value::Int(a), Value::Int(b)) => Value::Int(a+b),
            (Sub, Value::Int(a), Value::Int(b)) => Value::Int(a-b),
            (Mul, Value::Int(a), Value::Int(b)) => Value::Int(a*b),
            (Div, Value::Int(a), Value::Int(b)) => Value::Int(if *b!=0{a/b}else{0}),
            (Eq,  a, b) => Value::Bool(self.values_match(a,b)),
            (Ne,  a, b) => Value::Bool(!self.values_match(a,b)),
            (Lt,  Value::Int(a), Value::Int(b)) => Value::Bool(a<b),
            (Gt,  Value::Int(a), Value::Int(b)) => Value::Bool(a>b),
            (Le,  Value::Int(a), Value::Int(b)) => Value::Bool(a<=b),
            (Ge,  Value::Int(a), Value::Int(b)) => Value::Bool(a>=b),
            _ => Value::Null,
        }
    }

    pub fn to_str(&self, v: &Value) -> String {
        match v {
            Value::Str(s)  => s.clone(),
            Value::Int(n)  => alloc::format!("{}", n),
            Value::Bool(b) => String::from(if *b{"true"}else{"false"}),
            Value::List(l) => l.iter().map(|v| self.to_str(v)).collect::<Vec<_>>().join(" "),
            Value::Null    => String::new(),
        }
    }
}
