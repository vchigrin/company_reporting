use crate::model::Money;
use crate::report_parser::{GenericKeys, ParsedLineInfo};
use eyre::{Result, eyre};
use std::collections::HashMap;
use std::str::FromStr;

fn parse_money(line: &str) -> Result<Money> {
    // Sometimes tesseract add garbage '.' characters.
    let filtered: String = line
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '.')
        .collect();
    if filtered.is_empty() {
        return Err(eyre!("Can not parse {} as money", line));
    }
    if filtered == "-" {
        return Ok(Money::zero());
    }
    if filtered.starts_with('(') && filtered.ends_with(')') {
        let val = i64::from_str(filtered.trim_matches(|c| c == '(' || c == ')'))?;
        Ok(Money::from_thousands(-val))
    } else {
        let val = i64::from_str(&filtered)?;
        Ok(Money::from_thousands(val))
    }
}

pub fn classify_lines<Keys: GenericKeys>(
    page_lines: &[String],
    line_to_key: &HashMap<String, Keys>,
) -> Result<Vec<ParsedLineInfo<Keys>>> {
    let mut result = Vec::new();
    for line in page_lines {
        let tokens: Vec<&str> = line.split("  ").filter(|p| !p.is_empty()).collect();
        if tokens.len() != 3 && tokens.len() != 4 {
            log::info!("Skipping non-report line {:?}", tokens);
            continue;
        }
        let line_token: String = tokens[0]
            .to_lowercase()
            .chars()
            .map(|c| {
                // Leave only lowercase Russian, to strip OCR artifacts.
                if ('а'..='я').contains(&c) {
                    return c;
                }
                ' '
            })
            .collect();
        if let Some(key) = line_to_key.get(line_token.trim()) {
            // Last element is the value of previous period.
            // One before last - for current period.
            // Two before last - optional reference to additional info in report.
            let current_value_str = tokens[tokens.len() - 2];
            let money = match parse_money(current_value_str) {
                Ok(m) => m,
                Err(e) => {
                    return Err(eyre!("Error {} on line {}", e, line));
                }
            };
            result.push(ParsedLineInfo::<Keys> {
                key: *key,
                value: money,
                original_line: line.to_owned(),
            });
        } else {
            log::warn!("Unknown line {:?}", line_token);
        }
    }
    Ok(result)
}
