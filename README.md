# 💀 Phobos IDE

<div align="center">

![Phobos Logo](https://img.shields.io/badge/Phobos-IDE-red?style=for-the-badge&logo=rust)
![License](https://img.shields.io/badge/License-AGPL--3.0-blue?style=for-the-badge)
![Rust](https://img.shields.io/badge/Built_with-Rust-orange?style=for-the-badge&logo=rust)
![Status](https://img.shields.io/badge/Status-Alpha-red?style=for-the-badge)

> **"The God of Fear rises. The True Successor to Delphi."**

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Supported Languages](#-supported-languages) • [Roadmap](#-roadmap) • [Contributing](#-contributing)

</div>

---

## 🌟 Introduction

**Phobos** is a next-generation, **polyglot Integrated Development Environment (IDE)** built from the ground up with **Rust** for performance, safety, and modern development workflows. 

Inspired by the legendary **Delphi IDE**, Phobos brings back the rapid application development experience while embracing modern technologies, AI-powered assistance, and support for **25+ programming languages**.

Whether you're building low-level system software, web applications, or AI-driven solutions, Phobos provides the tools you need in a single, blazing-fast environment.

---

## ⚡ Why Choose Phobos?

| Feature | Legacy IDEs | **Phobos** |
|---------|-------------|------------|
| **Performance** | Slow, memory-heavy | ⚡ Blazing fast (Rust-native) |
| **Language Support** | Limited | 🌍 25+ languages (Polyglot) |
| **License** | Proprietary ($$$) | 🔓 Open Source (AGPL-3.0) |
| **AI Integration** | None/Plugin | 🤖 Built-in AI Assistant |
| **Low-Level Tools** | External tools | 🔧 Hex Editor, Disassembler included |
| **Modern UI** | Outdated | 🎨 Beautiful (egui-based) |
| **Community** | Declining | 🚀 Growing rapidly |

---

## 🚀 Features

### 📝 Code Editor
- **Multi-cursor editing** for efficient code manipulation
- **Syntax highlighting** for 25+ languages
- **Bracket matching** and auto-closing
- **Auto-indentation** with language-aware rules
- **Line numbers** and minimap for easy navigation
- **Code folding** for better organization
- **Find & Replace** with regex support

### 🎨 User Interface
- **Modern, responsive UI** built with egui
- **Dark theme** optimized for long coding sessions
- **Zen mode** for distraction-free coding
- **Customizable layouts** and panels
- **Tab management** for multiple files
- **File explorer** with tree view
- **Integrated terminal** for command-line access

### 🤖 AI-Powered Tools
- **Code Explainer**: Understand complex code instantly
- **Bug Finder**: Detect potential issues before runtime
- **Ghost Typer**: AI-assisted code completion
- **Smart Refactoring**: Optimize your code automatically

### 🛠️ Advanced Features
- **Built-in Git integration** for version control
- **Project builder** with customizable build configurations
- **Debugger** for step-by-step execution
- **Code runner** with output preview
- **Search panel** across entire projects

### 🔧 Low-Level Development
- **Hex Editor** for binary file inspection
- **Disassembler** for reverse engineering
- **Memory Viewer** for debugging
- **Crypto utilities** for security testing

### 🌐 Polyglot Support
Write code in multiple languages within the same project:
- **Systems**: Rust, C, C++, Assembly
- **Web**: JavaScript, TypeScript, HTML, CSS
- **Backend**: Python, Go, Ruby, PHP
- **Mobile**: Swift, Kotlin, Dart
- **Data**: SQL, R, Julia
- **And more**: Java, C#, Pascal, Lua, Perl, Shell, YAML, JSON, Markdown

---

## 📦 Installation

### Prerequisites
- **Rust** (1.70 or later) - [Install Rust](https://rustup.rs/)
- **Git** for cloning the repository

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/phobos.git
cd phobos

# Build in release mode for optimal performance
cargo build --release

# Run Phobos
./target/release/phobos
```

### Quick Start (Development Mode)

```bash
# Clone and run immediately
git clone https://github.com/yourusername/phobos.git
cd phobos
cargo run
```

### Package Managers (Coming Soon)
```bash
# Windows (winget)
winget install phobos-ide

# macOS (brew)
brew install phobos-ide

# Linux (snap)
sudo snap install phobos-ide
```

---

## 💻 Usage

### Starting a New Project

1. Launch Phobos IDE
2. Click **File → New Project**
3. Select your primary language
4. Choose a project template
5. Start coding!

### Opening Existing Projects

```bash
phobos /path/to/your/project
```

Or use the GUI:
- **File → Open Folder**
- Select your project directory
- Phobos will automatically detect languages and configure settings

### Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| New File | `Ctrl+N` |
| Open File | `Ctrl+O` |
| Save | `Ctrl+S` |
| Save All | `Ctrl+Shift+S` |
| Find | `Ctrl+F` |
| Replace | `Ctrl+H` |
| Toggle Terminal | `` Ctrl+` `` |
| Zen Mode | `F11` |
| Build Project | `Ctrl+B` |
| Run Project | `F5` |
| Debug | `Shift+F5` |

---

## 🌍 Supported Languages

Phobos provides first-class support for:

### Systems Programming
- 🦀 **Rust** - Full LSP support
-  **C/C++** - Syntax highlighting & compilation
-  **Assembly** (x86, ARM) - With disassembler

### Web Development
-  **JavaScript/TypeScript** - Modern ES6+ features
-  **HTML5** - Live preview
-  **CSS/SASS** - Auto-completion

### Backend & Scripting
- 🐍 **Python** - 3.x support
-  **Go** - Fast compilation
-  **Ruby** - Rails-ready
-  **PHP** - Web development

### Mobile & Cross-Platform
-  **Swift** - iOS development
-  **Kotlin** - Android development
-  **Dart** - Flutter support

### Data Science & Analytics
-  **SQL** - Database queries
-  **R** - Statistical computing
-  **Julia** - High-performance analytics

### Enterprise & Legacy
- ☕ **Java** - JVM ecosystem
-  **C#** - .NET compatibility
-  **Pascal/Delphi** - Honoring our roots

### Other Languages
-  **Lua** - Game scripting
-  **Perl** - Text processing
-  **Shell/Bash** - System automation
-  **YAML/JSON** - Configuration
-  **Markdown** - Documentation

---

## 🗺️ Roadmap

### ✅ Completed (v0.1 Alpha)
- [x] Core editor with syntax highlighting
- [x] Basic UI framework
- [x] Multi-language support structure
- [x] AI module architecture
- [x] Low-level tools foundation

### 🔄 In Progress (v0.2 Beta)
- [ ] Complete hex editor implementation
- [ ] Full debugger integration
- [ ] Enhanced AI features
- [ ] Plugin system
- [ ] Performance optimizations

### 📅 Planned (v0.3+)
- [ ] Marketplace for extensions
- [ ] Cloud sync for settings
- [ ] Collaborative coding features
- [ ] Mobile companion app
- [ ] Official stable release (v1.0)

---

## 🤝 Contributing

We welcome contributions from the community! Here's how you can help:

### Getting Started
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Areas We Need Help
- 📝 **Documentation**: Improve guides and examples
- 🧪 **Testing**: Write unit and integration tests
- 🎨 **UI/UX**: Enhance the user interface
- 🌍 **Translations**: Localize Phobos for different languages
- 🔌 **Plugins**: Build extensions for new languages/tools
- 🐛 **Bug Fixes**: Report and fix issues

### Code of Conduct
Please be respectful and inclusive. We're building a community where everyone feels welcome.

---

## 📄 License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. 

See the [LICENSE](LICENSE) file for details.

**Why AGPL?** We believe in keeping Phobos free and open-source forever. Any modifications to Phobos must also be open-sourced, ensuring the community benefits from improvements.

---

## 🙏 Acknowledgments

- **Anders Hejlsberg** - For creating Delphi and inspiring this project
- **egui team** - For the amazing immediate-mode GUI library
- **tree-sitter** - For powerful parsing capabilities
- **Rust community** - For continuous support and innovation

---

## 📬 Contact & Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/phobos/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/phobos/discussions)
- **Email**: contact@phobos-ide.dev (coming soon)
- **Discord**: Join our community server (coming soon)

---

<div align="center">

### 💀 *"The God of Fear rises. Microsoft should be afraid."*

**Built with ❤️ using Rust**

[⬆ Back to Top](#-phobos-ide)

</div>
