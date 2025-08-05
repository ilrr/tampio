use std::iter::zip;

use itertools::Itertools;

use crate::{
    html::Html,
    ledger::{Account, Ledger, LedgerType, Transaction},
    semantic::AccountType,
};

pub(crate) enum Budgeting {
    No,
    File,
    Server,
}

#[allow(dead_code)]
impl Ledger {
    pub fn html_string(&self) -> String {
        self.html(Budgeting::No).render()
    }

    pub fn html_string_with_budgeting(&self, budgeting: Budgeting) -> String {
        self.html(budgeting).render()
    }

    pub fn pretty_html_string(&self) -> String {
        self.html(Budgeting::No).pretty()
    }

    fn html(&self, budgeting: Budgeting) -> Html {
        let mut root = Html::new("html").with_attribute("lang", "fi");

        let mut body = Html::new("body");

        let is_budgeting = match budgeting {
            Budgeting::No => false,
            _ => true,
        };

        if !(self.ledger_type == LedgerType::Budget || is_budgeting) {
            body.push_child(
                Html::new("section")
                    .with_attribute("id", "päiväkirja")
                    .with_child(Html::new("h2").with_text("Päiväkirja"))
                    .with_child(self.html_diary()),
            );

            body.push_child(
                Html::new("section")
                    .with_attribute("id", "pääkirja")
                    .with_child(Html::new("h2").with_text("Pääkirja"))
                    .with_child(self.html_general_ledger()),
            );

            body.push_child(
                Html::new("section")
                    .with_attribute("id", "tase")
                    .with_child(Html::new("h2").with_text("Tase"))
                    .with_child(
                        Html::new_void("input")
                            .with_attribute("class", "hide-empty")
                            .with_attribute("type", "checkbox"),
                    )
                    .with_child(
                        Html::new_void("input")
                            .with_attribute("class", "hide-one-child-footers")
                            .with_attribute("type", "checkbox"),
                    )
                    .with_child(self.html_balance_sheet()),
            );
        }
        body.push_child(
            Html::new("section")
                .with_attribute("id", "tuloslaskelma")
                .with_child(
                    Html::new("h2").with_text(if self.ledger_type == LedgerType::Budget {
                        "Talousarvio"
                    } else {
                        "Tuloslaskelma"
                    }),
                )
                .with_child(
                    Html::new_void("input")
                        .with_attribute("class", "hide-empty")
                        .with_attribute("type", "checkbox"),
                )
                .with_child(
                    Html::new_void("input")
                        .with_attribute("class", "hide-one-child-footers")
                        .with_attribute("type", "checkbox"),
                )
                .with_child(self.html_income_statement(is_budgeting)),
        );

        match budgeting {
            Budgeting::File => {
                body.push_child(
                    Html::div_with_class("budget-output-container hidden")
                        .with_attribute("id", "budget-output-container")
                        .with_child(Html::new("textarea").with_attribute("id", "budget-output"))
                        .with_child(
                            Html::new("button")
                                .with_text("Piilota")
                                .with_attribute("onclick", "hideOutput()"),
                        ),
                );

                body.push_child(
                    Html::new("button")
                        .with_text("Näytä")
                        .with_attribute("id", "display-budget-output")
                        .with_attribute("onclick", "displayOutput()"),
                );
            }
            Budgeting::Server => {
                body.push_child(
                    Html::new("button")
                        .with_text("Tallenna")
                        .with_attribute("id", "save-budget-output")
                        .with_attribute("onclick", "saveBudget()"),
                );
            }
            _ => {}
        }

        root.push_child(self.head(budgeting));
        root.push_child(body);

        Html::document()
            .with_raw("<!DOCTYPE html>")
            .with_child(root)
    }

