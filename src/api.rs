use crate::MusicUploaderClientConfig;
use reqwest::blocking::{Client, Response};
use music_uploader_server::model::{from_json, AlbumSearchResponse};

fn build_url(config: &MusicUploaderClientConfig, route: &str) -> String {
    format!("{}/{}", config.server_url, route)
}

pub fn check_conn(client: &Client,config: &MusicUploaderClientConfig) -> Result<(),String> {
    match client.get(build_url(config, "conn")).send() {
        Ok(x) => {
            if !x.status().is_success() {
                return Err(format!("status: {}", x.status()));
            }
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}

pub fn check_auth(client: &Client, config: &MusicUploaderClientConfig) -> Result<(),String> {
    match client.get(build_url(config, "auth"))
    .basic_auth(
        config.user.clone(),
        Some(config.password.clone()))
    .send() {
        Ok(x) => {
            if !x.status().is_success() {
                return Err(format!("status: {}", x.status()));
            }
            Ok(())
        }
        Err(e) => Err(format!("{:?}", e)),
    }
}

pub fn send_song(
    client: &Client,
    config: &MusicUploaderClientConfig,
    file: Vec<u8>,
    artist: &String,
    album: &String,
    song_file_name: &String
) -> Result<Response, String> {
    match client.post(build_url(config, "upload"))
        .header("file", song_file_name)
        .header("album", album)
        .header("artist", artist)
        .header("hash", sha256::digest(&file))
        .body(file)
        .basic_auth(
            config.user.clone(),
            Some(config.password.clone()))
        .send() {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(format!("bad status: {}", response.status()));
                }
                Ok(response)
            }
            Err(e) => {
                println!("sending failed, if source error is disconnected this is likely an authorization issue");
                println!("sending error: {:?}", e);
                Err(e.to_string())
            }
        }
}

pub fn album_search(
    client: &Client,
    config: &MusicUploaderClientConfig,
    album: &String,
) -> Result<Vec<String>, String> {
    let url = format!("albumsearch/{}", album);
    match client.get(build_url(config, url.as_str()))
        .basic_auth(config.user.clone(),
            Some(config.password.clone()))
        .send() {
            Ok(response) => {
                if !response.status().is_success() {
                    let status = response.status();
                    println!("bad response: {}", response.text().unwrap_or("<no body>".to_string()));
                    return Err(format!("bad status: {}", status))
                }
                let response_json = response.text().map_err(|e| e.to_string())?;
                let parsed_result = from_json::<AlbumSearchResponse>(&response_json)
                    .map_err(|e| e.to_string())?;
                Ok(parsed_result.albums)
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
}