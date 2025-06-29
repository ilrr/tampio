use std::fs;

use clap::{Arg, ArgAction, Command};
use ledger::Ledger;
use parser::Parser;
use semantic::Semantic;

mod html;
mod htmll;
mod ledger;
mod lexer;
mod parser;
mod semantic;

fn main() {
    let matches = Command::new("Tampio")
        .version("0.1.0")
        .arg(Arg::new("inputs").required(true).action(ArgAction::Append))
        .arg(
            Arg::new("output")
                .short('o')
                .help("File to write html ledger into"),
        )
        .get_matches();

    let mut input_paths = matches.get_many::<String>("inputs").unwrap();

    let mut ledger;
    let path = input_paths.next().unwrap();
    if let Ok(s) = fs::read_to_string(path) {
        let mut parser = Parser::new(&s);
        ledger = Ledger::exec(Semantic::from_parse_tree(parser.parse()).statements);
    } else {
        println!("Tiedostoa '{path}' ei löydy.");
        return;
    }
    for path in input_paths {
        if let Ok(s) = fs::read_to_string(path) {
            ledger.add_comparison_from_str(&s);
        } else {
            println!("Tiedostoa '{path}' ei löydy.");
            return;
        }
    }

    if let Some(path) = matches.get_one::<String>("output") {
        if let Ok(_) = fs::write(path, ledger.html_string()) {
            println!("Kirjanpitoraportti luotu: {path}");
        } else {
            println!("Kirjanpitoraportin tallennus epäonnistui :-(");
        }
    } else {
        println!("{}", ledger.html_string());
    }
}
