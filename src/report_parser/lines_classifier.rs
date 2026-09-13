use crate::model::Money;
use crate::model::MoneyMultiplier;
use crate::model::{GenericKeys, ParsedLineInfo};
use eyre::{Result, eyre};
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

pub trait KeyClassifier<Keys: GenericKeys> {
    // Note, that line_token is already trimmed, lowe-case string.
    fn try_classify_key(&self, line_token: &str) -> Option<Keys>;
}

pub struct MapKeyClasifier<Keys: GenericKeys> {
    string_to_key: HashMap<String, Keys>,
}

impl<Keys: GenericKeys> MapKeyClasifier<Keys> {
    pub fn new(string_to_key: HashMap<String, Keys>) -> Self {
        Self { string_to_key }
    }
}

impl<Keys: GenericKeys> KeyClassifier<Keys> for MapKeyClasifier<Keys> {
    fn try_classify_key(&self, line_token: &str) -> Option<Keys> {
        self.string_to_key.get(line_token).copied()
    }
}

pub struct LinesClassifier<Keys: GenericKeys> {
    money_multiplier: MoneyMultiplier,
    keys_classifier: Rc<dyn KeyClassifier<Keys>>,
}

impl<Keys: GenericKeys> LinesClassifier<Keys> {
    pub fn new(
        money_multiplier: MoneyMultiplier,
        keys_classifier: Rc<dyn KeyClassifier<Keys>>,
    ) -> Self {
        Self {
            money_multiplier,
            keys_classifier,
        }
    }

    fn parse_money(&self, line: &str) -> Result<Money> {
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
        let val = if filtered.starts_with('(') && filtered.ends_with(')') {
            -i64::from_str(filtered.trim_matches(|c| c == '(' || c == ')'))?
        } else {
            i64::from_str(&filtered)?
        };
        match self.money_multiplier {
            MoneyMultiplier::Thousands => Ok(Money::from_thousands(val)),
            MoneyMultiplier::Millions => Ok(Money::from_millions(val)),
        }
    }

    pub fn classify_lines(&self, page_lines: &[String]) -> Result<Vec<ParsedLineInfo<Keys>>> {
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
            let key = self
                .keys_classifier
                .try_classify_key(line_token.trim())
                .unwrap_or_default();
            // Last element is the value of previous period.
            // One before last - for current period.
            // Two before last - optional reference to additional info in report.
            let current_value_str = tokens[tokens.len() - 2];
            let money = match self.parse_money(current_value_str) {
                Ok(m) => m,
                Err(e) => {
                    log::warn!("Error {} on line {}; Skipping", e, line);
                    continue;
                }
            };
            result.push(ParsedLineInfo::<Keys> {
                key,
                value: money,
                original_line: line.to_owned(),
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum_macros::{EnumString, IntoStaticStr, VariantArray};

    #[derive(
        Debug, Clone, Copy, Default, PartialEq, Eq, EnumString, IntoStaticStr, VariantArray,
    )]
    enum TestKeys {
        #[default]
        OtherDefault,
        Foo,
        Bar,
    }
    impl GenericKeys for TestKeys {}

    #[test]
    fn test_parse_money() {
        let classifier_k = LinesClassifier::new(
            MoneyMultiplier::Thousands,
            Rc::new(MapKeyClasifier::<TestKeys>::new(HashMap::new())),
        );
        assert_eq!(classifier_k.parse_money("1").unwrap().in_roubles(), 1000);
        assert_eq!(
            classifier_k.parse_money("2 123").unwrap().in_roubles(),
            2123000
        );
        assert_eq!(classifier_k.parse_money("(1)").unwrap().in_roubles(), -1000);
        assert_eq!(
            classifier_k.parse_money("(2 123)").unwrap().in_roubles(),
            -2123000
        );
        assert_eq!(classifier_k.parse_money("-1").unwrap().in_roubles(), -1000);
        assert_eq!(classifier_k.parse_money("-").unwrap().in_roubles(), 0);

        // OCR garbage .
        assert_eq!(
            classifier_k.parse_money("2.123").unwrap().in_roubles(),
            2123000
        );

        assert!(classifier_k.parse_money("").is_err());
        assert!(classifier_k.parse_money("asdf").is_err());
        assert!(classifier_k.parse_money("1k").is_err());
        assert!(classifier_k.parse_money("k1").is_err());

        let classifier_m = LinesClassifier::new(
            MoneyMultiplier::Millions,
            Rc::new(MapKeyClasifier::<TestKeys>::new(HashMap::new())),
        );
        assert_eq!(
            classifier_m.parse_money("  9 ").unwrap().in_roubles(),
            9000000
        );
        assert_eq!(
            classifier_m.parse_money("(3489)").unwrap().in_roubles(),
            -3489000000
        );
    }

    #[test]
    fn test_classify_lines() {
        let mut test_mapping = HashMap::<String, TestKeys>::new();
        test_mapping.insert("фу".to_owned(), TestKeys::Foo);
        test_mapping.insert("фоо".to_owned(), TestKeys::Foo);
        test_mapping.insert("бар".to_owned(), TestKeys::Bar);
        let classifier_k = LinesClassifier::new(
            MoneyMultiplier::Thousands,
            Rc::new(MapKeyClasifier::new(test_mapping)),
        );

        let parsed = classifier_k
            .classify_lines(&[
                " фу    1    12    34".to_owned(),
                " фу         89     -".to_owned(),
                " бар        -  (12)  ".to_owned(),
                "unused 1  2   3   5".to_owned(),
            ])
            .unwrap();
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0].key, TestKeys::Foo);
        assert_eq!(parsed[0].value.in_roubles(), 12000);

        assert_eq!(parsed[1].key, TestKeys::Foo);
        assert_eq!(parsed[1].value.in_roubles(), 89000);

        assert_eq!(parsed[2].key, TestKeys::Bar);
        assert_eq!(parsed[2].value.in_roubles(), 0);

        assert_eq!(parsed[3].key, TestKeys::OtherDefault);
        assert_eq!(parsed[3].value.in_roubles(), 3000);
    }
}
