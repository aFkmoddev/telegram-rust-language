use std::{collections::HashMap, rc::Rc, cell::RefCell};

#[derive(Clone, Debug)]
pub enum Expr {
    Num(i64),
    Var(String),
    Str(String),
    List(Vec<Expr>),
    Lambda(Vec<String>, Rc<Expr>),
    Builtin(fn(Vec<Rc<Expr>>, &mut Env) -> Rc<Expr>),
}

#[derive(Clone)]
pub struct Env(pub Rc<RefCell<HashMap<String, Rc<Expr>>>>);

impl Env {
    pub fn new() -> Self { Env(Rc::new(RefCell::new(HashMap::new()))) }
    pub fn extend(&self) -> Env {
        Env(Rc::new(RefCell::new((*self.0.borrow()).clone())))
    }
    pub fn set(&mut self, k: &str, v: Rc<Expr>) { self.0.borrow_mut().insert(k.into(), v); }
    pub fn get(&self, k: &str) -> Option<Rc<Expr>> { self.0.borrow().get(k).cloned() }
}

pub fn tokenize(s: &str) -> Vec<String> {
    s.replace("(", " ( ").replace(")", " ) ")
        .split_whitespace().map(|x| x.to_string()).collect()
}

pub fn parse(tokens: &mut Vec<String>) -> Expr {
    if tokens.is_empty() { panic!("unexpected EOF"); }
    let token = tokens.remove(0);
    match token.as_str() {
        "(" => {
            let mut list = vec![];
            while !tokens.is_empty() && tokens[0] != ")" { list.push(parse(tokens)); }
            if tokens.is_empty() { panic!("unexpected EOF while parsing list"); }
            tokens.remove(0);
            Expr::List(list)
        }
        ")" => panic!("unexpected )"),
        _ => {
            if let Ok(n) = token.parse::<i64>() { Expr::Num(n) }
            else if (token.starts_with('"') && token.ends_with('"')) || (token.starts_with("'") && token.ends_with("'")) {
                // Remove only the first and last quote, not all matching chars
                let chars: Vec<char> = token.chars().collect();
                if chars.len() >= 2 {
                    Expr::Str(chars[1..chars.len()-1].iter().collect())
                } else {
                    Expr::Str(String::new())
                }
            }
            else { Expr::Var(token) }
        }
    }
}

pub fn eval(expr: Rc<Expr>, env: &mut Env) -> Rc<Expr> {
    match &*expr {
        Expr::Num(_) | Expr::Lambda(_, _) | Expr::Builtin(_) | Expr::Str(_) => expr.clone(),
        Expr::Var(v) => {
            match env.get(v) {
                Some(val) => val,
                None => {
                    // If variable is a builtin, return builtin
                    if let Some(builtin) = env.get(v) {
                        builtin
                    } else {
                        Rc::new(Expr::Var(format!("خطا: متغیر تعریف نشده: {}", v)))
                    }
                }
            }
        },
        Expr::List(list) => {
            if list.is_empty() { return Rc::new(Expr::List(vec![])); }
            let head = &list[0];
            match head {
                Expr::Var(v) if v == "lambda" => {
                    if list.len() != 3 { panic!("bad lambda"); }
                    let args = match &list[1] {
                        Expr::List(v) => v.iter().map(|x| match x { Expr::Var(n) => n.clone(), _ => panic!() }).collect(),
                        _ => panic!()
                    };
                    Rc::new(Expr::Lambda(args, Rc::new(list[2].clone())))
                }
                Expr::Var(v) if v == "define" => {
                    let name = match &list[1] { Expr::Var(n) => n.clone(), _ => panic!() };
                    let val = if list.len() == 3 {
                        eval(Rc::new(list[2].clone()), env)
                    } else {
                        Rc::new(Expr::Num(0))
                    };
                    env.set(&name, val.clone());
                    val
                }
                Expr::Var(v) if v == "set!" => {
                    if list.len() != 3 {
                        return Rc::new(Expr::Var("خطا: set! باید دو آرگومان بگیرد".to_string()));
                    }
                    let name = match &list[1] { Expr::Var(n) => n.clone(), _ => return Rc::new(Expr::Var("خطا: نام متغیر معتبر نیست".to_string())) };
                    let val = eval(Rc::new(list[2].clone()), env);
                    env.set(&name, val.clone());
                    val
                }
                _ => {
                    let f = eval(Rc::new(list[0].clone()), env);
                    let args: Vec<Rc<Expr>> = list[1..].iter().map(|x| eval(Rc::new(x.clone()), env)).collect();
                    match &*f {
                        Expr::Builtin(b) => b(args, env),
                        Expr::Lambda(params, body) => {
                            let mut new_env = env.extend();
                            for (p,a) in params.iter().zip(args.iter()) {
                                new_env.set(p, a.clone());
                            }
                            for p in params.iter().skip(args.len()) {
                                new_env.set(p, Rc::new(Expr::Var("".to_string())));
                            }
                            eval(body.clone(), &mut new_env)
                        }
                        Expr::Num(n) if args.is_empty() => Rc::new(Expr::Num(*n)),
                        Expr::Str(s) if args.is_empty() => Rc::new(Expr::Str(s.clone())),
                        Expr::Var(s) if args.is_empty() => Rc::new(Expr::Var(s.clone())),
                        // If not callable, just return the value (for print and similar)
                        _ => f.clone()
                    }
                }
            }
        }
    }
}
