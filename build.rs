use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
};

use npm_rs::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = {
        let fnm =
            std::env::home_dir().map(|home| home.join(".local/share/fnm/aliases/default/bin"));

        let mut path: Vec<OsString> = Vec::new();
        path.push("/bin".into());
        path.push("/usr/bin".into());
        path.push("/usr/local/bin".into());
        if let Some(fnm_path) = fnm {
            if fnm_path.exists() {
                path.push(fnm_path.into());
            }
        }

        path.join(OsStr::from_bytes(":".as_bytes()))
    };

    let exit_status = NpmEnv::default()
        .with_node_env(&NodeEnv::from_cargo_profile().unwrap_or_default())
        .set_path(concat!(env!("CARGO_MANIFEST_DIR"), "/web"))
        .with_env("PATH", path)
        .init_env()
        .custom("install", Some(&["--include", "dev"]))
        .run("build")
        .exec()?;
    if exit_status.success() {
        Ok(())
    } else {
        Err("npm build failed".into())
    }
}
