use std::ffi::OsStr;

use crate::evaluator::{FinishedShushCmd, ShushCmd};
use crate::timestamps;
use anyhow::anyhow;

#[derive(Debug)]
pub enum BuiltInCommands {
    CD,
}

impl BuiltInCommands {
    pub fn run<'a>(&'a self, cmd: &'a ShushCmd) -> anyhow::Result<FinishedShushCmd>{
        let exit = match self {
            Self::CD => change_dir(cmd),
        };

        match exit {
          Ok(true) => Ok(FinishedShushCmd::new(cmd, true, timestamps::get())),
          Ok(false) => Err(anyhow!("unknown")),
          Err(e) => Err(e),
        }
    }
}

fn change_dir(cmd: &ShushCmd) -> anyhow::Result<bool> {
    if cmd.n_args() > 1 {
        return Err(anyhow!("cd accepts one or no arguments"));
    };
    match cmd.args().first() {
        Some(arg) => {
            let path = std::path::Path::new(arg);
            std::env::set_var("OLDPWD", std::env::current_dir()?);
            std::env::set_current_dir(path)?;
            std::env::set_var("PWD", path);
            Ok(true)
        }
        None => {
            let home_path = std::env::var_os("HOME").unwrap();
            let path = std::path::Path::new::<OsStr>(home_path.as_ref());
            std::env::set_var("OLDPWD", std::env::current_dir()?);
            std::env::set_current_dir(path)?;
            std::env::set_var("PWD", path);
            Ok(true)
        }
    }
}
