use std::{fs, io};

use api::{album_search, check_auth, check_conn, send_song};
use path_utls::{get_song_file_paths_in_dir, strip_directory_from_file, PathError};
use reqwest::blocking::Client;
use thiserror::Error;

mod api;
mod path_utls;

pub struct MusicUploaderClientConfig {
    pub user: String,
    pub password: String,
    pub valid_extension: Vec<String>,
    pub server_url: String,
}

pub struct MusicUploaderClient {
    config: MusicUploaderClientConfig,
    http_client: Client,
}

impl MusicUploaderClient {
    pub fn new(config: MusicUploaderClientConfig) -> Self {
        MusicUploaderClient {
            config,
            http_client: Client::new(),
        }
    }

    pub fn check_conn(&self) -> Result<(), String> {
        check_conn(&self.http_client, &self.config)
    }

    pub fn check_auth(&self) -> Result<(), String> {
        check_auth(&self.http_client, &self.config)
    }

    pub fn upload_album_dir(
        &self,
        album: &String,
        artist: &String,
        dir: &String
    ) -> Result<Vec<Result<String, MusicUploaderError>>, MusicUploaderError> {
        let song_paths = get_song_file_paths_in_dir(dir, &self.config.valid_extension)
            .map_err(|e| MusicUploaderError::FailedToFindSongs(Box::new(e)))?;
        println!("\n# beginning uploads");
        let results = song_paths.into_iter()
            .map(|song_path| self.upload_song_file(album, artist, song_path))
            .collect();
        Ok(results)
    }

    fn upload_song_file(
        &self,
        album: &String,
        artist: &String,
        song_path: String
    ) -> Result<String, MusicUploaderError> {
        println!("\nuploading song: {}", strip_directory_from_file(&song_path).expect("we built this stirng"));
        let file = fs::read(&song_path)
            .map_err(|e| MusicUploaderError::FailedToReadFile(song_path.clone(), Box::new(e)))?;
        self.upload_song(
            album,
            artist,
            &strip_directory_from_file(&song_path)
                .map_err(|e| MusicUploaderError::FailedToParseSongFileName(song_path.clone(), Box::new(e)))?,
            file,
        )
    }

    pub fn upload_song(
        &self,
        album: &String,
        artist: &String,
        song_name: &String,
        song_data: Vec<u8>,
    ) -> Result<String, MusicUploaderError> {
        let result = send_song(
            &self.http_client,
            &self.config,
            song_data,
            artist,
            album,
            song_name,
        ).map_err(|e| MusicUploaderError::FailedToUpload(e))?;
        match result.status().is_success() {
            true => Ok(song_name.to_string()),
            false => Err(MusicUploaderError::UnhappyResponse(
                result.status().as_u16(),
                result.text().unwrap_or("<no body text>".to_string()))),
        }
    }

    pub fn album_search(
        &self,
        album: &String,
    ) -> Result<Vec<String>, MusicUploaderError> {
        album_search(
            &self.http_client,
            &self.config,
            album
        ).map_err(|e| {
            MusicUploaderError::Failed(e)
        })
    }
}

#[derive(Error, Debug)]
pub enum MusicUploaderError {
    #[error("Failed to find songs")]
    FailedToFindSongs(Box<PathError>),
    #[error("failed to parse song file name {0}")]
    FailedToParseSongFileName(String, Box<PathError>),
    #[error("failed to read file {0}")]
    FailedToReadFile(String, Box<io::Error>),
    #[error("uplaod failed: {0}")]
    FailedToUpload(String),
    #[error("unhappy response: ({0}) {1}")]
    UnhappyResponse(u16, String),
    #[error("i am being really lazy: {0}")]
    Failed(String),
}
