use std::{ffi::OsStr, fs::read_dir, io::Error, path::Path};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathError {
    #[error("Path does not exist")]
    PathDoesNotExist,
    #[error("Path provided is not a directory")]
    PathIsNotDirectory,
    #[error("failed trying to read files in the provided directory")]
    ReadDirError(Box<Error>),
    #[error("path was not a file")]
    PathIsNotFile,
}

pub fn get_song_file_paths_in_dir(dir: &String, valid_extensions: &Vec<String>) -> Result<Vec<String>, PathError> {
    println!("\nlooking for song files");
    // validate the directory exists
    let path = Path::new(&dir);
    if !path.exists() {
        return Err(PathError::PathDoesNotExist) }
    if !path.is_dir() {
        return Err(PathError::PathIsNotDirectory) }
    let file_paths = read_dir(path)
        .map_err(|e| PathError::ReadDirError(Box::new(e)))?;
    let file_paths: Vec<String> = file_paths.into_iter()
        .filter_map(|x| x
            .map_err(|e| println!("could not see file: {}", e))
            .ok())
        .filter(|entry| {
            let valid = has_valid_extension(&entry.path(), valid_extensions);
            if !valid {
                println!("- excluding {}", 
                    entry.file_name().into_string().unwrap_or("(unknown file)".to_string()))
            }
            valid
        })
        .map(|entry| entry.path()
            .to_str().expect("failed to convert a known path to string")
            .to_string())
        .collect();
    println!("\nfound the following files:");
    for file_path in file_paths.iter() {
        println!("- {}", strip_directory_from_file(&file_path).expect("we just built these."));
    }
    Ok(file_paths)
}

fn has_valid_extension(path: &Path, valid_extensions: &Vec<String>) -> bool {
    let extension = get_rid_of_osstr(path.extension());
    match extension {
        None => false,
        Some(ext) => valid_extensions.contains(&ext),
    }
}

pub fn strip_directory_from_file(file_name: &String) -> Result<String, PathError> {
    let path = Path::new(file_name);
    if !path.is_file() { return Err(PathError::PathIsNotFile) }
    get_rid_of_osstr(path.file_name()).ok_or(PathError::PathIsNotFile)
}

fn get_rid_of_osstr(osstr: Option<&OsStr>) -> Option<String> {
    osstr.and_then(OsStr::to_str).map(str::to_string)
}
