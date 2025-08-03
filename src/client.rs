use crate::{Error, Result, models::*};
use reqwest::Client;
use std::time::Duration;

const BASE_URL: &str = "https://monster-siren.hypergryph.com";
const USER_AGENT: &str = "msr-downloader/1.0.0";

pub struct MonsterSirenClient {
    client: Client,
    base_url: String,
}

impl MonsterSirenClient {
    pub fn new(version: Option<&str>) -> Result<Self> {
        let user_agent = match version {
            Some(v) => format!("msr-downloader/{}", v),
            None => USER_AGENT.to_string(),
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(1200))
            .user_agent(user_agent)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("Accept", "*/*".parse().unwrap());
                headers.insert(
                    "Accept-Language",
                    "zh-CN,zh;q=0.9,ja;q=0.8,en;q=0.7,en-GB;q=0.6,en-US;q=0.5"
                        .parse()
                        .unwrap(),
                );
                headers.insert(
                    "Referer",
                    "https://monster-siren.hypergryph.com/".parse().unwrap(),
                );
                headers
            })
            .build()?;

        Ok(Self {
            client,
            base_url: BASE_URL.to_string(),
        })
    }

    pub async fn get_songs(&self) -> Result<(Vec<Song>, String)> {
        let url = format!("{}/api/songs", self.base_url);
        let response: SongsResponse = self.client.get(&url).send().await?.json().await?;

        if response.code != 0 {
            return Err(Error::Api {
                message: response.msg,
            });
        }

        match response.data {
            Some(data) => Ok((data.list, data.autoplay)),
            None => Ok((Vec::new(), String::new())),
        }
    }

    pub async fn get_song(&self, song_id: &str) -> Result<Option<Song>> {
        let url = format!("{}/api/song/{}", self.base_url, song_id);
        let response: SongResponse = self.client.get(&url).send().await?.json().await?;

        if response.code != 0 {
            return Err(Error::Api {
                message: response.msg,
            });
        }

        Ok(response.data)
    }

    pub async fn get_albums(&self) -> Result<Vec<Album>> {
        let url = format!("{}/api/albums", self.base_url);
        let response: AlbumsResponse = self.client.get(&url).send().await?.json().await?;

        if response.code != 0 {
            return Err(Error::Api {
                message: response.msg,
            });
        }

        Ok(response.data.unwrap_or_default())
    }

    pub async fn get_album(&self, album_id: &str) -> Result<Option<Album>> {
        let url = format!("{}/api/album/{}/data", self.base_url, album_id);
        let response: AlbumResponse = self.client.get(&url).send().await?.json().await?;

        if response.code != 0 {
            return Err(Error::Api {
                message: response.msg,
            });
        }

        Ok(response.data)
    }

    pub async fn get_album_with_songs(&self, album_id: &str) -> Result<Option<Album>> {
        let url = format!("{}/api/album/{}/detail", self.base_url, album_id);
        let response: AlbumResponse = self.client.get(&url).send().await?.json().await?;

        if response.code != 0 {
            return Err(Error::Api {
                message: response.msg,
            });
        }

        Ok(response.data)
    }

    pub async fn download_file(&self, url: &str) -> Result<reqwest::Response> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(Error::Download(format!(
                "Failed to download from {}: HTTP {}",
                url,
                response.status()
            )));
        }

        Ok(response)
    }
}