    fn html_diary(&self) -> Html {
        let mut diary = Html::div_with_class("diary");

        let mut header = Html::div_with_class("header").with_attribute("id", "diary-header");
        header.push_child_div_with_class_and_text("account-info", "".into());
        header.push_child_div_with_class_and_text("debit", "debet".into());
        header.push_child_div_with_class_and_text("credit", "kredit".into());
        diary.push_child(header);

        let transactions = &self.ledger;

        for transaction in transactions {
            diary.push_child(self.html_transaction(transaction.clone()));
        }
        diary
    }

    fn html_general_ledger(&self) -> Html {
        let mut general_ledger = Html::div_with_class("general-ledger");

        let mut header =
            Html::div_with_class("header").with_attribute("id", "general-ledger-header");
        header.push_child_div_with_class_and_text("account-info", "".into());
        header.push_child_div_with_class_and_text("debit", "debet".into());
        header.push_child_div_with_class_and_text("credit", "kredit".into());
        header.push_child_div_with_class_and_text("sum", "summa".into());
        general_ledger.push_child(header);
        general_ledger.push_child_div_with_class_and_text("line", "".into());

        let account_ns = self.account_dict.keys().sorted();
        for account_n in account_ns {
            if let Some(account) = self.get_account(*account_n) {
                if !account.transactions.is_empty() {
                    let mut account_elem = Html::div_with_class("account");
                    account_elem
                        .push_attribute("id", format!("gl-{}", account.n.unwrap()).as_str());
                    let mut header = Html::div_with_class("header");
                    header.push_child(
                        Html::div_with_class("account-info")
                            .with_child(
                                Html::new("a")
                                    .with_class("n")
                                    .with_attribute("href", format!("#a-{account_n}").as_str())
                                    .with_string(format!("{account_n}")),
                            )
                            .with_child(Html::div_with_class_and_text("name", account.name)),
                    );
                    account_elem.push_child(header);
                    let mut entries = Html::div_with_class("entries");
                    let mut debit_sum = 0;
                    let mut credit_sum = 0;
                    let sum_multiplyer = if account.t == AccountType::Assets {
                        1
                    } else {
                        -1
                    };
                    for transaction in account.transactions {
                        let date = transaction.clone().fmt_date();
                        let doc = transaction.doc;
                        let desc = transaction.description;
                        let amount = transaction.amount;
                        if amount > 0 {
                            debit_sum += amount;
                        } else {
                            credit_sum += amount;
                        }
                        let mut entry = Html::div_with_class("entry");
                        entry.push_attribute(
                            "id",
                            format!("gl-{}-{}", account.n.unwrap(), doc).as_str(),
                        );
                        entry.push_child(
                            Html::div_with_class("doc").with_child(
                                Html::new("a")
                                    .with_attribute("href", format!("#d-{}", doc.clone()).as_str())
                                    .with_string(doc),
                            ),
                        );
                        entry.push_child_div_with_class_and_text("date", date);
                        entry.push_child_div_with_class_and_text("description", desc);
                        entry.push_child_div_with_class_and_text(
                            "debit amount",
                            Self::debit(amount),
                        );
                        entry.push_child_div_with_class_and_text(
                            "credit amount",
                            Self::credit(amount),
                        );
                        entry.push_child_div_with_class_and_text(
                            "saldo amount",
                            Self::amount_as_string(sum_multiplyer * (debit_sum + credit_sum), true),
                        );
                        entries.push_child(entry);
                    }
                    let mut sums = Html::div_with_class("sums");
                    sums.push_child_div_with_class_and_text(
                        "debit amount",
                        Self::amount_as_string(debit_sum, true),
                    );
                    sums.push_child_div_with_class_and_text(
                        "credit amount",
                        Self::amount_as_string(-credit_sum, true),
                    );
                    sums.push_child_div_with_class_and_text(
                        "sum amount",
                        Self::amount_as_string(sum_multiplyer * (debit_sum + credit_sum), true),
                    );
                    account_elem.push_child(entries);
                    account_elem.push_child(sums);
                    general_ledger.push_child(account_elem);
                }
            }
        }
        general_ledger
    }

