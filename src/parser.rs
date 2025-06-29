use core::panic;
use std::str;

use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub enum Node {
    Block(Vec<Token>, Vec<Node>),
    List(Vec<Token>),
}

impl Node {
    pub fn push_child(&mut self, c: Node) {
        match self {
            Self::Block(_, b) => b.push(c),
            Self::List(_) => panic!("can't push to list"),
        }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
        }
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut node_stack: Vec<Node> = Vec::new();
        let mut header: Vec<Token> = Vec::new();
        let mut result: Vec<Node> = Vec::new();

        while let Some(t) = self.lexer.next_token() {
            match t.normalise() {
                Token::BlockStart(_c) => {
                    node_stack.push(Node::Block(header.clone(), Vec::new()));
                    header.clear();
                }
                Token::BlockEnd(_c) => {
                    if !header.is_empty() {
                        node_stack
                            .last_mut()
                            .unwrap()
                            .push_child(Node::List(header.clone()));
                        header.clear();
                    }
                    let top = node_stack.pop().unwrap();
                    if node_stack.is_empty() {
                        result.push(top);
                    } else {
                        node_stack.last_mut().unwrap().push_child(top);
                    }
                }
                Token::Semicolon => {
                    if !header.is_empty() {
                        if let Some(top) = node_stack.last_mut() {
                            top.push_child(Node::List(header.clone()));
                        } else {
                            result.push(Node::List(header.clone()));
                        }
                        header.clear();
                    }
                }
                Token::EOF => {
                    if !header.is_empty() {
                        result.push(Node::List(header.clone()));
                    }
                    return result;
                }
                token => header.push(token),
            }
        }
        result
    }
}
