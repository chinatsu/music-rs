use sqlx::{Postgres, QueryBuilder};

pub fn apply_genre_filter<'a>(builder: &mut QueryBuilder<'a, Postgres>, genres: &'a [String]) {
    if !genres.is_empty() {
        builder.push(" AND (SELECT COUNT(DISTINCT g2.name) FROM album_genres ag2 JOIN genres g2 ON ag2.genre_id = g2.id WHERE ag2.album_id = al.id AND g2.name = ANY(");
        builder.push_bind(genres);
        builder.push(")) = ");
        builder.push_bind(genres.len() as i64);
    }
}

pub fn apply_mood_filter<'a>(builder: &mut QueryBuilder<'a, Postgres>, moods: &'a [String]) {
    if !moods.is_empty() {
        builder.push(" AND (SELECT COUNT(DISTINCT m2.name) FROM album_moods am2 JOIN moods m2 ON am2.mood_id = m2.id WHERE am2.album_id = al.id AND m2.name = ANY(");
        builder.push_bind(moods);
        builder.push(")) = ");
        builder.push_bind(moods.len() as i64);
    }
}

pub fn apply_rating_filter(builder: &mut QueryBuilder<Postgres>, min_rating: Option<f64>) {
    if let Some(min_rating) = min_rating {
        builder.push(" AND al.score >= ");
        builder.push_bind(min_rating as f32);
    }
}

pub fn apply_date_range_filter(
    builder: &mut QueryBuilder<Postgres>,
    since: Option<chrono::NaiveDate>,
    to: Option<chrono::NaiveDate>,
) {
    if let Some(since) = since {
        builder.push(" AND al.date >= ");
        builder.push_bind(since);
    }
    if let Some(to) = to {
        builder.push(" AND al.date <= ");
        builder.push_bind(to);
    }
}

pub fn apply_url_filter(builder: &mut QueryBuilder<Postgres>, url: Option<String>) {
    if let Some(url) = url {
        builder.push(" AND al.url = '%");
        builder.push_bind(url);
        builder.push("%' ");
    }
}

pub fn apply_pagination(builder: &mut QueryBuilder<Postgres>, page: i64, limit: i64) {
    builder.push(" ORDER BY al.date desc, al.score desc LIMIT ");
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind((page - 1) * limit);
}
