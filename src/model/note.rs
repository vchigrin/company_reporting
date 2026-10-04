use time::Date;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: i64,
    pub date: Date,
    pub text: String,
}
