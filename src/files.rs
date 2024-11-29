use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_bib_file(bib: &Option<OsString>) -> Result<OsString, String> {
    match bib {
        Some(file) => Ok(file.clone()),
        None => find_bib(),
    }
}

pub fn get_all_files(
    files: &Vec<OsString>,
    dirs: &Vec<OsString>,
    ext: &Vec<OsString>,
) -> Result<Vec<OsString>, String> {
    let mut v = files.clone();
    let extset: HashSet<OsString> = ext.clone().into_iter().collect();
    get_files_from_dirs(dirs, &extset, &mut v)?;
    Ok(v)
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

fn get_files_from_dirs(
    dirs: &Vec<OsString>,
    extset: &HashSet<OsString>,
    v: &mut Vec<OsString>,
) -> Result<(), String> {
    for dir in dirs {
        get_files_from_dir(dir, extset, v)?;
    }
    Ok(())
}

fn get_files_from_dir(
    dir: &OsString,
    extset: &HashSet<OsString>,
    v: &mut Vec<OsString>,
) -> Result<(), String> {
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let fname = entry.file_name();
                let p = Path::new(&fname);
                if p.is_dir() {
                    get_files_from_dir(&fname, extset, v)?;
                    continue;
                }
                match p.extension() {
                    Some(ext) => {
                        if extset.contains(ext) {
                            let p: PathBuf = [dir, &fname].iter().collect();
                            v.push(p.into_os_string());
                        }
                    }
                    None => continue,
                }
            }
        }
    }
    Ok(())
}
