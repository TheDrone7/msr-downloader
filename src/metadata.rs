use crate::{
    Error, Result,
    models::{Album, Song},
};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::{Tag, TagExt};
use std::path::Path;

pub struct MetadataWriter;

impl MetadataWriter {
    pub fn new() -> Self {
        Self
    }

    pub async fn write_metadata(
        &self,
        file_path: &Path,
        song: &Song,
        album: &Album,
        track_number: u32,
        total_tracks: u32,
        album_cover_path: Option<&Path>,
    ) -> Result<()> {
        let mut tagged_file = Probe::open(file_path)
            .map_err(|e| Error::File(format!("Failed to probe audio file: {}", e)))?
            .read()
            .map_err(|e| Error::File(format!("Failed to read audio file: {}", e)))?;

        let tag_type = tagged_file.primary_tag_type();
        if tagged_file.tag(tag_type).is_none() {
            tagged_file.insert_tag(Tag::new(tag_type));
        }

        let tag = tagged_file.tag_mut(tag_type).unwrap();
        tag.clear();

        tag.set_title(song.name.clone());
        tag.set_album(album.name.clone());

        let artists = song.get_artists();
        if !artists.is_empty() {
            tag.set_artist(artists.join(", "));
        }

        tag.set_track(track_number);
        tag.set_track_total(total_tracks);

        if let Some(intro) = &album.intro {
            tag.set_comment(intro.clone());
        }

        if let Some(belong) = &album.belong {
            match belong.as_str() {
                "arknights" => tag.set_genre("Arknights".to_string()),
                _ => tag.set_genre("Unknown Genre".to_string()),
            }
        }

        if let Some(cover_path) = album_cover_path {
            if let Ok(cover_data) = std::fs::read(cover_path) {
                let mime_type = self.get_image_mime_type(cover_path);
                let picture = Picture::new_unchecked(
                    PictureType::CoverFront,
                    Some(mime_type),
                    None,
                    cover_data,
                );
                tag.set_picture(0, picture);
            }
        }

        tagged_file
            .save_to_path(file_path, Default::default())
            .map_err(|e| Error::File(format!("Failed to save metadata: {}", e)))?;

        Ok(())
    }

    fn get_image_mime_type(&self, image_path: &Path) -> MimeType {
        match image_path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => MimeType::Jpeg,
            Some("png") => MimeType::Png,
            Some("gif") => MimeType::Gif,
            Some("bmp") => MimeType::Bmp,
            _ => MimeType::Jpeg,
        }
    }
}
