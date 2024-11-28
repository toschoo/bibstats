use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_bib_file(bib: &Option<OsString>) -> Result<OsString, String> {
    match bib {
        Some(file) => Ok(file.clone()),
        None => find_bib(),
    }
}

pub fn get_all_files(files: &Vec<OsString>, dirs: &Vec<OsString>) -> Result<Vec<OsString>, String> {
    let mut mainvec = files.clone();
    let mut v = get_files_from_dirs(dirs)?;
    mainvec.append(&mut v);
    Ok(mainvec)
}

fn find_bib() -> Result<OsString, String> {
    let p: OsString = ".".into();
    if let Ok(entries) = fs::read_dir(&p) {
        for entry in entries {
            if let Ok(entry) = entry {
                let fname = entry.file_name();
                match Path::new(&fname).extension() {
                    Some(ext) => {
                        if ext == "bib" {
                            return Ok(fname);
                        }
                    }
                    None => continue,
                }
            }
        }
    }
    Err("no bib file found in directory".to_string())
}

fn get_files_from_dirs(dirs: &Vec<OsString>) -> Result<Vec<OsString>, String> {
    let mut mainvec = Vec::new();

    for dir in dirs {
        let mut v = get_files_from_dir(dir)?;
        if !v.is_empty() {
            mainvec.append(&mut v);
        }
    }

    Ok(mainvec)
}

fn get_files_from_dir(dir: &OsString) -> Result<Vec<OsString>, String> {
    let mut v = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let fname = entry.file_name();
                match Path::new(&fname).extension() {
                    Some(ext) => {
                        if ext == "tex" {
                            // use extensions!
                            let p: PathBuf = [dir, &fname].iter().collect();
                            v.push(p.into_os_string());
                        }
                    }
                    None => continue,
                }
            }
        }
    }
    Ok(v)
}
