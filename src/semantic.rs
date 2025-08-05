use core::panic;

use serde::Serialize;
use time::Date;

use crate::lexer::Token;
use crate::parser::Node;

#[derive(Debug)]
pub(crate) enum SStatement {
    Transaction {
        date: Option<Date>,
        description: String,
        entries: Vec<SEntry>,
        doc: Option<String>,
    },
    Block(SHeader, Vec<SStatement>),
    Expression(SExpression),
    Section(SectionType),
    Account(Option<i32>, String, Vec<SStatement>, AccountType),
    BudgetEntry {
        account: SAccount,
        amounts: Vec<(i32, EntryType)>,
    },
}

#[derive(Debug)]
pub(crate) enum EntryType {
    Debit,
    Credit,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub(crate) enum AccountType {
    Assets,
    LiabilitiesTopLevel,
    Liabilities,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SectionType {
    Ledger,
    AccountMap,
    Budget,
    Options,
}

#[derive(Debug)]
pub(crate) enum SHeader {
    Date { date: Date },
    AutoBalance { account: SAccount },
    Dummy,
}

#[derive(Debug)]
pub(crate) struct SEntry {
    pub account: SAccount,
    pub amounts: Vec<SAuto<i32>>,
}

#[derive(Debug)]
pub(crate) enum SAccount {
    N(i32),
    Alias(String),
}

#[derive(Debug)]
pub(crate) enum SAuto<T> {
    Val(T),
    Auto,
}

#[derive(Debug)]
pub(crate) enum SExpression {
    Alias(String, i32),
    Definition(String, String),
}

pub struct Semantic {
    pub statements: Vec<SStatement>,
    section: SectionType,
}

impl Semantic {
    fn new() -> Self {
        Self {
            statements: Vec::new(),
            section: SectionType::Ledger,
        }
    }

    pub fn from_parse_tree(parse_tree: Vec<Node>) -> Self {
        let mut instance = Self::new();
        instance.statements = instance.nodes(parse_tree);
        instance
    }

    fn nodes(&mut self, nodes: Vec<Node>) -> Vec<SStatement> {
        nodes.into_iter().map(|n| self.node(n)).collect()
    }

    fn node(&mut self, node: Node) -> SStatement {
        match node {
            Node::Block(_, _) => self.block(node),
            Node::List(l) => self.list(l),
        }
    }

    fn list(&mut self, list: Vec<Token>) -> SStatement {
        match &list[..] {
            [Token::Identifier(ident), Token::Assign, Token::Number(n)] => {
                SStatement::Expression(SExpression::Alias(ident.to_string(), *n))
            }
            [Token::Identifier(ident), Token::Assign, Token::String(s)] => {
                SStatement::Expression(SExpression::Definition(ident.to_string(), s.to_string()))
            }
            [Token::Section, Token::Identifier(ident)] => {
                let s_type = match ident.to_uppercase().as_str() {
                    "TILIKARTTA" => SectionType::AccountMap,
                    "TIEDOT" => SectionType::Options,
                    "TALOUSARVIO" | "BUDJETTI" => SectionType::Budget,
                    _ => SectionType::Ledger,
                };
                self.section = s_type;
                SStatement::Section(s_type)
            }
            [Token::Number(n), Token::String(s)] => {
                SStatement::Account(Some(*n / 100), s.to_string(), vec![], AccountType::None)
            }
            _ => panic!("malformed statement: {:?}", list),
        }
    }

    fn block(&mut self, block: Node) -> SStatement {
        match block {
            Node::Block(h, body) => match &h[..] {
                [Token::Date(date)] => self.date_block(*date, body),
                [Token::Date(date), Token::String(description)] => {
                    self.transaction(Some(*date), description.clone(), body, None)
                }
                [
                    Token::Identifier(d),
                    Token::Date(date),
                    Token::String(description),
                ]
                | [
                    Token::Date(date),
                    Token::String(description),
                    Token::Identifier(d),
                ] => self.transaction(Some(*date), description.clone(), body, Some(d.clone())),
                [Token::String(description)] => {
                    self.transaction(None, description.clone(), body, None)
                }
                [Token::Identifier(d), Token::String(description)]
                | [Token::String(description), Token::Identifier(d)] => {
                    self.transaction(None, description.clone(), body, Some(d.clone()))
                }
                [Token::Auto, tail @ ..] => self.auto_block(tail, body),
                [] => self.dummy_block(body),
                [Token::Number(n), Token::String(s)] => {
                    self.account(Some(*n), s.clone(), body, AccountType::None)
                }
                [Token::Plus, Token::String(s)] => {
                    self.account(None, s.clone(), body, AccountType::Assets)
                }
                [Token::Minus, Token::String(s)] => {
                    self.account(None, s.clone(), body, AccountType::LiabilitiesTopLevel)
                }
                [Token::Number(n)] => self.budget_row(SAccount::N(*n), body),
                [Token::Identifier(i)] => self.budget_row(SAccount::Alias(i.into()), body),
                _ => panic!("unknown header {:?}", h),
            },
            Node::List(l) => match &l[..] {
                [Token::Identifier(ident), Token::Assign, Token::Number(n)] => {
                    SStatement::Expression(SExpression::Alias(ident.to_string(), *n))
                }
                [Token::Identifier(ident), Token::Assign, Token::String(s)] => {
                    SStatement::Expression(SExpression::Definition(
                        ident.to_string(),
                        s.to_string(),
                    ))
                }
                _ => panic!("unknown expression"),
            },
        }
    }

    fn date_block(&mut self, date: Date, body: Vec<Node>) -> SStatement {
        SStatement::Block(SHeader::Date { date }, self.nodes(body))
    }

    fn dummy_block(&mut self, body: Vec<Node>) -> SStatement {
        SStatement::Block(SHeader::Dummy, self.nodes(body))
    }

    fn budget_row(&mut self, account: SAccount, body: Vec<Node>) -> SStatement {
        // if body.len() != 1 {
        //     panic!("too long body")
        // };
        let mut amounts = vec![];
        for entry in body {
            if let Node::List(amount) = entry {
                amounts.push(match &amount[..] {
                    [Token::Number(n)] => (*n, EntryType::None),
                    [Token::Minus, Token::Number(n)] => (-n, EntryType::None),
                    [Token::Debit, Token::Number(n)] | [Token::Number(n), Token::Debit] => {
                        (*n, EntryType::Debit)
                    }
                    [Token::Credit, Token::Number(n)] | [Token::Number(n), Token::Credit] => {
                        (*n, EntryType::Credit)
                    }
                    _ => panic!("should be a number"),
                })
            } else {
                panic!("no blocks here")
            }
        }
        SStatement::BudgetEntry {
            account: account,
            amounts,
        }
    }

    fn account(
        &mut self,
        n: Option<i32>,
        name: String,
        sub_accounts: Vec<Node>,
        account_type: AccountType,
    ) -> SStatement {
        let subs = self.nodes(sub_accounts);
        SStatement::Account(n.map(|num| num / 100), name, subs, account_type)
    }

    fn transaction(
        &mut self,
        date: Option<Date>,
        description: String,
        body: Vec<Node>,
        doc: Option<String>,
    ) -> SStatement {
        if let SectionType::AccountMap = self.section {
            SStatement::Account(None, description, self.nodes(body), AccountType::None)
        } else {
            SStatement::Transaction {
                date,
                description,
                entries: self.entries(body),
                doc,
            }
        }
    }

    fn auto_block(&mut self, tail: &[Token], body: Vec<Node>) -> SStatement {
        match tail {
            [Token::Number(n)] => SStatement::Block(
                SHeader::AutoBalance {
                    account: SAccount::N(*n),
                },
                self.nodes(body),
            ),
            [Token::Identifier(ident)] => SStatement::Block(
                SHeader::AutoBalance {
                    account: SAccount::Alias(ident.clone()),
                },
                self.nodes(body),
            ),
            _ => panic!(),
        }
    }

    fn entries(&mut self, body: Vec<Node>) -> Vec<SEntry> {
        let mut result = Vec::new();
        for e in body {
            if let Node::Block(h, b) = e {
                let account = match &h[..] {
                    [Token::Identifier(ident)] => SAccount::Alias(ident.to_string()),
                    [Token::Number(n)] => SAccount::N(*n),
                    _ => panic!("invalid account: {:?}", h),
                };
                let amounts = self.amounts(b);
                result.push(SEntry { account, amounts });
            } else {
                panic!("expected a block")
            }
        }
        result
    }

    fn amounts(&mut self, amounts: Vec<Node>) -> Vec<SAuto<i32>> {
        let mut result = Vec::new();
        for a in amounts {
            if let Node::List(l) = a {
                match &l[..] {
                    [Token::Number(n)]
                    | [Token::Debit, Token::Number(n)]
                    | [Token::Number(n), Token::Debit] => result.push(SAuto::Val(*n)),
                    [Token::Minus, Token::Number(n)]
                    | [Token::Credit, Token::Number(n)]
                    | [Token::Number(n), Token::Credit] => result.push(SAuto::Val(-n)),
                    [Token::Auto] => result.push(SAuto::Auto),
                    _ => panic!("expected some $$$"),
                }
            } else {
                panic!("expected a list")
            }
        }
        result
    }
}
