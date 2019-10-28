use serde::de;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum StreamMessage {
    Tweet(Tweet),
    Warning(Warning),
    Limit(Limit),
    Other(de::IgnoredAny),
}

#[derive(Deserialize)]
pub struct ExtendedTweet {
    pub full_text: String,
}

#[derive(Deserialize)]
pub struct Tweet {
    pub id_str: String,
    pub text: String,
    pub extended_tweet: Option<ExtendedTweet>,
    pub lang: String,
}

#[derive(Deserialize)]
pub struct WarningDetails {
    pub code: String,
    pub percent_full: i64,
}

#[derive(Deserialize)]
pub struct Warning {
    pub warning: WarningDetails,
}

#[derive(Deserialize)]
pub struct LimitDetails {
    pub track: i64,
    pub timestamp_ms: String,
}

#[derive(Deserialize)]
pub struct Limit {
    pub limit: LimitDetails,
}
