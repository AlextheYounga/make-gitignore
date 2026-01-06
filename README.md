# make-gitignore

A fast, interactive CLI tool for generating `.gitignore` files from GitHub's official [gitignore templates](https://github.com/github/gitignore).

## Features

- 🎨 **Interactive TUI** - Beautiful terminal UI with multi-select capabilities
- ⚡ **CLI Mode** - Pass languages as arguments for scripting and automation
- 🔄 **Smart Caching** - Downloads templates once, caches for 24 hours
- 🔀 **Multi-Language Support** - Combine multiple `.gitignore` templates with automatic deduplication
- 🔍 **Case-Insensitive** - Enter language names in any case (rust, Rust, RUST)
- 📦 **No Dependencies** - Self-contained binary, no runtime requirements

## Installation

### From Source

```bash
git clone <repository-url>
cd make-gitignore
cargo build --release
```

The binary will be available at `target/release/make-gitignore`.

### Install to PATH

```bash
cargo install --path .
```

## Usage

### Interactive Mode

Simply run the command without arguments to launch the interactive TUI:

```bash
make-gitignore
```

**Controls:**
- `↑/↓` or `j/k` - Navigate
- `Space` - Toggle selection
- `Enter` - Confirm and generate
- `Esc` or `q` - Cancel

### CLI Mode

Pass languages directly via the `--languages` flag:

```bash
make-gitignore --languages=Rust,Python,Node
```

**Features:**
- Comma-separated list of languages
- Case-insensitive matching
- Validates language names before generation

## Examples

### Single Language

```bash
# Interactive
make-gitignore
# Select "Python" → Creates .gitignore with Python rules

# CLI
make-gitignore --languages=Python
```

### Multiple Languages

```bash
# Interactive
make-gitignore
# Select "Rust", "Node", "VisualStudioCode" → Merges all templates

# CLI
make-gitignore --languages=Rust,Node,VisualStudioCode
```

### Case Insensitive

```bash
make-gitignore --languages=rust,PYTHON,node
# Matches: Rust, Python, Node
```

## How It Works

1. **Download** - Fetches the latest gitignore templates from GitHub (cached for 24 hours)
2. **Extract** - Unzips the archive to your cache directory
3. **Scan** - Indexes all `.gitignore` templates by language name
4. **Select** - Interactive UI or CLI arguments
5. **Generate** - Creates `.gitignore` in your current directory
   - **Single selection**: Copies the template directly
   - **Multiple selections**: Merges templates with deduplication

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
