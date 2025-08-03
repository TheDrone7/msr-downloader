use crate::{
    Result,
    client::MonsterSirenClient,
    metadata::MetadataWriter,
    models::{Album, Song},
    progress::ProgressTracker,
    utils,
};
use futures::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

const SAVE_DIR: &str = "./Monster Siren Records";
const MAX_CONCURRENT_DOWNLOADS: usize = 5;

pub struct Downloader {
    client: MonsterSirenClient,
    progress: ProgressTracker,
    metadata_writer: MetadataWriter,
    save_path: PathBuf,
}

impl Downloader {
    pub fn new(client: MonsterSirenClient) -> Self {
        Self {
            client,
            progress: ProgressTracker::new(),
            metadata_writer: MetadataWriter::new(),
            save_path: PathBuf::from(SAVE_DIR),
        }
    }

    pub async fn download_all_tracks(&self) -> Result<()> {
        utils::ensure_dir_exists(&self.save_path).await?;

        let albums = self.client.get_albums().await?;
        let total_albums = albums.len();

        self.progress
            .println(&format!("Found {} albums to download", total_albums));

        let main_progress = self.progress.create_progress_bar(
            total_albums as u64,
            &format!(
                "Downloading Monster Siren Records library, {} albums",
                total_albums
            ),
        );

        for (album_index, album_basic) in albums.iter().enumerate() {
            let album_no = total_albums - album_index;

            let album = match self.client.get_album_with_songs(&album_basic.cid).await? {
                Some(mut album) => {
                    if album.artistes.is_none() && album_basic.artistes.is_some() {
                        album.artistes = album_basic.artistes.clone();
                    }
                    album
                }
                None => {
                    self.progress
                        .println(&utils::format_failure_message(&format!(
                            "⚠️  Cannot get details for album: [{}] {}",
                            album_basic.cid, album_basic.name
                        )));
                    main_progress.inc(1);
                    continue;
                }
            };

            let songs = self.get_detailed_songs(&album).await;
            let album_with_songs = Album {
                songs: Some(songs),
                ..album
            };

            self.progress.set_pinned_message(&format!(
                "{}: downloading album tracks",
                utils::format_album_name(&album_with_songs.name)
            ));

            let album_dir_name = format!("{:03} - {}", album_no, album_with_songs.sanitized_name());
            let album_path = self.save_path.join(album_dir_name);
            utils::ensure_dir_exists(&album_path).await?;

            self.save_album_info(&album_with_songs, &album_path).await?;

            self.download_album_covers(&album_with_songs, &album_path)
                .await?;

            self.download_album_songs(&album_with_songs, &album_path)
                .await?;

            self.apply_metadata_to_songs(&album_with_songs, &album_path)
                .await?;

            self.progress
                .println(&utils::format_success_message(&format!(
                    "✅  {}",
                    utils::format_album_name(&album_with_songs.name)
                )));
            main_progress.inc(1);
        }

        main_progress.finish_with_message("Download completed!");
        self.progress.remove_progress_bar(&main_progress);
        Ok(())
    }

    async fn get_detailed_songs(&self, album: &Album) -> Vec<Song> {
        let songs = album.get_songs();
        let mut detailed_songs = Vec::new();

        for song in songs {
            match self.client.get_song(&song.cid).await {
                Ok(Some(detailed_song)) => detailed_songs.push(detailed_song),
                Ok(None) => {
                    self.progress
                        .println(&utils::format_failure_message(&format!(
                            "⚠️  Song not found: {}",
                            song.name
                        )));
                    detailed_songs.push(song);
                }
                Err(e) => {
                    self.progress
                        .println(&utils::format_failure_message(&format!(
                            "⚠️  Failed to get song details for {}: {}",
                            song.name, e
                        )));
                    detailed_songs.push(song);
                }
            }
        }

        detailed_songs
    }

    async fn save_album_info(&self, album: &Album, album_path: &Path) -> Result<()> {
        let info_path = album_path.join("info.txt");

        if utils::file_exists(&info_path) {
            return Ok(());
        }

        let mut content = String::new();
        content.push_str(&format!("Album Name: {}\n", album.name));

        if let Some(belong) = &album.belong {
            content.push_str(&format!("Album Belongs To: {}\n", belong));
        }

        let artistes = album.get_artistes();
        if !artistes.is_empty() {
            content.push_str(&format!("Album Artists: {}\n", artistes.join(", ")));
        }

        if let Some(intro) = &album.intro {
            content.push_str(&format!("Album Introduction:\n{}\n\n", intro));
        }

        content.push_str("Track List:\n");

        let songs = album.get_songs();
        for (index, song) in songs.iter().enumerate() {
            let track_no = index + 1;
            if !song.is_valid() {
                content.push_str(&format!(
                    "- {:02}. {}\n",
                    track_no, "<unknown: missing data>"
                ));
                continue;
            }

            content.push_str(&format!("- {:02}. {}\n", track_no, song.name));

            let artists = song.get_artists();
            if !artists.is_empty() {
                content.push_str(&format!("  Artists: {}\n", artists.join(", ")));
            }
        }

        tokio::fs::write(info_path, content.trim()).await?;
        Ok(())
    }

