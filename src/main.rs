use std::fs;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use clap::{Arg, ArgAction, Command};
use ledger::Ledger;
use parser::Parser;
use semantic::Semantic;

use crate::ledger::LedgerType;

mod html;
mod htmll;
mod ledger;
mod lexer;
mod parser;
mod semantic;

#[tokio::main]
async fn main() {
    let matches = Command::new("Tampio")
        .version("0.1.2")
        .arg(Arg::new("inputs").required(true).action(ArgAction::Append))
        .arg(
            Arg::new("output")
                .short('o')
                .help("File to write html ledger into"),
        )
        .arg(
            Arg::new("budgeting_html")
                .short('b')
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("budget_file").short('B'))
        .get_matches();

    let mut input_paths = matches.get_many::<String>("inputs").unwrap();
    let input_paths_2 = input_paths.clone().map(|p| format!("{p}")).collect();

    let mut ledger;
    let mut budgeting_file_exists = false;
    let path = if let Some(path) = matches.get_one("budget_file") {
        match fs::exists(path) {
            Ok(true) => {
                budgeting_file_exists = true;
                path
            }
            _ => input_paths.next().unwrap(),
        }
    } else {
        input_paths.next().unwrap()
    };
    if let Ok(s) = fs::read_to_string(path) {
        let mut parser = Parser::new(&s);
        ledger = Ledger::exec(Semantic::from_parse_tree(parser.parse()).statements);
        if budgeting_file_exists {
            ledger.ledger_type = LedgerType::Budgeting;
        }
    } else {
        eprintln!("Tiedostoa '{path}' ei löydy.");
        return;
    }
    for path in input_paths {
        if let Ok(s) = fs::read_to_string(path) {
            ledger.add_comparison_from_str(&s);
        } else {
            eprintln!("Tiedostoa '{path}' ei löydy.");
            return;
        }
    }

    if let Some(path) = matches.get_one::<String>("output") {
        let res = fs::write(
            path,
            if matches.get_flag("budgeting_html") {
                ledger.html_string_with_budgeting(htmll::Budgeting::File)
            } else {
                ledger.html_string()
            },
        );
        if let Ok(_) = res {
            eprintln!("Kirjanpitoraportti luotu: {path}");
        } else {
            eprintln!("Kirjanpitoraportin tallennus epäonnistui :-(");
        }
    } else if let Some(path) = matches.get_one::<String>("budget_file") {
        let path = path.clone();
        let p2 = path.clone();
        let account_map = ledger.account_map_string();
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    Into::<axum::response::Html<String>>::into(generate_budgeting_html(
                        p2,
                        input_paths_2,
                    ))
                }),
            )
            .route(
                "/save_budget",
                post(|b: Request<Body>| async move {
                    let b = to_bytes(b.into_body(), usize::MAX).await.unwrap();
                    let file_content = format!(
                        "§ TILIKARTTA\n{}\n\n{}",
                        account_map,
                        String::from_utf8_lossy(&b)
                    );
                    if let Ok(_) = fs::write(path.clone(), file_content) {
                        "OK".into_response()
                    } else {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Couldn't write to {}", path.clone()),
                        )
                            .into_response()
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3995")
            .await
            .unwrap();
        eprintln!("http://localhost:3995/");
        axum::serve(listener, app).await.unwrap();
    } else {
        println!("{}", ledger.html_string());
    }
}

fn generate_budgeting_html(budget_path: String, comparison_paths: Vec<String>) -> String {
    if let Ok(s) = fs::read_to_string(budget_path) {
        let mut ledger = Ledger::from_string(s);
        ledger.ledger_type = LedgerType::Budgeting;
        for path in comparison_paths {
            if let Ok(s) = fs::read_to_string(&path) {
                ledger.add_comparison_from_str(&s);
            } else {
                eprintln!("Tiedostoa '{path}' ei löydy.");
                return format!("Tiedostoa '{path}' ei löydy.");
            }
        }
        return ledger.html_string_with_budgeting(htmll::Budgeting::Server);
    } else {
        let path = &comparison_paths[0]; 
        let mut ledger;
        if let Ok(s) = fs::read_to_string(path) {
            let mut parser = Parser::new(&s);
            ledger = Ledger::exec(Semantic::from_parse_tree(parser.parse()).statements);
        } else {
            eprintln!("Tiedostoa '{path}' ei löydy.");
            return format!("Tiedostoa '{path}' ei löydy.");
        }
        for path in comparison_paths.iter().skip(1) {
            if let Ok(s) = fs::read_to_string(path) {
                ledger.add_comparison_from_str(&s);
            } else {
                eprintln!("Tiedostoa '{path}' ei löydy.");
                return format!("Tiedostoa '{path}' ei löydy.");
            }
        }
        ledger.html_string_with_budgeting(htmll::Budgeting::Server)
    }
}
