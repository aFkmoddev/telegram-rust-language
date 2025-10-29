use teloxide::prelude::*;
use teloxide::types::Message;
use log;
use pretty_env_logger;
use crate::interpreter::*;
use crate::builtins::add_builtins;
use std::rc::Rc;
use std::fs;

pub fn run_code(code: &str) -> String {
    let result = std::panic::catch_unwind(|| {
        let mut tokens = tokenize(code);
        let parsed = parse(&mut tokens);
        let mut env = Env::new();
        add_builtins(&mut env);
        let res = eval(Rc::new(parsed), &mut env);
        match &*res {
            Expr::Num(n) => n.to_string(),
            Expr::Str(s) => s.clone(),
            Expr::Var(v) => v.clone(),
            Expr::List(l) => {
                if !l.is_empty() {
                    if let Some(last_str) = l.iter().rev().find_map(|e| if let Expr::Str(s) = e { Some(s.clone()) } else { None }) {
                        last_str
                    } else if let Some(last_num) = l.iter().rev().find_map(|e| if let Expr::Num(n) = e { Some(n.to_string()) } else { None }) {
                        last_num
                    } else if let Some(last_var) = l.iter().rev().find_map(|e| if let Expr::Var(v) = e { Some(v.clone()) } else { None }) {
                        last_var
                    } else {
                        format!("{:?}", l)
                    }
                } else {
                    String::new()
                }
            },
            _ => format!("{:?}", res),
        }
    });
    match result {
        Ok(res) => res,
        Err(e) => {
            // Print error location and message to console
            println!("[ERROR] در اجرای کد خطا رخ داد: {:?}", e);
            "ورودی یا کد نامعتبر!".to_string()
        }
    }
}

pub async fn telegram_bot() {
    pretty_env_logger::init();
    log::info!("Starting telegram bot...");
    let token = fs::read_to_string("bot_token.txt").expect("توکن تلگرام یافت نشد").trim().to_string();
    let bot = Bot::new(token);
    teloxide::repl(bot, |message: Message, bot: Bot| async move {
        if let Some(text) = message.text() {
            let trimmed = text.trim();
            if trimmed.starts_with('/') {
                match trimmed {
                    "/help" => {
                        let help = "راهنمای زبان:\n\nبرای اجرای کد باید پیام با code شروع شود:\nمثال:\ncode (+ 2 3)\ncode (print \"سلام\")\n\nعملگرها:\n+  جمع\n-  تفریق\n*  ضرب\n/  تقسیم\n%  باقیمانده\npow یا ** توان\n<  کوچکتر\n<= کوچکتر یا مساوی\n>  بزرگتر\n>= بزرگتر یا مساوی\n\nدستورات:\nprint چاپ\nset! مقداردهی متغیر\nif شرطی\nwhile حلقه\nlambda تابع\nbegin اجرای چند دستور\n\nرشته‌ها با \"متن\" یا 'متن' قابل استفاده‌اند.\nمثال:\ncode (print \"سلام دنیا\")\n";
                        bot.send_message(message.chat.id, help).await?;
                    },
                    "/start" => {
                        bot.send_message(message.chat.id, "سلام! برای راهنما /help را بزنید.").await?;
                    },
                    _ => {
                        bot.send_message(message.chat.id, "کامند ناشناخته. برای راهنما /help را بزنید.").await?;
                    }
                }
            } else if trimmed.starts_with("code") {
                let code = trimmed.trim_start_matches("code").trim();
                let output = run_code(code);
                let valid = !(output.starts_with("خطا:") || output.contains("متغیر تعریف نشده") || output.contains("undefined variable") || output.contains("ورودی یا کد نامعتبر") || output.trim().is_empty() || output == "[]");
                if valid {
                    bot.send_message(message.chat.id, format!("{}", output)).await?;
                } else {
                    println!("Error for user: {}", output);
                }
            }
            
        }
        respond(())
    }).await;
}
