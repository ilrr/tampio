use itertools::Itertools;
use serde::{Serialize, ser::SerializeStruct};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use time::{Date, macros::format_description};

use crate::{
    parser::Parser,
    semantic::{
        AccountType, EntryType, SAccount, SAuto, SEntry, SExpression, SHeader, SStatement,
        SectionType, Semantic,
    },
};

#[derive(Clone, Debug)]
pub struct Transaction {
    pub date: Date,
    pub description: String,
    pub entries: Vec<(i32, i32)>,
    pub n: i32,
    pub doc: String,
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut t = serializer.serialize_struct("Transaction", 4)?;
        t.serialize_field("n", &self.n)?;
        t.serialize_field("date", &format!("{}", self.date))?;
        t.serialize_field("description", &self.description)?;
        t.serialize_field("entries", &self.entries)?;
        t.end()
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date)
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.date.cmp(&other.date))
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.date.eq(&other.date)
    }
}

impl Eq for Transaction {}

impl Transaction {
    pub fn fmt_date(self) -> String {
        let date_format =
            format_description!("[day padding:none].[month padding:none].[year padding:none]");
        self.date.format(&date_format).unwrap()
    }
}

impl AccTransaction {
    pub fn fmt_date(self) -> String {
        let date_format =
            format_description!("[day padding:none].[month padding:none].[year padding:none]");
        self.date.format(&date_format).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct AccTransaction {
    pub(crate) n: i32,
    pub(crate) date: Date,
    pub(crate) description: String,
    pub(crate) amount: i32,
    pub(crate) doc: String,
}

impl Serialize for AccTransaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut t = serializer.serialize_struct("AccTransaction", 4)?;
        t.serialize_field("n", &self.n)?;
        t.serialize_field("date", &format!("{}", self.date))?;
        t.serialize_field("description", &self.description)?;
        t.serialize_field("amount", &self.amount)?;
        t.end()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub n: Option<i32>,
    pub name: String,
    pub(crate) sub_accounts: Vec<Rc<RefCell<Account>>>,
    pub credits: Vec<i32>,
    pub debits: Vec<i32>,
    pub rec_credits: Vec<i32>,
    pub rec_debits: Vec<i32>,
    pub transactions: Vec<AccTransaction>,
    pub t: AccountType,
}

impl Account {
    fn new(n: Option<i32>, name: String, t: AccountType) -> Rc<RefCell<Account>> {
        Rc::new(RefCell::new(Self {
            n,
            name,
            t,
            sub_accounts: Vec::new(),
            credits: vec![0],
            debits: vec![0],
            rec_credits: vec![0],
            rec_debits: vec![0],
            transactions: Vec::new(),
        }))
    }

