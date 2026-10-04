use super::balance_report;
use super::income_report;
use super::metrics_report;
use super::{Metric, MetricInput, RowDescriptor, ValueGetter};
use crate::model;
use crate::model::balance_report::BalanceReport;
use crate::model::{Period, Report};
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
use strum::VariantArray;
use strum_macros::{EnumString, IntoStaticStr, VariantArray};

#[derive(Debug, Clone, Copy, PartialEq, EnumString, IntoStaticStr, VariantArray)]
pub enum DisplayedReportType {
    Balance,
    Income,
    Metrics,
}

impl clap::ValueEnum for DisplayedReportType {
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let val: &'static str = self.into();
        Some(clap::builder::PossibleValue::new(val))
    }

    fn value_variants<'a>() -> &'a [Self] {
        Self::VARIANTS
    }
}

#[derive(Debug, Args)]
#[command(group(ArgGroup::new("company").required(true).multiple(false).args(["inn", "name"])))]
pub struct DetailedReportArgs {
    #[arg(long)]
    pub inn: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub displayed_report_type: DisplayedReportType,
}

const CLOSED_MARK: &str = "\u{25b6} "; // Arrow to right
const OPENED_MARK: &str = "\u{25bc} "; // Arrow down
const MAX_LEVEL: i32 = 2;

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
            ValueDisplayMode::Absolute => format!("{:.2}", cur_value),
            ValueDisplayMode::PercentToPrevious => {
                if let Some(prev_value) = maybe_prev_value {
                    let percent = Metric::percent_increase(prev_value, cur_value);
                    format!("{:+.2}%", percent)
                } else {
                    // First column will contain abolute values.
                    format!("{:.2}", cur_value)
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

    fn collect_ltm_reports(&self, period: Period) -> Option<Vec<Report>> {
        let ltm_periods = Period::get_periods_for_ltm(period);
        let mut ltm_reports = Vec::<Report>::with_capacity(ltm_periods.len());
        for ltm_period in ltm_periods {
            if let Some(report) = &self.reports.get(&ltm_period) {
                ltm_reports.push((*report).clone());
            } else {
                return None;
            }
        }
        Some(ltm_reports)
    }

    fn build_metric(&self, period: Period, value_getter: &ValueGetter) -> Option<Metric> {
        match value_getter {
            ValueGetter::None => None,
            ValueGetter::Money(money_getter) => {
                let maybe_cur_report = &self.reports.get(&period);
                let maybe_money = maybe_cur_report.map(money_getter);
                maybe_money.map(Metric::Money)
            }
            ValueGetter::Metric(metric_getter) => {
                let Some(cur_report) = &self.reports.get(&period) else {
                    return None;
                };
                let metric_input = MetricInput {
                    reports_ltm: self.collect_ltm_reports(period),
                    last_report: (*cur_report).clone(),
                };
                metric_getter(&metric_input)
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
    args: &DetailedReportArgs,
) -> Result<()> {
    let company = match (&args.inn, &args.name) {
        (Some(inn), None) => db.get_company_by_inn(inn)?,
        (None, Some(name)) => db.get_company_by_name(name)?,
        _ => {
            panic!("Conflicting args passed");
        }
    };

    let row_descriptors: &'static [RowDescriptor] = match args.displayed_report_type {
        DisplayedReportType::Balance => &balance_report::ROWS,
        DisplayedReportType::Income => &income_report::ROWS,
        DisplayedReportType::Metrics => &metrics_report::ROWS,
    };
    let table = CompanyReportTable::new(company, row_descriptors, args.displayed_report_type)?;
    let mut terminal = ratatui::init();
    let result = run_report_viewer(&mut terminal, table);
    ratatui::restore();
    result
}
