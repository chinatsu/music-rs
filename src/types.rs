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
    pub moods: Option<Vec<Mood>>,
    pub tracks: Option<Vec<Track>>,
    pub url: String,
    pub rym_url: Option<String>,
    pub score: f32,
    pub voters: i32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct NewAlbum {
    pub album: String,
    pub artists: Vec<String>,
    #[serde(with = "my_date_format")]
    pub date: Date,
    pub genres: Vec<String>,
    pub moods: Vec<String>,
    pub tracks: Vec<Track>,
    pub url: String,
    pub rym_url: String,
    pub score: f32,
    pub voters: i32,
}

#[derive(Serialize, Deserialize)]
pub struct InsertedAlbum {
    pub id: Uuid,
    pub title: String,
    #[serde(with = "my_date_format")]
    pub date: Date,
    pub url: String,
    pub rym_url: Option<String>,
    pub score: f32,
    pub voters: i32,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Track {
    pub track_number: i32,
    pub title: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
pub struct Genre {
    pub id: Uuid,
    pub name: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
pub struct Mood {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimilarGenre {
    pub id: Uuid,
    pub name: Option<String>,
    pub count: Option<i64>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SimilarMood {
    pub id: Uuid,
    pub name: Option<String>,
    pub count: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenreInfo {
    pub genre: Genre,
    pub similar_genres: Vec<SimilarGenre>,
    pub albums: Vec<Album>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MoodInfo {
    pub mood: Mood,
    pub similar_moods: Vec<SimilarMood>,
    pub albums: Vec<Album>,
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
