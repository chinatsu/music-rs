use serde::{Deserialize, Serialize};
use time::Date;
use uuid::Uuid;



#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
pub struct Album {
    pub id: Uuid,
    pub title: String,
    pub artists: Option<Vec<Artist>>,
    #[serde(with = "my_date_format")]
    pub date: Date,
    pub genres: Option<Vec<Genre>>,
    pub url: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct NewAlbum {
    pub album: String,
    pub artists: Vec<String>,
    #[serde(with = "my_date_format")]
    pub date: Date,
    pub genres: Vec<String>,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct InsertedAlbum {
    pub id: Uuid,
    pub title: String,
    #[serde(with = "my_date_format")]
    pub date: Date,
    pub url: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
pub struct Genre {
    pub id: Uuid,
    pub name: String,
}

mod my_date_format {
    use anyhow::Result;
    use serde::{self, Deserialize, Deserializer, Serializer};
    use sqlx::types::time::Date;
    use time::format_description;

    pub fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        let s = date.format(&format).unwrap();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        let date = Date::parse(&s, &format).map_err(serde::de::Error::custom)?;
        Ok(date)
    }
}