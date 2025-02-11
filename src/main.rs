use std::{env, fs::File, io::Read, path::Path};

use clap::Parser;
use music_uploader_client::{MusicUploaderClientConfig, MusicUploaderClient};
use serde::Deserialize;

#[derive(Deserialize)]
struct TomlConfig {
    user: String,
    password: String,
    valid_extensions: Vec<String>,
    server_url: String,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// albums artist. if there are spaces, wrap the name in single quotes. ex: 'taylor swuft'
    #[arg(short='r', long)]
    artist: String,
    /// name of the album. if there are spaces. if there are emojis, noone can help you.
    #[arg(short, long)]
    album: String,
    /// directory to the songs. ex: '/path/to/songs'
    #[arg(short, long)]
    dir: String,
}

fn main() {
    let args = Args::parse();
    let toml_config = get_toml_config();
    let lib_config = build_lib_config(toml_config);
    let client = MusicUploaderClient::new(lib_config);
    client.upload_album_dir(
        &args.album,
        &args.artist,
        &args.dir,
    ).map_err(|e| println!("upload_album error: {:?}", e))
    .expect("Failed to upload album");
}

const CLIENT_CONFIG_NAME: &str = "MusicUploaderClient.toml";
const CLIENT_CONFIG_BASE_PATH_KEY: &str = "MUSIC_UPLOADER_CONFIG_DIR";
fn get_toml_config() -> TomlConfig {
    let client_config_base_path = env::var(CLIENT_CONFIG_BASE_PATH_KEY)
        .unwrap_or(".".to_string());
    let config_path = Path::new(&client_config_base_path).join(CLIENT_CONFIG_NAME);
    let mut f = File::open(&config_path)
        .expect(&format!("Failed to find {}. Currently looking here ({}).  You can set the search directory with the env var {}",
            CLIENT_CONFIG_NAME, client_config_base_path, CLIENT_CONFIG_BASE_PATH_KEY));
    let mut file_text = String::new();
    f.read_to_string(&mut file_text)
        .expect(&format!("Failed to read contents of {}. idk what ths menas", CLIENT_CONFIG_NAME));
    let config = toml::from_str::<TomlConfig>(&file_text)
        .expect(&format!("Failed to parse contents of {}, probably typo", CLIENT_CONFIG_NAME));
    println!("succesfully found and parsed: {}", config_path.to_str().expect("config should be able to unwrap to str because we just built it."));
    config
}

fn build_lib_config(toml_config: TomlConfig) -> MusicUploaderClientConfig {
    MusicUploaderClientConfig {
        user: toml_config.user,
        password: toml_config.password,
        valid_extension: toml_config.valid_extensions,
        server_url: toml_config.server_url,
    }
}
