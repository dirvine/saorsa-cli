# Saorsa CLI Tools

A collection of powerful command-line tools for developers and system administrators.

## 🛠️ Available Tools

### saorsa - Interactive CLI Menu
An interactive menu system for all Saorsa tools with automatic binary management.

**Features:**
- 📱 Interactive terminal UI menu
- 🔄 Automatic binary downloads from GitHub releases
- 🖥️ Platform detection (macOS, Linux, Windows)
- 📦 Binary caching and version management
- ⚡ Direct tool execution mode

**Usage:**
```bash
# Interactive menu
saorsa

# Run tool directly
saorsa --run sb
saorsa --run sdisk
```

### sb - Terminal Markdown Browser/Editor
A terminal-based Markdown browser and editor with Git integration, syntax highlighting, and media support.

**Features:**
- 📝 Browse and edit Markdown files in the terminal
- 🎨 Syntax highlighting for code blocks
- 🖼️ Image and video preview support
- 🔄 Git integration with diff view
- ⌨️ Vim-like keybindings
- 🌲 File tree navigation

**Installation:**
```bash
cargo install sb
```

**Usage:**
```bash
# Browse current directory
sb

# Browse specific directory
sb /path/to/notes

# Open specific file
sb README.md
```

### sdisk - Disk Usage Analyzer
Cross-platform CLI tool to analyze disk usage and suggest safe cleanups.

**Features:**
- 📊 Analyze disk space usage
- 🧹 Identify safe files to clean
- 🔍 Find large files and directories
- 🗑️ Smart cleanup suggestions
- 📈 Progress indicators
- 🖥️ Cross-platform support

**Installation:**
```bash
cargo install sdisk
```

**Usage:**
```bash
# Analyze current directory
sdisk

# Analyze specific directory
sdisk /path/to/analyze

# Interactive cleanup mode
sdisk --interactive
```

## 🚀 Installation

### Quick Install (Recommended)
Download the latest release from [GitHub Releases](https://github.com/dirvine/saorsa-cli/releases):

1. Download the `cli-<platform>` archive for your system
2. Extract and run `saorsa` (or `saorsa.exe` on Windows)
3. The menu will automatically download other tools as needed

### Install All Tools from Source
```bash
# Clone the repository
git clone https://github.com/dirvine/saorsa-cli
cd saorsa-cli

# Build and install all tools
cargo install --path sb
cargo install --path sdisk
cargo install --path cli
```

### Install from crates.io
Each tool is also available individually on crates.io:

```bash
cargo install sb
cargo install sdisk
```

## 🔧 Building from Source

### Prerequisites
- Rust 1.70 or later
- Git

### Build Commands
```bash
# Clone the repository
git clone https://github.com/dirvine/saorsa-cli
cd saorsa-cli

# Build all tools
cargo build --release

# Build specific tool
cargo build --release -p sb
cargo build --release -p sdisk
cargo build --release -p cli

# Run tests
cargo test --all

# Run with cargo
cargo run --bin sb
cargo run --bin sdisk
cargo run --bin saorsa
```

## 📦 Project Structure

```
saorsa-cli/
├── Cargo.toml          # Workspace configuration
├── README.md           # This file
├── .github/
│   └── workflows/
│       ├── ci.yml      # Continuous Integration
│       └── release.yml # Release automation
├── cli/                # Interactive CLI Menu
│   ├── Cargo.toml
│   ├── src/
│   └── README.md
├── sb/                 # Terminal Markdown Browser/Editor
│   ├── Cargo.toml
│   ├── src/
│   └── README.md
├── sdisk/              # Disk Usage Analyzer
│   ├── Cargo.toml
│   ├── src/
│   └── README.md
└── scripts/
    └── create-release.sh # Release helper script
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is dual-licensed under:
- MIT License
- Apache License 2.0

You may choose either license for your use.

## 👤 Author

**David Irvine**
- GitHub: [@dirvine](https://github.com/dirvine)
- Email: david.irvine@saorsa.net

## 🌟 Acknowledgments

- Built with Rust 🦀
- Uses the amazing Ratatui TUI framework
- Powered by the Saorsa ecosystem

## 📚 More Information

For detailed documentation on each tool, see their respective README files:
- [CLI Menu README](./cli/README.md)
- [sb README](./sb/README.md)
- [sdisk README](./sdisk/README.md)

## 🚢 Releases

### Creating a Release

1. **Update version numbers** in `Cargo.toml` files
2. **Run the release script**:
   ```bash
   ./scripts/create-release.sh v0.1.0
   ```
3. **Push the tag** to trigger GitHub Actions:
   ```bash
   git push origin v0.1.0
   ```

GitHub Actions will automatically:
- Build binaries for all platforms
- Create individual and combined archives
- Generate SHA256 checksums
- Create a GitHub release with all assets

### Supported Platforms

Releases include pre-built binaries for:
- **macOS**: Apple Silicon (M1/M2) and Intel
- **Linux**: x86_64 and ARM64
- **Windows**: x86_64

## 🔮 Future Tools

This repository will continue to grow with more CLI tools. Stay tuned for:
- Network utilities
- Development helpers
- System monitoring tools
- And more!

---

*Part of the Saorsa ecosystem - Building tools for a better development experience*