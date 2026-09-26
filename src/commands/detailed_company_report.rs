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
use std::fmt;
use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
pub enum DisplayedReportType {
    Balance,
    Income,
    Metrics,
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

fn net_debt(report: &Report) -> Money {
    let current_assets = report.balance.assets().current();
    // TODO(vchigrin): May be we should move financial assets to
    // kind of options here, may be using them not always a good idea?
    let liquidity = current_assets.cash() + current_assets.financial_assets();

    let liabilities = report.balance.liabilities();
    let percent_debt = liabilities.current().loans() + liabilities.long_term().loans();
    percent_debt - liquidity
}

fn ebit_ltm(reports_ltm: &[Report]) -> Metric {
    let mut result = Money::zero();
    for r in reports_ltm {
        result += r.income.operational_profit();
    }
    Metric::Money(result)
}

fn net_debt_ebit_ltm(reports_ltm: &[Report]) -> Metric {
    // reports_ltm must not be empty.
    let net_debt = net_debt(reports_ltm.last().unwrap());
    let Metric::Money(ebit_ltm) = ebit_ltm(reports_ltm) else {
        panic!("Unexpected ebit_ltm metric type");
    };
    let result = (net_debt.in_roubles() as f64) / (ebit_ltm.in_roubles() as f64);
    Metric::Ratio(result)
}

fn icr(reports_ltm: &[Report]) -> Metric {
    // reports_ltm must not be empty.
    let last_income = &reports_ltm.last().unwrap().income;
    let operational_profit = last_income.operational_profit();
    let net_financial_expenses = last_income
        .financial_segment()
        .net_financial_expenses()
        .abs();
    let result =
        (operational_profit.in_roubles() as f64) / (net_financial_expenses.in_roubles() as f64);
    Metric::Ratio(result)
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
enum Metric {
    Money(Money),
    Ratio(f64),
}

impl Metric {
    fn percent_increase(prev: Metric, cur: Metric) -> f64 {
        match prev {
            Metric::Money(prev_money) => {
                let Metric::Money(cur_money) = cur else {
                    panic!("Heteroheneus percent attempt");
                };
                if prev_money != Money::zero() {
                    ((cur_money - prev_money).in_roubles() as f64 * 100.)
                        / (prev_money.in_roubles() as f64)
                } else {
                    f64::INFINITY
                }
            }
            Metric::Ratio(prev_ratio) => {
                let Metric::Ratio(cur_ratio) = cur else {
                    panic!("Heteroheneus percent attempt");
                };
                if prev_ratio != 0. {
                    ((cur_ratio - prev_ratio) * 100.) / (prev_ratio)
                } else {
                    f64::INFINITY
                }
            }
        }
    }
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Metric::Money(m) => m.fmt(f),
            Metric::Ratio(r) => r.fmt(f),
        }
    }
}

type MoneyGetter = fn(&Report) -> Money;
// Last twelve month reports, sorted (most recent period last).
// Guaranted not empty.
type ReportsLTM = [Report];
type MetricGetter = fn(&ReportsLTM) -> Metric;

enum ValueGetter {
    Money(MoneyGetter),
    Metric(MetricGetter),
}

struct RowDescriptor {
    title: &'static str,
    value_getter: ValueGetter,
    level: i32,
}