    pub(crate) fn naked(n: Option<i32>, name: String, t: AccountType) -> Account {
        Self {
            n: n,
            name: name,
            sub_accounts: vec![],
            credits: vec![],
            debits: vec![],
            rec_credits: vec![],
            rec_debits: vec![],
            transactions: vec![],
            t: t,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.sub_accounts.is_empty()
    }

    fn add_ledger(&mut self) {
        self.credits.push(0);
        self.debits.push(0);
        self.rec_credits.push(0);
        self.rec_debits.push(0);
        for child in &self.sub_accounts {
            let mut child = child.borrow_mut();
            child.add_ledger();
        }
    }

    fn add_amount(&mut self, amount: i32, li: usize) {
        if amount > 0 {
            self.debits[li] += amount;
        } else {
            self.credits[li] -= amount;
        }
    }

    fn calc_rec_sum(&mut self, li: usize) -> (i32, i32) {
        self.rec_credits[li] = self.credits[li];
        self.rec_debits[li] = self.debits[li];
        for child in &self.sub_accounts {
            let (child_d, child_c) = child.borrow_mut().calc_rec_sum(li);
            self.rec_credits[li] += child_c;
            self.rec_debits[li] += child_d;
        }
        (self.rec_debits[li], self.rec_credits[li])
    }

    fn add_transaction(&mut self, transaction: Transaction) {
        let t_n = transaction.n;
        let t_date = transaction.date;
        let t_desc = transaction.description;
        if let Some(n) = self.n {
            for entry in transaction.entries.iter().filter(|t| t.0 == n) {
                self.transactions.push(AccTransaction {
                    n: t_n,
                    date: t_date,
                    description: t_desc.clone(),
                    amount: entry.1,
                    doc: transaction.doc.clone(),
                })
            }
        }
    }

    pub fn as_string(&self,top_level:bool, indent_level: usize) -> String {
        let sub_account_strings = if self.is_leaf() {
            "".to_string()
        } else {
            format!("\n{}",self.sub_accounts.iter().map(|a| a.borrow().as_string(false, indent_level+2)).join("\n"))
        };
        let number = if let Some(n) = self.n {
            format!("{n} ")
        } else {
            "".to_string()
        };
        let prefix = if !top_level {""} else { match self.t {
            AccountType::None | AccountType::Liabilities => "",
            AccountType::Assets => "+ ",
            AccountType::LiabilitiesTopLevel => "- "
        }};
        let name = self.name.clone();
        let name = if !name.contains("\"") {
            format!("\"{name}\"")
        } else if !name.contains("'") {
            format!("'{name}'")
        } else {
            format!("»{name}»")
        };
        let indent = " ".repeat(indent_level);
        format!("{indent}{prefix}{number}{name}{sub_account_strings}")
    }
}

pub struct Ledger {
    pub ledger: Vec<Transaction>,
    scopes: Vec<Scope>,
    pub year: i32,
    pub accounts: Vec<Rc<RefCell<Account>>>,
    pub account_dict: HashMap<i32, Rc<RefCell<Account>>>,
    section: SectionType,
    pub options: Vec<HashMap<String, String>>,
    pub ledger_type: LedgerType,
    pub comp_ledger_types: Vec<LedgerType>,
    pub(crate) l_index: usize,
    doc_d: HashMap<String, i32>,
}

#[derive(Default)]
struct Scope {
    date: Option<Date>,
    auto_balance: Option<i32>,
    aliases: HashMap<String, i32>,
}

#[derive(Serialize)]
pub struct LedgerOut {
    pub ledger: Vec<Transaction>,
    pub accounts: Vec<Account>,
    pub general_ledger: Vec<Vec<Transaction>>,
}

impl Scope {
    pub fn add_hash(&mut self, k: String, v: i32) {
        self.aliases.insert(k, v);
    }
}

pub trait ScopeStack<T> {
    fn collapsed(&self) -> T;
}

impl ScopeStack<Scope> for Vec<Scope> {
    fn collapsed(&self) -> Scope {
        let mut date = None;
        let mut auto_balance = None;
        let mut aliases = HashMap::new();
        for s in self {
            if let Some(d) = s.date {
                date = Some(d);
            }
            if let Some(a_b) = s.auto_balance {
                auto_balance = Some(a_b);
            }
            aliases.extend(s.aliases.clone());
        }
        Scope {
            date,
            auto_balance,
            aliases,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LedgerType {
    Main,
    Budget,
    Budgeting,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            ledger: Vec::new(),
            scopes: vec![Scope {
                ..Default::default()
            }],
            year: 0,
            section: SectionType::Ledger,
            accounts: Vec::new(),
            account_dict: HashMap::new(),
            options: vec![HashMap::new()],
            ledger_type: LedgerType::Main,
            comp_ledger_types: vec![LedgerType::Main],
            l_index: 0,
            doc_d: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn ledger(&self) -> Vec<Transaction> {
        self.ledger.clone()
    }

    pub fn add_comparison_from_str(&mut self, s: &str) {
        let lt = self.ledger_type;
        let mut parser = Parser::new(s);
        let statements = Semantic::from_parse_tree(parser.parse()).statements;
        if statements.iter().any(|x| match x {
            SStatement::Section(SectionType::Budget) => true,
            _ => false
        }) {
            self.comp_ledger_types.push(LedgerType::Budget);
        } else {
            self.comp_ledger_types.push(LedgerType::Main);
        }
        self.l_index += 1;
        self.options.push(HashMap::new());
        for account in &self.accounts {
            let mut account = account.borrow_mut();
            account.add_ledger();
        }
        self.exec_statements(statements);
        self.calculate_sums();
        self.ledger_type = lt;
    }

    pub fn accounts(&self) -> Vec<Account> {
        self.accounts.iter().map(|rc| rc.borrow().clone()).collect()
    }

    pub fn account_map_string(&self) -> String {
        self.accounts.iter().map(|a| a.borrow().as_string(true,0)).join("\n")
    }

    #[allow(dead_code)]
    pub fn print_accounts(&self) {
        println!("{:?}", self.accounts());
        println!("{:?}", self.account_dict);
    }

    #[allow(dead_code)]
    pub fn json(&self) -> String {
        serde_json::to_string(&LedgerOut {
            ledger: self.ledger(),
            accounts: self.accounts(),
            general_ledger: Vec::new(),
        })
        .unwrap()
    }

    pub fn exec(statements: Vec<SStatement>) -> Self {
        let mut instance = Ledger::new();
        instance.exec_statements(statements);
        instance.complete_multi_docs();
        instance.sort_ledger();
        instance.calculate_sums();
        instance.populate_account_transactions();
        instance
    }

    pub fn from_string(source: String) -> Self {
        let mut parser = Parser::new(&source);
        Self::exec(Semantic::from_parse_tree(parser.parse()).statements)
    }

    pub fn get_account(&self, account_n: i32) -> Option<Account> {
        self.account_dict
            .get(&account_n)
            .map(|acc| acc.borrow().clone())
    }

    fn populate_account_transactions(&mut self) {
        for transaction in &self.ledger {
            let mut v = HashSet::new();
            for entry in &transaction.entries {
                if v.insert((transaction.n, entry.0)) {
                    self.account_dict
                        .get(&entry.0)
                        .expect("invalid account number")
                        .borrow_mut()
                        .add_transaction(transaction.clone());
                }
            }
        }
    }

    fn calculate_sums(&mut self) {
        for account in &self.accounts {
            account.borrow_mut().calc_rec_sum(self.l_index);
        }
    }

    fn complete_multi_docs(&mut self) {
        for (k, _) in self.doc_d.iter().filter(|(_, v)| **v > 1) {
            self.ledger.iter_mut().find(|t| t.doc == *k).unwrap().doc = format!("{k}:1");
        }
    }

    fn sort_ledger(&mut self) {
        self.ledger.sort();
        let mut doc_n = 0;
        for (i, t) in self.ledger.iter_mut().enumerate() {
            t.n = i as i32;
            if t.doc.is_empty() {
                t.doc = doc_n.to_string();
                doc_n += 1;
            }
        }
    }

    fn exec_statements(&mut self, statements: Vec<SStatement>) {
        for statement in statements {
            self.exec_s(statement);
        }
    }

    fn exec_s(&mut self, statement: SStatement) {
        match statement {
            SStatement::Block(header, body) => self.exec_block(header, body),
            SStatement::Transaction {
                date,
                description,
                entries,
                doc,
            } => self.exec_transaction(date, description, entries, doc, false),
            SStatement::Expression(expr) => self.exec_expression(expr),
            SStatement::Section(section) => self.section = section,
            SStatement::Account(n, name, subs, acc_type) => {
                if self.l_index == 0 {
                    self.exec_account(n, name, subs, acc_type, None)
                }
            }
            SStatement::BudgetEntry { account, amounts } => self.exec_transaction(
                None,
                "".into(),
                vec![SEntry {
                    account,
                    amounts: amounts
                        .iter()
                        .map(|(a, t)| match t {
                            EntryType::None | EntryType::Credit => SAuto::Val(-a),
                            EntryType::Debit => SAuto::Val(*a),
                        })
                        .collect(), // amounts: vec![SAuto::Val(amount)],
                }],
                None,
                true,
            ),
        }
    }

    fn exec_account(
        &mut self,
        n: Option<i32>,
        name: String,
        sub_accounts: Vec<SStatement>,
        acc_type: AccountType,
        parent: Option<Rc<RefCell<Account>>>,
    ) {
        let account = Account::new(n, name, acc_type.clone());
        for sub in sub_accounts {
            if let SStatement::Account(sn, sname, ssubs, _) = sub {
                let child_type = match acc_type {
                    AccountType::LiabilitiesTopLevel => AccountType::Liabilities,
                    _ => acc_type,
                };
                self.exec_account(sn, sname, ssubs, child_type, Some(Rc::clone(&account)));
            }
        }

        if let Some(num) = n {
            self.account_dict.insert(num, Rc::clone(&account));
        }

        match parent {
            Some(p) => p.borrow_mut().sub_accounts.push(account),
            None => self.accounts.push(account),
        }
    }

    fn exec_expression(&mut self, expr: SExpression) {
        match expr {
            SExpression::Alias(ident, n) => {
                self.scopes
                    .last_mut()
                    .expect("scope stack empty")
                    .add_hash(ident, n);
            }
            SExpression::Definition(i, d) => {
                if self.section == SectionType::Options {
                    self.options[self.l_index].insert(i.to_lowercase(), d);
                }
            }
        }
    }

    fn exec_block(&mut self, header: SHeader, body: Vec<SStatement>) {
        match header {
            SHeader::Date { date } => {
                self.scopes.push(Scope {
                    date: Some(date),
                    ..Default::default()
                });
                self.exec_statements(body);
                self.scopes.pop();
            }
            SHeader::AutoBalance { account } => match account {
                SAccount::N(n) => {
                    self.scopes.push(Scope {
                        auto_balance: Some(n),
                        ..Default::default()
                    });
                    self.exec_statements(body);
                    self.scopes.pop();
                }
                SAccount::Alias(alias) => {
                    if let Some(n) = self.scopes.collapsed().aliases.get(&alias) {
                        self.scopes.push(Scope {
                            auto_balance: Some(*n),
                            ..Default::default()
                        });
                        self.exec_statements(body);
                        self.scopes.pop();
                    } else {
                        panic!("undefined alias")
                    }
                }
            },
            SHeader::Dummy => {
                self.scopes.push(Scope::default());
                self.exec_statements(body);
                self.scopes.pop();
            }
        }
    }

    fn exec_transaction(
        &mut self,
        date: Option<Date>,
        description: String,
        entries: Vec<SEntry>,
        doc: Option<String>,
        budget: bool,
    ) {
        let scope = self.scopes.collapsed();
        let resolved_date = if budget {
            self.ledger_type = LedgerType::Budget;
            Date::MIN
        } else {
            let mut resolved_date =
                date.unwrap_or_else(|| scope.date.expect("no date for transaction"));
            let year = resolved_date.year();
            if year == 0 {
                resolved_date = resolved_date.replace_year(self.year).unwrap();
            } else if year < 100 {
                resolved_date = resolved_date.replace_year(year + 2000).unwrap();
            } else {
                self.year = year;
            }
            resolved_date
        };
        let mut resolved_entries = Vec::new();
        let mut balance = 0;
        let mut auto_index = None;

        for entry in &entries {
            let account_number = match &entry.account {
                SAccount::N(n) => *n / 100,
                SAccount::Alias(s) => *scope.aliases.get(s).expect("alias not found"),
            };

            for amount in &entry.amounts {
                match amount {
                    SAuto::Val(n) => {
                        // let n = if budget { -*n } else { *n };
                        let n = *n;
                        resolved_entries.push((account_number, n));
                        balance += n;
                    }
                    SAuto::Auto => {
                        resolved_entries.push((account_number, 0));
                        auto_index = Some(resolved_entries.len() - 1);
                    }
                }
            }
        }

        if let Some(i) = auto_index {
            resolved_entries[i].1 = -balance;
        } else if balance != 0 {
            if let Some(ab) = scope.auto_balance {
                resolved_entries.push((ab / 100, -balance));
            } else if !budget {
                panic!("unbalanced transaction: \"{description}\"");
            }
        }

        for (n, amt) in &resolved_entries {
            let acc = self
                .account_dict
                .get(n)
                .unwrap_or_else(|| panic!("account {} not defined", n));
            acc.borrow_mut().add_amount(*amt, self.l_index);
        }

        if self.l_index == 0 {
            let doc = if let Some(doc) = doc {
                match self.doc_d.get_mut(&doc) {
                    None => {
                        self.doc_d.insert(doc.clone(), 1);
                        doc
                    }
                    Some(v) => {
                        *v += 1;
                        format!("{doc}:{v}")
                    }
                }
            } else {
                "".into()
            };

            self.ledger.push(Transaction {
                date: resolved_date,
                description,
                entries: resolved_entries,
                n: 0,
                doc: doc,
            });
        }
    }
}
