use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TWEntry {
    #[serde(deserialize_with = "tw_date_parser")]
    pub start: Option<NaiveDateTime>,
    #[serde(deserialize_with = "tw_date_parser", default = "default_date")]
    pub end: Option<NaiveDateTime>,
    pub tags: Vec<String>,
}

impl TWEntry {
    pub fn get_duration(&self) -> i64 {
        if let Some(end) = self.end {
            let start = self.start.unwrap();
            let duration = end - start;
            duration.num_minutes()
        } else {
            0
        }
    }
}

fn default_date() -> Option<NaiveDateTime> {
    Some(NaiveDate::from_ymd(0, 1, 1).and_hms(0, 0, 0))
}

fn tw_date_parser<'de, D: Deserializer<'de>>(d: D) -> Result<Option<NaiveDateTime>, D::Error> {
    let s: Option<String> = Deserialize::deserialize(d)?;

    if let Some(string) = s {
        let parsed = NaiveDateTime::parse_from_str(&string, "%Y%m%dT%H%M%SZ");

        if let Ok(result) = parsed {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