    async fn download_album_songs(&self, album: &Album, album_path: &Path) -> Result<()> {
        let songs = album.get_songs();
        let valid_songs: Vec<_> = songs
            .iter()
            .enumerate()
            .filter(|(_, song)| song.is_valid())
            .collect();

        if valid_songs.is_empty() {
            return Ok(());
        }

        let song_progress = self.progress.create_progress_bar(
            valid_songs.len() as u64,
            &format!(
                "{}: downloading {} tracks",
                utils::format_album_name(&album.name),
                valid_songs.len()
            ),
        );

        stream::iter(valid_songs)
            .map(|(index, song)| {
                let track_no = index + 1;
                self.download_song(song, track_no, album_path, &song_progress)
            })
            .buffer_unordered(MAX_CONCURRENT_DOWNLOADS)
            .collect::<Vec<_>>()
            .await;

        song_progress.finish_with_message("Track downloads completed");
        self.progress.remove_progress_bar(&song_progress);
        Ok(())
    }

    async fn apply_metadata_to_songs(&self, album: &Album, album_path: &Path) -> Result<()> {
        let songs = album.get_songs();
        let valid_songs: Vec<_> = songs
            .iter()
            .enumerate()
            .filter(|(_, song)| song.is_valid())
            .collect();

        if valid_songs.is_empty() {
            return Ok(());
        }

        self.progress.set_pinned_message(&format!(
            "{}: applying metadata to tracks",
            utils::format_album_name(&album.name)
        ));

        let total_tracks = valid_songs.len();
        let cover_path = self.find_album_cover(album_path);

        for (index, song) in valid_songs {
            let track_no = index + 1;
            let song_name = song.sanitized_name();

            if song.source_url.is_some() {
                let ext = utils::get_file_extension(song.source_url.as_ref().unwrap())
                    .unwrap_or_else(|| ".mp3".to_string());
                let filename = format!("{:02}.{}{}", track_no, song_name, ext);
                let file_path = album_path.join(&filename);

                if utils::file_exists(&file_path) {
                    if let Err(e) = self
                        .metadata_writer
                        .write_metadata(
                            &file_path,
                            song,
                            album,
                            track_no as u32,
                            total_tracks as u32,
                            cover_path.as_deref(),
                        )
                        .await
                    {
                        eprintln!("Failed to apply metadata to {}: {}", filename, e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn download_song(
        &self,
        song: &Song,
        track_no: usize,
        album_path: &Path,
        progress: &indicatif::ProgressBar,
    ) -> Result<()> {
        let song_name = song.sanitized_name();

        if let Some(source_url) = &song.source_url {
            let ext = utils::get_file_extension(source_url).unwrap_or_else(|| ".mp3".to_string());
            let filename = format!("{:02}.{}{}", track_no, song_name, ext);
            self.download_file(source_url, album_path, &filename)
                .await?;
        }

        if let Some(lyric_url) = &song.lyric_url {
            let filename = format!("{:02}.{}.lrc", track_no, song_name);
            self.download_file(lyric_url, album_path, &filename).await?;
        }

        progress.inc(1);
        Ok(())
    }

    fn find_album_cover(&self, album_path: &Path) -> Option<PathBuf> {
        let cover_names = [
            "Album Cover.jpg",
            "Album Cover.png",
            "Cover.jpg",
            "Cover.png",
        ];

        for name in &cover_names {
            let cover_path = album_path.join(name);
            if utils::file_exists(&cover_path) {
                return Some(cover_path);
            }
        }
        None
    }

    async fn download_album_covers(&self, album: &Album, album_path: &Path) -> Result<()> {
        if let Some(cover_url) = &album.cover_url {
            self.progress.set_pinned_message(&format!(
                "{}: downloading album cover",
                utils::format_album_name(&album.name)
            ));
            let ext = utils::get_file_extension(cover_url).unwrap_or_else(|| ".jpg".to_string());
            let filename = format!("Album Cover{}", ext);
            self.download_file(cover_url, album_path, &filename).await?;
        }

        if let Some(cover_de_url) = &album.cover_de_url {
            self.progress.set_pinned_message(&format!(
                "{}: downloading detailed cover",
                utils::format_album_name(&album.name)
            ));
            let ext = utils::get_file_extension(cover_de_url).unwrap_or_else(|| ".jpg".to_string());
            let filename = format!("Cover{}", ext);
            self.download_file(cover_de_url, album_path, &filename)
                .await?;
        }

        Ok(())
    }

    async fn download_file(&self, url: &str, dir_path: &Path, filename: &str) -> Result<()> {
        let file_path = dir_path.join(filename);

        if utils::file_exists(&file_path) {
            return Ok(());
        }

        let temp_path = file_path.with_extension(format!(
            "{}.tmp",
            file_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("tmp")
        ));

        if utils::file_exists(&temp_path) {
            let _ = tokio::fs::remove_file(&temp_path).await;
        }

        let response = self.client.download_file(url).await?;
        let mut file = tokio::fs::File::create(&temp_path).await?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        drop(file);

        tokio::fs::rename(temp_path, file_path).await?;
        Ok(())
    }
}