    fn html_balance_sheet(&self) -> Html {
        let mut balance_sheet = Html::div_with_class("balance-sheet");

        let fiscal_years = self
            .options
            .iter()
            .map(|o| o.get("lyhenne").map_or("".into(), |s| s.clone()))
            .rev();
        let mut fy_elem = Html::div_with_class("fiscal-years");
        for fiscal_year in fiscal_years {
            fy_elem.push_child(
                Html::new("div")
                    .with_class("fiscal-year")
                    .with_text(&fiscal_year),
            );
        }
        balance_sheet.push_child(Html::div_with_class("table-header").with_child(fy_elem));

        let accounts = self
            .accounts()
            .into_iter()
            .filter(|a| a.t == AccountType::Assets || a.t == AccountType::LiabilitiesTopLevel);
        for account in accounts {
            balance_sheet.push_child(self.html_account_row(account.clone(), false));
        }

        balance_sheet
    }

    fn html_income_statement(&self, include_budgeting_cells: bool) -> Html {
        let mut income_statement = Html::div_with_class("income-statement");

        let fiscal_years = self
            .options
            .iter()
            .map(|o| o.get("lyhenne").map_or("".into(), |s| s.clone()))
            .skip(if self.ledger_type == LedgerType::Budgeting {
                1
            } else {
                0
            })
            .rev();
        // .chain(if include_budgeting_cells {
        //     vec!["Talousarvio".into()]
        // } else {
        //     vec![]
        // });
        let mut fy_elem = Html::div_with_class("fiscal-years");
        let mut headers_elem = Html::div_with_class("header-cells")
            .with_child(Html::div_with_class_and_text("", "tili".into()));
        for fiscal_year in fiscal_years {
            let mut fy_container_elem = Html::div_with_class("fy");
            fy_container_elem.push_child(Html::new("div"));
            fy_container_elem.push_child(
                Html::new("div").with_class("fy2").with_child(
                    Html::new("div")
                        .with_class("fiscal-year")
                        .with_text(&fiscal_year)
                        .with_attribute("colspan", "3"),
                ),
            );
            fy_container_elem.push_child(Html::new("div"));
            fy_elem.push_child(fy_container_elem);
            headers_elem.push_child_div_with_class_and_text("", "menot".into());
            headers_elem.push_child_div_with_class_and_text("", "tulot".into());
            headers_elem.push_child_div_with_class_and_text("", "summa".into());
        }

        if include_budgeting_cells {
            let mut fy_container_elem = Html::div_with_class("fy");
            fy_container_elem.push_child(Html::new("div"));
            let title = if self.ledger_type == LedgerType::Budgeting
                && self.options[0].get("lyhenne").is_some()
            {
                self.options[0].get("lyhenne").unwrap().replace('"', "&quot;")
            } else {
                "Talousarvio".to_string()
            };
            fy_container_elem.push_child(
                Html::new("div").with_class("fy2").with_child(
                    Html::new_void("input")
                        .with_class("fiscal-year")
                        .with_attribute("value", title.as_str())
                        .with_attribute("type", "text")
                        .with_attribute("id", "budget-fy-title")
                        .with_attribute("autocomplete", "off"),
                ),
            );
            fy_container_elem.push_child(Html::new("div"));
            fy_elem.push_child(fy_container_elem);
            headers_elem.push_child_div_with_class_and_text("", "menot".into());
            headers_elem.push_child_div_with_class_and_text("", "tulot".into());
            headers_elem.push_child_div_with_class_and_text("", "summa".into());
        }

        income_statement.push_child(
            Html::div_with_class("table-header")
                .with_child(fy_elem)
                .with_child(headers_elem),
        );
        let accounts = self
            .accounts()
            .into_iter()
            .filter(|a| a.t == AccountType::None);
        for account in accounts {
            income_statement
                .push_child(self.html_account_row(account.clone(), include_budgeting_cells));
        }
        income_statement
    }

