use strum_macros::{EnumString, IntoStaticStr};
use time::Date;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum RatingAgency {
    Akra,
    ExpertRA,
    Nkr,
    Nra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum Rating {
    #[strum(serialize = "AAA")]
    Aaa,
    #[strum(serialize = "AA+")]
    AAPlus,
    AA,
    #[strum(serialize = "AA-")]
    AAMinus,
    #[strum(serialize = "A+")]
    APlus,
    A,
    #[strum(serialize = "A-")]
    AMinus,
    #[strum(serialize = "BBB+")]
    BBBPlus,
    #[strum(serialize = "BBB")]
    Bbb,
    #[strum(serialize = "BBB-")]
    BBBMinus,
    #[strum(serialize = "BB+")]
    BBPlus,
    BB,
    #[strum(serialize = "BB-")]
    BBMinus,
    #[strum(serialize = "B+")]
    BPlus,
    B,
    #[strum(serialize = "B-")]
    BMinus,
    #[strum(serialize = "CCC")]
    Ccc,
    CC,
    C,
    D,
    Withdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum RatingForecast {
    Stable,
    Positive,
    Negative,
    Evolving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RAConclusion {
    pub date: Date,
    pub rating_agency: RatingAgency,
    pub rating: Rating,
    pub forecast: RatingForecast,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn rating_str(rating: Rating) -> &'static str {
        rating.into()
    }

    #[test]
    fn rating_parse() {
        assert_eq!(Rating::from_str("AAA").unwrap(), Rating::Aaa);
        assert_eq!(Rating::from_str("AA+").unwrap(), Rating::AAPlus);
        assert_eq!(Rating::from_str("A-").unwrap(), Rating::AMinus);
        assert_eq!(Rating::from_str("BBB").unwrap(), Rating::Bbb);
        assert_eq!(Rating::from_str("B+").unwrap(), Rating::BPlus);
        assert_eq!(Rating::from_str("D").unwrap(), Rating::D);
        assert!(Rating::from_str("AA++").is_err());
        assert!(Rating::from_str("A++").is_err());
    }

    #[test]
    fn rating_string() {
        assert_eq!(rating_str(Rating::AAPlus), "AA+");
        assert_eq!(rating_str(Rating::AMinus), "A-");
        assert_eq!(rating_str(Rating::Aaa), "AAA");
        assert_eq!(rating_str(Rating::Bbb), "BBB");
    }

    #[test]
    fn rating_agency_parse() {
        assert_eq!(RatingAgency::from_str("Akra").unwrap(), RatingAgency::Akra);
        assert_eq!(RatingAgency::from_str("Nkr").unwrap(), RatingAgency::Nkr);
        assert!(RatingAgency::from_str("Moody's").is_err());
    }

    #[test]
    fn forecast_parse() {
        assert_eq!(
            RatingForecast::from_str("Evolving").unwrap(),
            RatingForecast::Evolving
        );
        assert!(RatingForecast::from_str("Stable ").is_err());
    }
}
