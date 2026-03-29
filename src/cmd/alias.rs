use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::alias::AliasStore;
use crate::cmd::{Alias, AliasCommand, Run};
use crate::config;

impl Run for Alias {
    fn run(&self) -> Result<()> {
        match &self.cmd {
            AliasCommand::Add { name, path, resolve } => Self::cmd_add(name, path, *resolve),
            AliasCommand::Rm { name } => Self::cmd_rm(name),
            AliasCommand::List => Self::cmd_list(),
            AliasCommand::Jump { name } => Self::cmd_jump(name),
            AliasCommand::ListComplete => Self::cmd_list_complete(),
        }
    }
}

impl Alias {
    fn cmd_add(name: &str, path: &PathBuf, resolve: bool) -> Result<()> {
        let data_dir = config::data_dir()?;
        let mut store = AliasStore::new(&data_dir)?;

        store.add(name, path.clone(), resolve)?;
        store.save()?;

        println!("Added alias: {} -> {}", name, path.display());
        Ok(())
    }

    fn cmd_rm(name: &str) -> Result<()> {
        let data_dir = config::data_dir()?;
        let mut store = AliasStore::new(&data_dir)?;

        if store.remove(name)? {
            store.save()?;
            println!("Removed alias: {}", name);
            Ok(())
        } else {
            bail!("alias not found: {}", name);
        }
    }

    fn cmd_list() -> Result<()> {
        let data_dir = config::data_dir()?;
        let store = AliasStore::new(&data_dir)?;

        let aliases = store.list();
        if aliases.is_empty() {
            println!("No aliases defined.");
            return Ok(());
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        for (name, path) in aliases {
            writeln!(handle, "{}\t{}", name, path.display())?;
        }

        Ok(())
    }

    fn cmd_jump(name: &str) -> Result<()> {
        let data_dir = config::data_dir()?;
        let store = AliasStore::new(&data_dir)?;

        match store.get(name) {
            Some(path) => {
                print!("{}", path.display());
                Ok(())
            }
            None => {
                // Print warning to stderr and return error for shell to handle fallback
                eprintln!("warning: alias \"{}\" not found; using zoxide match", name);
                // Use exit code to signal that shell should fallback to z command
                bail!(crate::error::SilentExit { code: 1 });
            }
        }
    }

    fn cmd_list_complete() -> Result<()> {
        let data_dir = config::data_dir()?;
        let store = AliasStore::new(&data_dir)?;

        for name in store.list_names() {
            println!("{}", name);
        }

        Ok(())
    }
}
