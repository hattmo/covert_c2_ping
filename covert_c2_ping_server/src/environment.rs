use std::{env, path::PathBuf};

pub fn get_static_path() -> PathBuf {
    env::var_os("STATIC_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe().ok().map(|mut v| {
                v.pop();
                v.push("static");
                v
            })
        })
        .or_else(|| {
            env::current_dir().ok().map(|mut v| {
                v.push("static");
                v
            })
        })
        .unwrap()
}

pub fn get_artifact_path() -> PathBuf {
    env::var_os("ARTIFACT_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe().ok().map(|mut v| {
                v.pop();
                v.push("artifact");
                v
            })
        })
        .or_else(|| {
            env::current_dir().ok().map(|mut v| {
                v.push("artifact");
                v
            })
        })
        .unwrap()
}
