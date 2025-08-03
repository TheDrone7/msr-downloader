use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub cid: String,
    pub name: String,
    #[serde(rename = "albumCid")]
    pub album_cid: Option<String>,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
    #[serde(rename = "lyricUrl")]
    pub lyric_url: Option<String>,
    #[serde(rename = "mvUrl")]
    pub mv_url: Option<String>,
    #[serde(rename = "mvCoverUrl")]
    pub mv_cover_url: Option<String>,
    pub artists: Option<Vec<String>>,
    pub artistes: Option<Vec<String>>,
}

impl Song {
    pub fn is_valid(&self) -> bool {
        !self.cid.is_empty()
    }

    pub fn sanitized_name(&self) -> String {
        crate::utils::sanitize_filename(&self.name)
    }

    pub fn get_artists(&self) -> Vec<String> {
        self.artists
            .as_ref()
            .or(self.artistes.as_ref())
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub cid: String,
    pub name: String,
    pub intro: Option<String>,
    pub belong: Option<String>,
    #[serde(rename = "coverUrl")]
    pub cover_url: Option<String>,
    #[serde(rename = "coverDeUrl")]
    pub cover_de_url: Option<String>,
    pub artistes: Option<Vec<String>>,
    pub songs: Option<Vec<Song>>,
}

impl Album {
    pub fn is_valid(&self) -> bool {
        !self.cid.is_empty()
    }

    pub fn sanitized_name(&self) -> String {
        let mut name = crate::utils::sanitize_filename(&self.name);
        if name.ends_with('.') {
            name = name.trim_end_matches('.').to_string() + "_";
        }
        name
    }

    pub fn get_artistes(&self) -> Vec<String> {
        self.artistes.clone().unwrap_or_default()
    }

    pub fn get_songs(&self) -> Vec<Song> {
        self.songs.clone().unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
pub struct SongsResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<SongsData>,
}

#[derive(Debug, Deserialize)]
pub struct SongsData {
    pub list: Vec<Song>,
    pub autoplay: String,
}

#[derive(Debug, Deserialize)]
pub struct SongResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<Song>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumsResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<Vec<Album>>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<Album>,
}
