pub fn builtin_string(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    let mut out = String::new();
    for a in &args {
        match &**a {
            Expr::Num(n) => out.push_str(&n.to_string()),
            Expr::Str(s) => out.push_str(s),
            Expr::Var(s) => out.push_str(s),
            Expr::List(l) => out.push_str(&format!("{:?}", l)),
            Expr::Lambda(_, _) => out.push_str("<lambda>"),
            Expr::Builtin(_) => {},
        }
    }
    Rc::new(Expr::Str(out))
}
use std::rc::Rc;
use crate::interpreter::{Expr, Env, eval};
// use std::ops::Rem;

pub fn builtin_add(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    Rc::new(Expr::Num(args.iter().map(|x| match &**x { Expr::Num(n)=>*n, _=>panic!() }).sum()))
}

pub fn builtin_sub(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    if args.is_empty() { panic!(); }
    let mut it = args.iter();
    let first = match &**it.next().unwrap() { Expr::Num(n)=>*n, _=>panic!() };
    Rc::new(Expr::Num(it.fold(first, |a,x| a - match &**x { Expr::Num(n)=>*n, _=>panic!() })))
}

pub fn builtin_mul(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    Rc::new(Expr::Num(args.iter().map(|x| match &**x { Expr::Num(n)=>*n, _=>panic!() }).product()))
}

pub fn builtin_div(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    if args.len() != 2 { panic!("/ expects 2 arguments"); }
    let a = match &*args[0] { Expr::Num(n) => *n, _ => panic!() };
    let b = match &*args[1] { Expr::Num(n) => *n, _ => panic!() };
    if b == 0 { panic!("division by zero"); }
    Rc::new(Expr::Num(a / b))
}

pub fn builtin_rem(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    if args.len() != 2 { panic!("% expects 2 arguments"); }
    let a = match &*args[0] { Expr::Num(n) => *n, _ => panic!() };
    let b = match &*args[1] { Expr::Num(n) => *n, _ => panic!() };
    if b == 0 { panic!("modulo by zero"); }
    Rc::new(Expr::Num(a % b))
}

pub fn builtin_pow(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    if args.len() != 2 { panic!("pow expects 2 arguments"); }
    let a = match &*args[0] { Expr::Num(n) => *n, _ => panic!() };
    let b = match &*args[1] { Expr::Num(n) => *n, _ => panic!() };
    Rc::new(Expr::Num(a.pow(b as u32)))
}

pub fn builtin_print(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    let mut out = String::new();
    for a in &args {
        match &**a {
            Expr::Num(n) => out.push_str(&n.to_string()),
            Expr::Str(s) => out.push_str(s),
            Expr::Var(s) => {
                if s.starts_with("خطا: ") {
                    out.push_str("");
                } else {
                    out.push_str(s);
                }
            },
            Expr::List(l) => out.push_str(&format!("{:?}", l)),
            Expr::Lambda(_, _) => out.push_str("<lambda>"),
            Expr::Builtin(_) => {}, // do not print anything for builtin
        }
        out.push(' ');
    }
    println!("{}", out.trim_end());
    Rc::new(Expr::Str(out.trim_end().to_string()))
}
pub fn builtin_lt(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    if args.len() != 2 { panic!("< expects 2 arguments"); }
    let a = match &*args[0] { Expr::Num(n) => *n, _ => panic!() };
    let b = match &*args[1] { Expr::Num(n) => *n, _ => panic!() };
    Rc::new(Expr::Num(if a < b { 1 } else { 0 }))
}
pub fn builtin_le(args: Vec<Rc<Expr>>, _: &mut Env) -> Rc<Expr> {
    if args.len() != 2 { panic!("<= expects 2 arguments"); }
    let a = match &*args[0] { Expr::Num(n) => *n, _ => panic!() };
    let b = match &*args[1] { Expr::Num(n) => *n, _ => panic!() };
    Rc::new(Expr::Num(if a <= b { 1 } else { 0 }))
}
pub fn builtin_set(args: Vec<Rc<Expr>>, env: &mut Env) -> Rc<Expr> {
    if args.len() != 2 {
        return Rc::new(Expr::Var("خطا: set! expects 2 arguments".to_string()));
    }
    let name = match &*args[0] {
        Expr::Var(n) => n.clone(),
        _ => return Rc::new(Expr::Var("خطا: set! first argument must be variable name".to_string())),
    };
    env.set(&name, args[1].clone());
    args[1].clone()
}
pub fn builtin_while(args: Vec<Rc<Expr>>, env: &mut Env) -> Rc<Expr> {
    if args.len() < 2 {
        return Rc::new(Expr::Var("خطا: while expects condition and body".to_string()));
    }
    let mut last = Rc::new(Expr::List(vec![]));
    loop {
        let cond = match &*eval(args[0].clone(), env) {
            Expr::Num(n) => *n,
            Expr::Var(msg) => {
                return Rc::new(Expr::Var(format!("خطا در شرط while: {}", msg)));
            },
            _ => return Rc::new(Expr::Var("خطا: شرط while باید عدد باشد".to_string())),
        };
        if cond == 0 { break; }
        for expr in &args[1..] {
            last = eval(expr.clone(), env);
        }
    }
    last
}
pub fn builtin_if(args: Vec<Rc<Expr>>, env: &mut Env) -> Rc<Expr> {
    if args.len() != 3 { panic!("if expects 3 arguments"); }
    let cond = match &*args[0] { Expr::Num(n) => *n, _ => panic!() };
    if cond != 0 {
        eval(Rc::new((*args[1]).clone()), env)
    } else {
        eval(Rc::new((*args[2]).clone()), env)
    }
}
pub fn builtin_begin(args: Vec<Rc<Expr>>, env: &mut Env) -> Rc<Expr> {
    let mut result = Rc::new(Expr::List(vec![]));
    for expr in args {
        result = eval(expr, env);
    }
    result
}