const CLOSED_MARK: &str = "\u{25b6} "; // Arrow to right
const OPENED_MARK: &str = "\u{25bc} "; // Arrow down
const MAX_LEVEL: i32 = 2;
const BALANCE_ROWS: [RowDescriptor; 26] = [
    RowDescriptor {
        title: "Assets",
        value_getter: ValueGetter::Money(total_assets),
        level: 0,
    },
    RowDescriptor {
        title: "Non-current assets",
        value_getter: ValueGetter::Money(total_non_current_assets),
        level: 1,
    },
    RowDescriptor {
        title: "Non-material assets",
        value_getter: ValueGetter::Money(non_material_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Fixed assets",
        value_getter: ValueGetter::Money(fixed_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Financial investments (non-current)",
        value_getter: ValueGetter::Money(non_current_financial_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Other non-current assets",
        value_getter: ValueGetter::Money(non_current_other),
        level: 2,
    },
    RowDescriptor {
        title: "Current assets",
        value_getter: ValueGetter::Money(total_current_assets),
        level: 1,
    },
    RowDescriptor {
        title: "Inventory",
        value_getter: ValueGetter::Money(physical_inventory),
        level: 2,
    },
    RowDescriptor {
        title: "Accounts receivable",
        value_getter: ValueGetter::Money(accounts_receivable),
        level: 2,
    },
    RowDescriptor {
        title: "Financial investments (current)",
        value_getter: ValueGetter::Money(current_financial_assets),
        level: 2,
    },
    RowDescriptor {
        title: "Cash and equivalents",
        value_getter: ValueGetter::Money(cash),
        level: 2,
    },
    RowDescriptor {
        title: "Other current assets",
        value_getter: ValueGetter::Money(current_other),
        level: 2,
    },
    RowDescriptor {
        title: "Equity",
        value_getter: ValueGetter::Money(total_equity),
        level: 0,
    },
    RowDescriptor {
        title: "Authorised capital",
        value_getter: ValueGetter::Money(authorised_capital),
        level: 2,
    },
    RowDescriptor {
        title: "Capital surplus",
        value_getter: ValueGetter::Money(capital_surplus),
        level: 2,
    },
    RowDescriptor {
        title: "Retained earnings",
        value_getter: ValueGetter::Money(retained_earnings),
        level: 2,
    },
    RowDescriptor {
        title: "Other equity",
        value_getter: ValueGetter::Money(equity_other),
        level: 2,
    },
    RowDescriptor {
        title: "Liabilities",
        value_getter: ValueGetter::Money(total_liabilities),
        level: 0,
    },
    RowDescriptor {
        title: "Long-term liabilities",
        value_getter: ValueGetter::Money(total_long_term_liabilities),
        level: 1,
    },
    RowDescriptor {
        title: "Long-term loans",
        value_getter: ValueGetter::Money(long_term_loans),
        level: 2,
    },
    RowDescriptor {
        title: "Long-term accounts payable",
        value_getter: ValueGetter::Money(long_term_accounts_payable),
        level: 2,
    },
    RowDescriptor {
        title: "Other long-term liabilities",
        value_getter: ValueGetter::Money(long_term_other),
        level: 2,
    },
    RowDescriptor {
        title: "Short-term liabilities",
        value_getter: ValueGetter::Money(total_current_liabilities),
        level: 1,
    },
    RowDescriptor {
        title: "Short-term loans",
        value_getter: ValueGetter::Money(current_loans),
        level: 2,
    },
    RowDescriptor {
        title: "Short-term accounts payable",
        value_getter: ValueGetter::Money(current_accounts_payable),
        level: 2,
    },
    RowDescriptor {
        title: "Other current liabilities",
        value_getter: ValueGetter::Money(current_liabilities_other),
        level: 2,
    },
];

const INCOME_ROWS: [RowDescriptor; 15] = [
    RowDescriptor {
        title: "Gross profit",
        value_getter: ValueGetter::Money(gross_profit),
        level: 0,
    },
    RowDescriptor {
        title: "Sales revenue",
        value_getter: ValueGetter::Money(sales_revenue),
        level: 2,
    },
    RowDescriptor {
        title: "Cost of sales",
        value_getter: ValueGetter::Money(cost_of_sales),
        level: 2,
    },
    RowDescriptor {
        title: "Operational expenses",
        value_getter: ValueGetter::Money(operational_expenses),
        level: 0,
    },
    RowDescriptor {
        title: "Comercial expenses",
        value_getter: ValueGetter::Money(commercial_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Management expenses",
        value_getter: ValueGetter::Money(management_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Other income",
        value_getter: ValueGetter::Money(other_income),
        level: 2,
    },
    RowDescriptor {
        title: "Other expenses",
        value_getter: ValueGetter::Money(other_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Operational profit",
        value_getter: ValueGetter::Money(operational_profit),
        level: 0,
    },
    RowDescriptor {
        title: "Financial net expenses",
        value_getter: ValueGetter::Money(net_financial_expenses),
        level: 0,
    },
    RowDescriptor {
        title: "Financial income",
        value_getter: ValueGetter::Money(financial_income),
        level: 2,
    },
    RowDescriptor {
        title: "Financial expenses",
        value_getter: ValueGetter::Money(financial_expenses),
        level: 2,
    },
    RowDescriptor {
        title: "Profit before tax",
        value_getter: ValueGetter::Money(profit_before_tax),
        level: 0,
    },
    RowDescriptor {
        title: "Profit tax",
        value_getter: ValueGetter::Money(profit_tax),
        level: 0,
    },
    RowDescriptor {
        title: "Net profit",
        value_getter: ValueGetter::Money(net_profit),
        level: 0,
    },
];

const RATIOS_ROWS: [RowDescriptor; 4] = [
    RowDescriptor {
        title: "Net debt",
        value_getter: ValueGetter::Money(net_debt),
        level: 0,
    },
    RowDescriptor {
        title: "EBIT LTM",
        value_getter: ValueGetter::Metric(ebit_ltm),
        level: 0,
    },
    RowDescriptor {
        title: "Net debt/EBIT LTM",
        value_getter: ValueGetter::Metric(net_debt_ebit_ltm),
        level: 0,
    },
    RowDescriptor {
        title: "ICR",
        value_getter: ValueGetter::Metric(icr),
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

    fn get_cell_text(
        &self,
        cur_value: Metric,
        maybe_prev_value: Option<Metric>,
        balance: &BalanceReport,
    ) -> String {
        match self.value_display_mode {
            ValueDisplayMode::Absolute => cur_value.to_string(),
            ValueDisplayMode::PercentToPrevious => {
                if let Some(prev_value) = maybe_prev_value {
                    let percent = Metric::percent_increase(prev_value, cur_value);
                    format!("{:+.2}%", percent)
                } else {
                    // First column will contain abolute values.
                    cur_value.to_string()
                }
            }
            ValueDisplayMode::PercentFromBalance => {
                let Metric::Money(cur_money) = cur_value else {
                    panic!("Unexpected metric in PercentFromBalance mode");
                };
                let percent: f64 = (cur_money.in_roubles() as f64 * 100.)
                    / (balance.assets().total().in_roubles() as f64);
                format!("{:.2}%", percent)
            }
        }
    }

    fn build_cell(
        &self,
        maybe_cur_value: Option<Metric>,
        maybe_prev_value: Option<Metric>,
        report: &Report,
    ) -> Cell<'static> {
        if let Some(cur_value) = maybe_cur_value {
            let text = self.get_cell_text(cur_value, maybe_prev_value, &report.balance);
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
            // Cell::default not suits here - it returns cell with
            // zero column span.
            Cell::new(String::new())
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

    fn build_metric(&self, period: Period, value_getter: &ValueGetter) -> Option<Metric> {
        match value_getter {
            ValueGetter::Money(money_getter) => {
                let maybe_cur_report = &self.reports.get(&period);
                let maybe_money = maybe_cur_report.map(money_getter);
                maybe_money.map(Metric::Money)
            }
            ValueGetter::Metric(metric_getter) => {
                let ltm_periods = Period::get_periods_for_ltm(period);
                let mut ltm_reports = Vec::<Report>::with_capacity(ltm_periods.len());
                for ltm_period in ltm_periods {
                    if let Some(report) = &self.reports.get(&ltm_period) {
                        ltm_reports.push((*report).clone());
                    } else {
                        return None;
                    }
                }
                Some(metric_getter(&ltm_reports))
            }
        }
    }

    fn build_row(&self, descriptor: &RowDescriptor) -> Row<'static> {
        let mut cells = vec![self.build_title_cell(descriptor)];
        let mut maybe_prev_value: Option<Metric> = None;
        for period in &self.periods {
            let maybe_cur_value = self.build_metric(*period, &descriptor.value_getter);
            let maybe_report = &self.reports.get(period).unwrap();
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
        DisplayedReportType::Metrics => &RATIOS_ROWS,
    };
    let table = CompanyReportTable::new(company, row_descriptors, args.displayed_report_type)?;
    let mut terminal = ratatui::init();
    let result = run_report_viewer(&mut terminal, table);
    ratatui::restore();
    result
}
