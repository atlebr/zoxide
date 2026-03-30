# Alias Implementation Plan for zoxide (za command)

## Design Summary
Add a separate `za` command for managing and jumping to named directory aliases, with:
- Fallback to frecent `z` matching if no alias found
- Warning printed to stderr on fallback
- Support for bash, zsh, tcsh (csh best-effort)
- TSV storage format in `$_ZO_DATA_DIR/aliases.tsv`
- Both resolved and unresolved path storage options

## Files to Create

### 1. `src/cmd/alias.rs` (NEW)
**Purpose**: Main CLI command handler for alias operations

**Responsibilities**:
- Define `Alias` struct with subcommands (add/rm/list/jump)
- Implement `Run` trait for alias operations
- Handle path resolution via `--resolve` flag
- Path validation (exists check with warning pattern)
- Direct file I/O or delegate to new `AliasStore`

### 2. `src/alias/mod.rs` (NEW - new module)
**Purpose**: Alias store abstraction layer

**Content**:
- `AliasStore` struct: encapsulates aliases.tsv read/write
- Methods:
  - `new(data_dir: &Path) -> Result<Self>`
  - `load() -> Result<HashMap<String, PathBuf>>`
  - `save(aliases: &HashMap<String, PathBuf>) -> Result<()>`
  - `add(name: &str, path: PathBuf, resolve: bool) -> Result<()>`
  - `remove(name: &str) -> Result<()>`
  - `get(name: &str) -> Result<Option<PathBuf>>`
  - `list() -> Result<BTreeMap<String, PathBuf>>` (sorted for stable output)
  - `list_names() -> Result<Vec<String>>` (for completion)

**TSV Format** (aliases.tsv):
```
# Comments are supported (lines starting with #)
proj	/home/user/work/project
lib	/usr/local/lib
docs	/home/user/Documents
```

## Files to Modify

### 1. `src/cmd/cmd.rs`
**Add to `Cmd` enum**:
```rust
Alias(Alias),
```

**Add new struct** (or import from alias.rs):
```rust
/// Manage directory aliases
#[derive(Debug, Parser)]
pub struct Alias {
    #[clap(subcommand)]
    pub cmd: AliasCommand,
}

#[derive(Debug, Subcommand)]
pub enum AliasCommand {
    /// Add or update an alias
    Add {
        name: String,
        path: PathBuf,
        #[clap(long)]
        resolve: bool,
        #[clap(long, hide = true)]
        no_resolve: bool,
    },
    
    /// Remove an alias
    Rm { name: String },
    
    /// List all aliases
    List,
    
    /// Jump to an alias (internal, called by shell wrapper)
    #[clap(hide = true)]
    Jump { name: String },
    
    /// List alias names for completion
    #[clap(hide = true)]
    ListComplete,
}
```

### 2. `src/cmd/mod.rs`
**Add module**:
```rust
mod alias;
```

**Add to Run impl for Cmd**:
```rust
Cmd::Alias(cmd) => cmd.run(),
```

### 3. `src/main.rs`
**Add at module level**:
```rust
mod alias;
```

### 4. `src/config.rs`
**Add helper function**:
```rust
pub fn resolve_path(path: &Path, resolve_symlinks: bool) -> Result<PathBuf> {
    if resolve_symlinks {
        std::fs::canonicalize(path)
            .with_context(|| format!("could not resolve path: {}", path.display()))
    } else {
        Ok(path.to_path_buf())
    }
}
```

### 5. Shell Integration Files

#### `templates/bash.txt`
**Add after the `z` and `zi` function definitions**:
```bash
za() {
    if [[ "$#" -eq 0 ]]; then
        command zoxide alias list
    else
        command zoxide alias jump "$@" || command zoxide query --exclude "$(pwd)" "$@"
    fi
}
```

#### `templates/zsh.txt`
**Add after the `z` and `zi` function definitions**:
```zsh
za() {
    if [[ "$#" -eq 0 ]]; then
        command zoxide alias list
    else
        command zoxide alias jump "$@" || command zoxide query --exclude "$(pwd)" "$@"
    fi
}
```

#### `templates/tcsh.txt`
**Add after the `z` and `zi` alias definitions**:
```tcsh
alias za 'zoxide alias jump \!*'
```

Note: tcsh doesn't support functions easily; this is a simple wrapper.

### 6. `build.rs` (Shell Completions)
**Add completion for `zoxide alias` subcommand**

In the build script, when generating clap completions, the `Alias` and `AliasCommand` will be automatically included by clap_complete. No manual changes needed if clap_complete handles nested subcommands properly.

For bash/zsh, we may need to add custom completion that calls:
```bash
_zoxide_alias_complete() {
    local prefix="${COMP_WORDS[COMP_CWORD]}"
    IFS=$'\n' read -rd '' -a COMPREPLY < <(zoxide alias list-complete | grep "^${prefix}" | tr '\n' '\0')
}
```

## Implementation Steps (Sequential Order)

1. **Create `src/alias/mod.rs`**
   - Implement `AliasStore` struct with all CRUD operations
   - Handle TSV parsing and serialization
   - Add validation (name uniqueness, tab rejection)

2. **Create `src/cmd/alias.rs`**
   - Implement `Alias` and `AliasCommand` definitions
   - Implement `Run` trait for each subcommand
   - Handle fallback logic (jump → zoxide query)
   - Print warnings to stderr on fallback

3. **Modify `src/cmd/cmd.rs`**
   - Add `Alias` variant to `Cmd` enum
   - Add CLI structs/enums for alias command

4. **Modify `src/cmd/mod.rs`**
   - Add `mod alias` declaration
   - Add pattern match in `Run` impl

5. **Modify `src/main.rs`**
   - Add `mod alias` declaration

6. **Modify `src/config.rs`**
   - Add `resolve_path()` helper

7. **Update shell templates** (bash.txt, zsh.txt, tcsh.txt)
   - Add `za` function/alias wrapper

8. **Update `build.rs` for completions** (if needed)
   - Add custom completion logic or verify clap auto-generation

## Estimated Scope

- New files: ~400-500 lines (alias module + cmd handler)
- Modified files: ~50-100 total added lines
- Shell templates: ~10-15 lines per shell
- Build script: ~20-30 lines (completion setup)

## Testing Checklist

- [ ] `za add proj ~/work`
- [ ] `za proj` → jumps to alias
- [ ] `za nonexistent` → warns and falls back to `z` frecent
- [ ] `za list` → shows all aliases sorted
- [ ] `za rm proj`
- [ ] `za add --resolve symlink /some/symlink` → stores canonical path
- [ ] `za add --no-resolve path /some/path` → stores as-is
- [ ] Completion works in bash/zsh
- [ ] tcsh support (basic, no completion)

## Notes

- **Backward Compatibility**: No changes to zoxide database or `z`/`zi` behavior
- **Data Isolation**: Aliases stored separately from frecent DB
- **Shell Portability**: bash/zsh fully supported; tcsh via simple wrapper; csh not attempted
- **Fallback Safety**: If alias file is corrupted/missing, `za <name>` still works (falls back to `z`)
