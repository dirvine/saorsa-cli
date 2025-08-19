# Saorsa CLI Tools

A collection of powerful command-line tools for developers and system administrators.

## 🛠️ Available Tools

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

### Install All Tools
```bash
# Clone the repository
git clone https://github.com/dirvine/saorsa-cli
cd saorsa-cli

# Build and install all tools
cargo install --path sb
cargo install --path sdisk
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

# Run tests
cargo test --all

# Run with cargo
cargo run --bin sb
cargo run --bin sdisk
```

## 📦 Project Structure

```
saorsa-cli/
├── Cargo.toml          # Workspace configuration
├── README.md           # This file
├── sb/                 # Terminal Markdown Browser/Editor
│   ├── Cargo.toml
│   ├── src/
│   └── README.md
└── sdisk/              # Disk Usage Analyzer
    ├── Cargo.toml
    ├── src/
    └── README.md
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
- [sb README](./sb/README.md)
- [sdisk README](./sdisk/README.md)

## 🔮 Future Tools

This repository will continue to grow with more CLI tools. Stay tuned for:
- Network utilities
- Development helpers
- System monitoring tools
- And more!

---

*Part of the Saorsa ecosystem - Building tools for a better development experience*