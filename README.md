# Gitignore CLI from Github's Official Repository

A fast, interactive CLI tool for generating `.gitignore` files from GitHub's official [gitignore templates](https://github.com/github/gitignore).

Inspired by [gitignore-it](https://github.com/christopherkade/gitignore-it)

## Features

- 🎨 **Interactive TUI** - Beautiful terminal UI with multi-select capabilities
- ⚡ **CLI Mode** - Pass languages as arguments for scripting and automation
- 🔄 **Smart Caching** - Downloads templates once, caches for 24 hours
- 🔀 **Multi-Language Support** - Append multiple `.gitignore` templates together (preserves template formatting)
- 🔍 **Case-Insensitive** - Enter language names in any case (rust, Rust, RUST)
- 📦 **No Dependencies** - Self-contained binary, no runtime requirements
- 🧹 **General Ignores** - Always appends common OS/temp/log/editor junk patterns

## Installation

### Install from [crates.io](https://crates.io/crates/make-gitignore)

```bash
cargo add gitignore
```

### Install to PATH

```bash
cargo install --git https://github.com/AlextheYounga/make-gitignore make-gitignore
```


### From Source

```bash
git clone https://github.com/AlextheYounga/make-gitignore
cd make-gitignore
cargo build --release
```

The binary will be available at `target/release/gitignore`.

## Usage

### Interactive Mode

Simply run the command without arguments to launch the interactive TUI:

```bash
gitignore
```

**Controls:**
- `↑/↓` or `j/k` - Navigate
- `Space` - Toggle selection
- `Enter` - Confirm and generate
- `Esc` or `q` - Cancel

### CLI Mode

Pass languages directly via the `--languages` flag:

```bash
gitignore --languages=rust,python,node
```

**Features:**
- Comma-separated list of languages
- Case-insensitive matching
- Validates language names before generation

## Examples

### Single Language

```bash
# Interactive
gitignore
# Select "Python" → Creates .gitignore with Python rules

# CLI
gitignore --languages=python
```

### Multiple Languages

```bash
# Interactive
gitignore
# Select "Rust", "Node", "VisualStudioCode" → Merges all templates

# CLI
gitignore --languages=rust,node,visualstudiocode
```

### Case Insensitive

```bash
gitignore --languages=rust,PYTHON,node
# Matches: Rust, Python, Node
```

## How It Works

1. **Download** - Fetches the latest gitignore templates from GitHub (cached for 24 hours)
2. **Extract** - Unzips the archive to your cache directory
3. **Scan** - Indexes all `.gitignore` templates by language name
4. **Select** - Interactive UI or CLI arguments
5. **Generate** - Creates `.gitignore` in your current directory
   - **Single selection**: Appends the template
   - **Multiple selections**: Appends templates in order
   - Always appends a small set of general ignore patterns (e.g. `.DS_Store`, `*.sqlite`)

## Cache Location

Templates are cached in your system's cache directory:
- **Linux**: `~/.cache/make-gitignore/`
- **macOS**: `~/Library/Caches/make-gitignore/`
- **Windows**: `%LOCALAPPDATA%\make-gitignore\`

## Requirements

- Rust 1.70+ (for building)
- Internet connection (first run or after 24h cache expiration)

## Available Templates

The tool includes all templates from GitHub's official repository, including:
- Programming languages (Rust, Python, JavaScript, Go, Java, C++, etc.)
- Frameworks (Node, Rails, Django, etc.)
- IDEs (VisualStudioCode, IntelliJ, Vim, etc.)
- Operating systems (macOS, Linux, Windows)

Run without `--languages` to see all available options in the interactive UI.

## Safety Features

- ⚠️ **Prevents overwriting** - Refuses to replace existing `.gitignore` files
- ✅ **Validates languages** - Checks all language names before generation
- 🔒 **Safe caching** - Atomic downloads with timestamp verification

## License

MIT

## Contributing

Issues and pull requests welcome!