pub fn builtin_for(args: Vec<Rc<Expr>>, env: &mut Env) -> Rc<Expr> {
    // (for var from to body)
    if args.len() != 4 { panic!("for expects 4 arguments: var, from, to, body"); }
    let var = match &*args[0] { Expr::Var(v) => v.clone(), _ => panic!("first arg must be var") };
    let from = match &*args[1] { Expr::Num(n) => *n, _ => panic!("second arg must be num") };
    let to = match &*args[2] { Expr::Num(n) => *n, _ => panic!("third arg must be num") };
    let body = &args[3];
    let mut last = Rc::new(Expr::Num(0));
    for i in from..=to {
        env.set(&var, Rc::new(Expr::Num(i)));
        last = eval(body.clone(), env);
    }
    last
}

pub fn add_builtins(env: &mut Env) {
    env.set("+", Rc::new(Expr::Builtin(builtin_add)));
    env.set("-", Rc::new(Expr::Builtin(builtin_sub)));
    env.set("*", Rc::new(Expr::Builtin(builtin_mul)));
    env.set("/", Rc::new(Expr::Builtin(builtin_div)));
    env.set("%", Rc::new(Expr::Builtin(builtin_rem)));
    env.set("pow", Rc::new(Expr::Builtin(builtin_pow)));
    env.set("print", Rc::new(Expr::Builtin(builtin_print)));
    env.set("string", Rc::new(Expr::Builtin(builtin_string)));
    env.set("<", Rc::new(Expr::Builtin(builtin_lt)));
    env.set("<=", Rc::new(Expr::Builtin(builtin_le)));
    env.set(">", Rc::new(Expr::Builtin(|args, _| {
        if args.len() != 2 { panic!("> expects 2 arguments"); }
        let a = match &*args[0] { Expr::Num(n) => *n, _ => panic!() };
        let b = match &*args[1] { Expr::Num(n) => *n, _ => panic!() };
        Rc::new(Expr::Num(if a > b { 1 } else { 0 }))
    })));
    env.set(">=", Rc::new(Expr::Builtin(|args, _| {
        if args.len() != 2 { panic!(">= expects 2 arguments"); }
        let a = match &*args[0] { Expr::Num(n) => *n, _ => panic!() };
        let b = match &*args[1] { Expr::Num(n) => *n, _ => panic!() };
        Rc::new(Expr::Num(if a >= b { 1 } else { 0 }))
    })));
    env.set("set!", Rc::new(Expr::Builtin(builtin_set)));
    env.set("while", Rc::new(Expr::Builtin(builtin_while)));
    env.set("if", Rc::new(Expr::Builtin(builtin_if)));
    env.set("begin", Rc::new(Expr::Builtin(builtin_begin)));
    env.set("for", Rc::new(Expr::Builtin(builtin_for)));
}
