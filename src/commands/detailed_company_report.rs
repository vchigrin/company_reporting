use crate::model;
use crate::model::balance_report::BalanceReport;
use crate::model::{Money, Period, Report};
use crate::storage;
use clap::{ArgGroup, Args};
use crossterm::event::{KeyCode, KeyEvent};
use eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};
use std::collections::HashMap;
use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
pub enum DisplayedReportType {
    Balance,
    Income,
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["inn", "name"])))]
pub struct GetCompanyArgs {
    #[arg(long)]
    pub inn: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub displayed_report_type: DisplayedReportType,
}

fn total_assets(report: &Report) -> Money {
    report.balance.assets().total()
}

fn total_non_current_assets(report: &Report) -> Money {
    report.balance.assets().non_current().total()
}

fn total_current_assets(report: &Report) -> Money {
    report.balance.assets().current().total()
}

fn non_material_assets(report: &Report) -> Money {
    report.balance.assets().non_current().non_material_assets()
}

fn fixed_assets(report: &Report) -> Money {
    report.balance.assets().non_current().fixed_assets()
}

fn non_current_financial_assets(report: &Report) -> Money {
    report.balance.assets().non_current().financial_assets()
}

fn non_current_other(report: &Report) -> Money {
    report.balance.assets().non_current().other()
}

fn physical_inventory(report: &Report) -> Money {
    report.balance.assets().current().physical_inventory()
}

fn accounts_receivable(report: &Report) -> Money {
    report.balance.assets().current().accounts_receivable()
}

fn current_financial_assets(report: &Report) -> Money {
    report.balance.assets().current().financial_assets()
}

fn cash(report: &Report) -> Money {
    report.balance.assets().current().cash()
}

fn current_other(report: &Report) -> Money {
    report.balance.assets().current().other()
}

fn total_equity(report: &Report) -> Money {
    report.balance.equity().total()
}

fn total_liabilities(report: &Report) -> Money {
    report.balance.liabilities().total()
}

fn total_long_term_liabilities(report: &Report) -> Money {
    report.balance.liabilities().long_term().total()
}

fn total_current_liabilities(report: &Report) -> Money {
    report.balance.liabilities().current().total()
}

fn authorised_capital(report: &Report) -> Money {
    report.balance.equity().authorised_capital()
}

fn capital_surplus(report: &Report) -> Money {
    report.balance.equity().capital_surplus()
}

fn retained_earnings(report: &Report) -> Money {
    report.balance.equity().retained_earnings()
}

fn equity_other(report: &Report) -> Money {
    report.balance.equity().other()
}

fn long_term_loans(report: &Report) -> Money {
    report.balance.liabilities().long_term().loans()
}

fn long_term_accounts_payable(report: &Report) -> Money {
    report.balance.liabilities().long_term().accounts_payable()
}

fn long_term_other(report: &Report) -> Money {
    report.balance.liabilities().long_term().other()
}

fn current_loans(report: &Report) -> Money {
    report.balance.liabilities().current().loans()
}

fn current_accounts_payable(report: &Report) -> Money {
    report.balance.liabilities().current().accounts_payable()
}

fn current_liabilities_other(report: &Report) -> Money {
    report.balance.liabilities().current().other()
}

fn gross_profit(report: &Report) -> Money {
    report.income.gross_profit()
}

fn sales_revenue(report: &Report) -> Money {
    report.income.gross_profit_segment().sales_revenue()
}

fn cost_of_sales(report: &Report) -> Money {
    report.income.gross_profit_segment().cost_of_sales()
}

fn operational_expenses(report: &Report) -> Money {
    report.income.operational_segment().operational_expenses()
}

fn commercial_expenses(report: &Report) -> Money {
    report.income.operational_segment().commercial_expenses()
}

fn management_expenses(report: &Report) -> Money {
    report.income.operational_segment().management_expenses()
}

fn other_income(report: &Report) -> Money {
    report.income.operational_segment().other_income()
}

fn other_expenses(report: &Report) -> Money {
    report.income.operational_segment().other_expenses()
}

fn operational_profit(report: &Report) -> Money {
    report.income.operational_profit()
}

fn net_financial_expenses(report: &Report) -> Money {
    report.income.financial_segment().net_financial_expenses()
}