    fn html_account_row(&self, mut account: Account, include_budgeting_cells: bool) -> Html {
        let mut account_elem = Html::div_with_class("account");
        let is_leaf = account.is_leaf();
        if is_leaf {
            account_elem.push_attribute("class", "leaf");
        }
        if account.transactions.is_empty()
            && (account.debits.iter().all(|a| *a == 0) && account.credits.iter().all(|a| *a == 0))
        {
            account_elem.push_attribute("class", "empty");
        }
        let mut header = Html::div_with_class("header");
        let account_n = if let Some(n) = account.n {
            header.push_attribute("id", format!("a-{n}").as_str());
            if !account.transactions.is_empty() && !include_budgeting_cells {
                Html::div_with_class("n").with_child(
                    Html::new("a")
                        .with_attribute("href", format!("#gl-{n}").as_str())
                        .with_string(format!("{n}")),
                )
            } else {
                Html::div_with_class_and_text("n", format!("{n}"))
            }
        } else {
            Html::div_with_class("n")
        };
        let account_name = account.clone().name;

        header.push_child(
            Html::div_with_class("account-info")
                .with_child(account_n)
                .with_child(Html::div_with_class_and_text("name", account_name.clone())),
        );
        for e in self.html_account_header_numbers(account.clone(), include_budgeting_cells) {
            header.push_child(e);
        }
        account_elem.push_child(header);
        for sub_account in account.clone().sub_accounts {
            let sub_account = sub_account.borrow().to_owned();
            account_elem.push_child(self.html_account_row(sub_account, include_budgeting_cells));
        }
        if account.t == AccountType::LiabilitiesTopLevel {
            let accs = self.accounts();
            let mut profit_account =
                Account::naked(None, "Tilikauden tulos".into(), AccountType::Liabilities);
            profit_account.credits = vec![0; self.l_index + 1];
            profit_account.debits = vec![0; self.l_index + 1];
            profit_account.rec_credits = vec![0; self.l_index + 1];
            profit_account.rec_debits = vec![0; self.l_index + 1];
            accs.iter()
                .filter(|a| a.t == AccountType::None)
                .for_each(|x| {
                    profit_account.debits =
                        zip(profit_account.debits.clone(), x.rec_debits.clone())
                            .map(|(a, b)| a + b)
                            .collect_vec();
                    profit_account.credits =
                        zip(profit_account.credits.clone(), x.rec_credits.clone())
                            .map(|(a, b)| a + b)
                            .collect_vec();
                });
            account.rec_debits = zip(account.rec_debits.clone(), profit_account.debits.clone())
                .map(|(a, b)| a + b)
                .collect_vec();
            account.rec_credits = zip(account.rec_credits.clone(), profit_account.credits.clone())
                .map(|(a, b)| a + b)
                .collect_vec();
            account_elem.push_child(self.html_account_row(profit_account, include_budgeting_cells));
        }
        if !is_leaf {
            let mut footer = Html::div_with_class("footer");

            footer.push_child(
                Html::div_with_class("account-info")
                    .with_child(
                        Html::new("span")
                            .with_class("name")
                            .with_string(account_name),
                    )
                    .with_child(
                        Html::new("span")
                            .with_class("yht")
                            .with_text("yhteensä".into()),
                    ),
            );
            // footer.push_child_div_with_class_and_text(
            //     "account-info",
            //     format!("{} yhteensä", account_name),
            // );
            for e in self.html_account_footer_numbers(account.clone(), include_budgeting_cells) {
                footer.push_child(e);
            }
            account_elem.push_child(footer);
        }
        account_elem
    }

