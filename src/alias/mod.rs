use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};

/// Manages directory aliases stored in a TSV file.
pub struct AliasStore {
    path: PathBuf,
    aliases: BTreeMap<String, PathBuf>,
}

impl AliasStore {
    /// Creates a new AliasStore pointing to the aliases.tsv file in data_dir.
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let path = data_dir.as_ref().join("aliases.tsv");
        let aliases = Self::load_file(&path).unwrap_or_default();
        Ok(AliasStore { path, aliases })
    }

    /// Loads aliases from the TSV file. Returns empty map if file doesn't exist.
    fn load_file(path: &Path) -> Result<BTreeMap<String, PathBuf>> {
        let mut aliases = BTreeMap::new();

        if !path.exists() {
            return Ok(aliases);
        }

        let file = fs::File::open(path)
            .with_context(|| format!("could not open aliases file: {}", path.display()))?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.with_context(|| {
                format!("could not read line {} from aliases file", line_num + 1)
            })?;

            // Skip empty lines and comments
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Parse TSV format: name<TAB>path
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 2 {
                eprintln!(
                    "warning: skipping malformed line {} in aliases file (expected 2 fields, got {})",
                    line_num + 1,
                    parts.len()
                );
                continue;
            }

            let name = parts[0].to_string();
            let path = PathBuf::from(parts[1]);

            // Validate name (no tabs, non-empty)
            if name.is_empty() || name.contains('\t') {
                eprintln!(
                    "warning: skipping invalid alias name on line {}: '{}'",
                    line_num + 1,
                    name
                );
                continue;
            }

            aliases.insert(name, path);
        }

        Ok(aliases)
    }

    /// Saves aliases to the TSV file.
    pub fn save(&self) -> Result<()> {
        let mut file = fs::File::create(&self.path)
            .with_context(|| format!("could not create aliases file: {}", self.path.display()))?;

        writeln!(file, "# Directory aliases (auto-generated)")?;
        writeln!(file, "# Format: name\\tpath")?;
        writeln!(file)?;

        for (name, path) in &self.aliases {
            let path_str = path.to_string_lossy();
            writeln!(file, "{}\t{}", name, path_str)?;
        }

        Ok(())
    }

    /// Reloads aliases from disk.
    pub fn reload(&mut self) -> Result<()> {
        self.aliases = Self::load_file(&self.path).unwrap_or_default();
        Ok(())
    }

    /// Adds or updates an alias.
    pub fn add(&mut self, name: &str, path: PathBuf, resolve: bool) -> Result<()> {
        // Validate name
        if name.is_empty() {
            bail!("alias name cannot be empty");
        }
        if name.contains('\t') {
            bail!("alias name cannot contain tabs");
        }

        // Check if path exists (warn but allow)
        if !path.exists() {
            eprintln!("warning: path does not exist: {}", path.display());
        }

        // Resolve symlinks if requested
        let final_path = if resolve {
            fs::canonicalize(&path).with_context(|| {
                format!("could not resolve path: {}", path.display())
            })?
        } else {
            path
        };

        self.aliases.insert(name.to_string(), final_path);
        Ok(())
    }

    /// Removes an alias.
    pub fn remove(&mut self, name: &str) -> Result<bool> {
        Ok(self.aliases.remove(name).is_some())
    }

    /// Gets a single alias.
    pub fn get(&self, name: &str) -> Option<&PathBuf> {
        self.aliases.get(name)
    }

    /// Returns all aliases as a sorted map.
    pub fn list(&self) -> &BTreeMap<String, PathBuf> {
        &self.aliases
    }

    /// Returns a list of all alias names (for completion).
    pub fn list_names(&self) -> Vec<String> {
        self.aliases.keys().cloned().collect()
    }

    /// Checks if an alias exists.
    pub fn exists(&self, name: &str) -> bool {
        self.aliases.contains_key(name)
    }
}