fn financial_income(report: &Report) -> Money {
    report.income.financial_segment().financial_income()
}

fn financial_expenses(report: &Report) -> Money {
    report.income.financial_segment().financial_expenses()
}

fn profit_tax(report: &Report) -> Money {
    report.income.profit_tax()
}

fn profit_before_tax(report: &Report) -> Money {
    report.income.profit_before_tax()
}

fn net_profit(report: &Report) -> Money {
    report.income.net_profit()
}

type MoneyGetter = fn(&Report) -> Money;

struct RowDescriptor {
    title: &'static str,
    money_getter: Option<MoneyGetter>,
    level: i32,
}

const CLOSED_MARK: &str = "\u{25b6} "; // Arrow to right
const OPENED_MARK: &str = "\u{25bc} "; // Arrow down
const MAX_LEVEL: i32 = 2;
const BALANCE_ROWS: [RowDescriptor; 26] = [
    RowDescriptor {
        title: "Assets",
        money_getter: Some(total_assets),
        level: 0,
    },
    RowDescriptor {
        title: "Non-current assets",
        money_getter: Some(total_non_current_assets),
        level: 1,
    },
    RowDescriptor {
        title: "Non-material assets",
        money_getter: Some(non_material_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Fixed assets",
        money_getter: Some(fixed_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Financial investments (non-current)",
        money_getter: Some(non_current_financial_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Other non-current assets",
        money_getter: Some(non_current_other),
        level: 2,
    },
    RowDescriptor {
        title: "Current assets",
        money_getter: Some(total_current_assets),
        level: 1,
    },
    RowDescriptor {
        title: "Inventory",
        money_getter: Some(physical_inventory),
        level: 2,
    },
    RowDescriptor {
        title: "Accounts receivable",
        money_getter: Some(accounts_receivable),
        level: 2,
    },
    RowDescriptor {
        title: "Financial investments (current)",
        money_getter: Some(current_financial_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Cash and equivalents",
        money_getter: Some(cash),
        level: 2,
    },
    RowDescriptor {
        title: "Other current assets",
        money_getter: Some(current_other),
        level: 2,
    },
    RowDescriptor {
        title: "Equity",
        money_getter: Some(total_equity),
        level: 0,
    },
    RowDescriptor {
        title: "Authorised capital",
        money_getter: Some(authorised_capital),
        level: 2,
    },
    RowDescriptor {
        title: "Capital surplus",
        money_getter: Some(capital_surplus),
        level: 2,
    },
    RowDescriptor {
        title: "Retained earnings",
        money_getter: Some(retained_earnings),
        level: 2,
    },
    RowDescriptor {
        title: "Other equity",
        money_getter: Some(equity_other),
        level: 2,
    },
    RowDescriptor {
        title: "Liabilities",
        money_getter: Some(total_liabilities),
        level: 0,
    },
    RowDescriptor {
        title: "Long-term liabilities",
        money_getter: Some(total_long_term_liabilities),
        level: 1,
    },
    RowDescriptor {
        title: "Long-term loans",
        money_getter: Some(long_term_loans),
        level: 2,
    },
    RowDescriptor {
        title: "Long-term accounts payable",
        money_getter: Some(long_term_accounts_payable),
        level: 2,
    },
    RowDescriptor {
        title: "Other long-term liabilities",
        money_getter: Some(long_term_other),
        level: 2,
    },
    RowDescriptor {
        title: "Short-term liabilities",
        money_getter: Some(total_current_liabilities),
        level: 1,
    },
    RowDescriptor {
        title: "Short-term loans",
        money_getter: Some(current_loans),
        level: 2,
    },
    RowDescriptor {
        title: "Short-term accounts payable",
        money_getter: Some(current_accounts_payable),
        level: 2,
    },
    RowDescriptor {
        title: "Other current liabilities",
        money_getter: Some(current_liabilities_other),
        level: 2,
    },
];

const INCOME_ROWS: [RowDescriptor; 15] = [
    RowDescriptor {
        title: "Gross profit",
        money_getter: Some(gross_profit),
        level: 0,
    },
    RowDescriptor {
        title: "Sales revenue",
        money_getter: Some(sales_revenue),
        level: 2,
    },
    RowDescriptor {
        title: "Cost of sales",
        money_getter: Some(cost_of_sales),
        level: 2,
    },
    RowDescriptor {
        title: "Operational expenses",
        money_getter: Some(operational_expenses),
        level: 0,
    },
    RowDescriptor {
        title: "Comercial expenses",
        money_getter: Some(commercial_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Management expenses",
        money_getter: Some(management_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Other income",
        money_getter: Some(other_income),
        level: 2,
    },
    RowDescriptor {
        title: "Other expenses",
        money_getter: Some(other_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Operational profit",
        money_getter: Some(operational_profit),
        level: 0,
    },
    RowDescriptor {
        title: "Financial net expenses",
        money_getter: Some(net_financial_expenses),
        level: 0,
    },
    RowDescriptor {
        title: "Financial income",
        money_getter: Some(financial_income),
        level: 2,
    },
    RowDescriptor {
        title: "Financial expenses",
        money_getter: Some(financial_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Profit before tax",
        money_getter: Some(profit_before_tax),
        level: 0,
    },
    RowDescriptor {
        title: "Profit tax",
        money_getter: Some(profit_tax),
        level: 0,
    },
    RowDescriptor {
        title: "Net profit",
        money_getter: Some(net_profit),
        level: 0,
    },
];

enum ValueDisplayMode {
    Absolute,
    PercentToPrevious,
    PercentFromBalance,
}

struct CompanyReportTable {
    table: Table<'static>,
    table_state: TableState,
    value_display_mode: ValueDisplayMode,
    row_descriptors: &'static [RowDescriptor],
    reports: HashMap<Period, Report>,
    periods: Vec<Period>,
    max_expand_level: i32,
    // TODO(vchigrin): Find a way to disable PercentFromBalance in more
    // elegant way...
    displayed_report_type: DisplayedReportType,
}

impl CompanyReportTable {
    fn new(
        company: model::CompanyInfo,
        row_descriptors: &'static [RowDescriptor],
        displayed_report_type: DisplayedReportType,
    ) -> Result<Self> {
        let reports = company.build_reports()?;
        let mut periods: Vec<Period> = reports.keys().copied().collect();
        periods.sort();

        let column_widths = (0..=periods.len()).map(|_| Constraint::Fill(1));

        let header = Row::new(
            std::iter::once(Cell::from("Period")).chain(
                periods
                    .iter()
                    .map(|period| Cell::from(period.short_string())),
            ),
        );

        let mut result = Self {
            table: Table::new(Vec::<Row<'static>>::default(), column_widths)
                .header(header)
                .column_spacing(1)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("{} ({})", company.name, company.inn)),
                )
                .column_highlight_style(Style::default().bold()),
            table_state: TableState::default(),
            value_display_mode: ValueDisplayMode::Absolute,
            reports,
            row_descriptors,
            periods,
            max_expand_level: 0,
            displayed_report_type,
        };
        result.build_rows();
        Ok(result)
    }

    fn draw(&mut self, f: &mut Frame<'_>) {
        f.render_stateful_widget(&self.table, f.area(), &mut self.table_state);
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('h') => {
                self.value_display_mode = ValueDisplayMode::PercentToPrevious;
                self.build_rows();
            }
            KeyCode::Char('v') => {
                if self.displayed_report_type == DisplayedReportType::Balance {
                    self.value_display_mode = ValueDisplayMode::PercentFromBalance;
                    self.build_rows();
                }
            }
            KeyCode::Char('a') => {
                self.value_display_mode = ValueDisplayMode::Absolute;
                self.build_rows();
            }
            KeyCode::Char('e') => {
                self.max_expand_level = (self.max_expand_level + 1) % (MAX_LEVEL + 1);
                self.build_rows();
            }
            KeyCode::Left => {
                self.table_state.select_previous_column();
            }
            KeyCode::Right => {
                self.table_state.select_next_column();
            }
            _ => {}
        }
        false
    }

    fn get_balance_cell_text(
        &self,
        cur_value: Money,
        maybe_prev_value: Option<Money>,
        balance: &BalanceReport,
    ) -> String {
        match self.value_display_mode {
            ValueDisplayMode::Absolute => cur_value.to_string(),
            ValueDisplayMode::PercentToPrevious => {
                if let Some(prev_value) = maybe_prev_value {
                    let percent: f64 = if prev_value != Money::zero() {
                        ((cur_value - prev_value).in_roubles() as f64 * 100.)
                            / (prev_value.in_roubles() as f64)
                    } else {
                        f64::INFINITY
                    };
                    format!("{:+.2}%", percent)
                } else {
                    // First column will contain abolute values.
                    cur_value.to_string()
                }
            }
            ValueDisplayMode::PercentFromBalance => {
                let percent: f64 = (cur_value.in_roubles() as f64 * 100.)
                    / (balance.assets().total().in_roubles() as f64);
                format!("{:.2}%", percent)
            }
        }
    }

    fn build_cell(
        &self,
        maybe_cur_value: Option<Money>,
        maybe_prev_value: Option<Money>,
        report: &Report,
    ) -> Cell<'static> {
        if let Some(cur_value) = maybe_cur_value {
            let text = self.get_balance_cell_text(cur_value, maybe_prev_value, &report.balance);
            let mut cell = Cell::new(text);
            if let Some(prev_value) = maybe_prev_value {
                // Important notice: color here show abolute value difference,
                // what may be not intutivie in case "PercentFromBalance" display
                // mode. Consider refactoring...
                if cur_value > prev_value {
                    cell = cell.style(Style::default().fg(Color::Green));
                } else if cur_value < prev_value {
                    cell = cell.style(Style::default().fg(Color::Red));
                }
            }
            cell
        } else {
            Cell::default()
        }
    }

    fn build_title_cell(&self, descriptor: &RowDescriptor) -> Cell<'static> {
        let mut title = String::new();
        for _ in 0..descriptor.level {
            title.push_str("  ");
        }
        if descriptor.level < self.max_expand_level {
            title.push_str(OPENED_MARK);
        } else if descriptor.level < MAX_LEVEL && descriptor.level == self.max_expand_level {
            title.push_str(CLOSED_MARK);
        }
        title.push_str(descriptor.title);
        let mut style = Style::default();
        match descriptor.level {
            0 => {
                style = style.underlined().light_red();
            }
            1 => {
                style = style.yellow();
            }
            _ => {}
        }
        Cell::new(title).style(style)
    }

    fn build_row(&self, descriptor: &RowDescriptor) -> Row<'static> {
        let mut cells = vec![self.build_title_cell(descriptor)];
        let mut maybe_prev_value: Option<Money> = None;
        for period in &self.periods {
            let maybe_report = &self.reports.get(period).unwrap();
            let maybe_cur_value = descriptor.money_getter.map(|getter| getter(maybe_report));
            cells.push(self.build_cell(maybe_cur_value, maybe_prev_value, maybe_report));
            maybe_prev_value = maybe_cur_value;
        }
        Row::new(cells)
    }

    fn build_rows(&mut self) {
        let mut rows: Vec<Row> = Vec::with_capacity(self.row_descriptors.len());
        for descriptor in self.row_descriptors {
            if descriptor.level > self.max_expand_level {
                continue;
            }
            rows.push(self.build_row(descriptor));
        }
        self.table = self.table.clone().rows(rows);
    }
}

fn run_report_viewer(terminal: &mut DefaultTerminal, mut table: CompanyReportTable) -> Result<()> {
    loop {
        terminal.draw(|f| table.draw(f))?;
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            // Belive having it as separate if is better readable.
            #[allow(clippy::collapsible_if)]
            if table.handle_key(key) {
                return Ok(());
            }
        }
    }
}

pub fn process_detailed_company_report(
    db: &mut storage::Storage,
    args: &GetCompanyArgs,
) -> Result<()> {
    let company = match (&args.inn, &args.name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };

    let row_descriptors: &'static [RowDescriptor] = match args.displayed_report_type {
        DisplayedReportType::Balance => &BALANCE_ROWS,
        DisplayedReportType::Income => &INCOME_ROWS,
    };
    let table = CompanyReportTable::new(company, row_descriptors, args.displayed_report_type)?;
    let mut terminal = ratatui::init();
    let result = run_report_viewer(&mut terminal, table);
    ratatui::restore();
    result
}