    fn html_account_header_numbers(
        &self,
        account: Account,
        include_budgeting_cells: bool,
    ) -> Vec<Html> {
        let mut elems = vec![];

        let starting_index = if self.ledger_type == LedgerType::Budgeting {
            1
        } else {
            0
        };
        for i in (starting_index..account.debits.len()).rev() {
            if account.t == AccountType::None {
                // println!("{} {:?} {:?}", account.name, account.credits, account.debits);
                elems.push(Html::div_with_class_and_text(
                    "debit amount",
                    Self::debit(account.debits[i]),
                ));
                elems.push(Html::div_with_class_and_text(
                    "credit amount",
                    Self::debit(account.credits[i]),
                ));
                elems.push(Html::div_with_class_and_text(
                    "sum amount",
                    Self::amount_as_string(
                        account.credits[i] - account.debits[i],
                        account.credits[i] != 0 || account.debits[i] != 0,
                    ),
                ));
            } else {
                let sum = if account.t == AccountType::Assets {
                    account.debits[i] - account.credits[i]
                } else {
                    account.credits[i] - account.debits[i]
                };
                elems.push(Html::div_with_class_and_text(
                    "sum amount",
                    Self::amount_as_string(sum, account.is_leaf()),
                ));
            }
        }

        if include_budgeting_cells {
            if account.t == AccountType::None {
                if account.n.is_some() {
                    let (debit, credit, sum) = {
                        if self.ledger_type == LedgerType::Budgeting {
                            (
                                Self::debit(account.debits[0]),
                                Self::debit(account.credits[0]),
                                Self::amount_as_string(
                                    account.credits[0] - account.debits[0],
                                    account.credits[0] != 0 || account.debits[0] != 0,
                                ),
                            )
                        } else {
                            ("".to_string(), "".to_string(), "".to_string())
                        }
                    };
                    elems.push(
                        Html::div_with_class("debit amount budget").with_child(
                            Html::new_void("input")
                                .with_attribute("type", "text")
                                .with_attribute("autocomplete", "off")
                                .with_attribute("value", &debit),
                        ),
                    );
                    elems.push(
                        Html::div_with_class("credit amount budget").with_child(
                            Html::new_void("input")
                                .with_attribute("type", "text")
                                .with_attribute("autocomplete", "off")
                                .with_attribute("value", &credit),
                        ),
                    );
                    elems.push(
                        Html::div_with_class("sum amount budget").with_string(sum), // .with_child(Html::new_void("input").with_attribute("type", "text")),
                    );
                } else {
                    elems.push(Html::div_with_class("debit"));
                    elems.push(Html::div_with_class("credit"));
                    elems.push(Html::div_with_class("sum"));
                }
            }
        }
        elems
    }

