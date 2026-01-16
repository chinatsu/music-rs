mod filters;
mod reads;
mod writes;

// Re-export public API
pub use reads::{
    get_albums, get_albums_for_artist, get_albums_for_genre, get_albums_for_mood, get_genre,
    get_mood, get_similar_genres, get_similar_moods,
};
pub use writes::register_album;
