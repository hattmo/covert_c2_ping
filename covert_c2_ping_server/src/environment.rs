use std::{path::PathBuf, env};

pub fn get_static_path() -> PathBuf {
    env::var_os("STATIC_PATH")
        .and_then(|v| Some(PathBuf::from(v)))
        .or(env::current_exe().ok().and_then(|mut v| {
            v.pop();
            v.push("static");
            Some(v)
        }))
        .or(env::current_dir().ok().and_then(|mut v| {
            v.push("static");
            Some(v)
        }))
        .unwrap()
}

pub fn get_artifact_path() -> PathBuf {
    env::var_os("ARTIFACT_PATH")
        .and_then(|v| Some(PathBuf::from(v)))
        .or(env::current_exe().ok().and_then(|mut v| {
            v.pop();
            v.push("artifact");
            Some(v)
        }))
        .or(env::current_dir().ok().and_then(|mut v| {
            v.push("artifact");
            Some(v)
        }))
        .unwrap()
}
