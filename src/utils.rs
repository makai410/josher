use std::{ffi::{OsStr, OsString}, path::PathBuf};

use directories::ProjectDirs;
use uuid::Uuid;

pub fn install_dir() -> PathBuf {
    let Some(prj_dirs) = ProjectDirs::from("dev", "makai410", "josher") else {
        eprintln!("Oops!!! Fallback");
        return PathBuf::from(".josher").join("cargo");
    };
    prj_dirs.data_local_dir().join("cargo")
}

pub fn collect_args<I, S>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .collect()
}

pub fn gen_runid() -> String {
    Uuid::now_v7().simple().to_string()
}