    fn html_account_footer_numbers(
        &self,
        account: Account,
        include_budgeting_cells: bool,
    ) -> Vec<Html> {
        let mut elems = vec![];

        // let starting_index = if self.ledger_type==LedgerType::Budgeting {1} else {0};
        for i in (0..account.debits.len()).rev() {
            if account.t == AccountType::None {
                let (dc, cc, sc) = if i == 0 && self.ledger_type == LedgerType::Budgeting {
                    (
                        "debit amount budget",
                        "credit amount budget",
                        "sum amount budget",
                    )
                } else {
                    ("debit amount", "credit amount", "sum amount")
                };
                elems.push(Html::div_with_class_and_text(
                    dc,
                    Self::debit(account.rec_debits[i]),
                ));
                elems.push(Html::div_with_class_and_text(
                    cc,
                    Self::debit(account.rec_credits[i]),
                ));
                elems.push(Html::div_with_class_and_text(
                    sc,
                    Self::amount_as_string(
                        account.rec_credits[i] - account.rec_debits[i],
                        account.rec_credits[i] != 0 || account.rec_debits[i] != 0,
                    ),
                ));
            } else {
                let sum = if account.t == AccountType::Assets {
                    account.rec_debits[i] - account.rec_credits[i]
                } else {
                    account.rec_credits[i] - account.rec_debits[i]
                };
                elems.push(Html::div_with_class_and_text(
                    "sum amount",
                    format!("{}", Self::amount_as_string(sum, true)),
                ));
            }
        }
        if include_budgeting_cells && self.ledger_type != LedgerType::Budgeting {
            if account.t == AccountType::None {
                elems.push(Html::div_with_class("debit amount budget"));
                elems.push(Html::div_with_class("credit amount budget"));
                elems.push(Html::div_with_class("sum amount budget"));
            }
        }
        elems
    }
    fn html_transaction(&self, transaction: Transaction) -> Html {
        let date = transaction.clone().fmt_date();
        let desc = transaction.clone().description;
        let doc = transaction.clone().doc;
        let mut elem = Html::new("div")
            .with_class("transaction")
            .with_attribute("id", format!("d-{}", doc).as_str());
        let mut heading = Html::div_with_class("header");
        heading.push_child_div_with_class_and_text("doc", doc.clone());
        heading.push_child_div_with_class_and_text("date", date);
        heading.push_child_div_with_class_and_text("description", desc);
        elem.push_child(heading);

        let mut entries = Html::div_with_class("entries");
        for entry in transaction.entries {
            let mut entry_elem = Html::div_with_class("entry");
            let mut account_info = Html::div_with_class("account-info");
            let account_n = entry.0;
            let doc = doc.clone();
            account_info.push_child(
                Html::div_with_class("account-n").with_child(
                    Html::new("a")
                        .with_attribute("href", format!("#gl-{account_n}-{doc}").as_str())
                        .with_string(account_n.to_string()),
                ),
            );

            let account_name = if let Some(account) = self.get_account(entry.0) {
                account.name
            } else {
                entry_elem.push_attribute("class", "invalid-account");
                "TUNTEMATON TILI".to_string()
            };
            account_info.push_child_div_with_class_and_text("account-name", account_name);
            entry_elem.push_child(account_info);
            entry_elem.push_child_div_with_class_and_text("debit amount", Self::debit(entry.1));
            entry_elem.push_child_div_with_class_and_text("credit amount", Self::credit(entry.1));
            entries.push_child(entry_elem);
        }
        elem.push_child(entries);
        elem
    }

    fn amount_as_string(amount: i32, render_zero: bool) -> String {
        if !render_zero && amount == 0 {
            return "".to_string();
        }
        format!("{:.2}", (amount as f64) / 100.0)
            .replacen('.', ",", 1)
            .replacen('-', "\u{2212}", 1)
    }

    fn debit(amount: i32) -> String {
        if amount > 0 {
            Ledger::amount_as_string(amount, false)
        } else {
            "".into()
        }
    }
    fn credit(amount: i32) -> String {
        if amount < 0 {
            Ledger::amount_as_string(-amount, false)
        } else {
            "".into()
        }
    }

    #[cfg(debug_assertions)]
    fn head(&self, budgeting: Budgeting) -> Html {
        let mut head = Html::new("head");
        match budgeting {
            Budgeting::Server => {
                head.push_child(
                    Html::new("style").with_raw(include_str!("../html_assets/style.css")),
                );
                head.push_child(
                    Html::new("script").with_raw(include_str!("../html_assets/script.js")),
                );
            }
            _ => {
                head.push_child(
                    Html::new_void("link")
                        .with_attribute("rel", "stylesheet")
                        .with_attribute("href", "../../html_assets/style.css"),
                );
                head.push_child(
                    Html::new("script")
                        .with_attribute("type", "text/javascript")
                        .with_attribute("src", "../../html_assets/script.css"),
                );
            }
        }
        head
    }

    #[cfg(not(debug_assertions))]
    fn head(&self, budgeting: Budgeting) -> Html {
        let mut head = Html::new("head");
        head.push_child(Html::new("style").with_raw(include_str!("../html_assets/mini/style.css")));
        head.push_child(
            Html::new("script").with_raw(include_str!("../html_assets/mini/script.js")),
        );
        head
    }
}
