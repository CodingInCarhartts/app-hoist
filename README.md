<div align="center">

# 🚀 App Hoist

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/app-hoist.svg)](https://crates.io/crates/app-hoist)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A Rust CLI tool for hoisting and managing applications across different environments with Docker, project, and package support**

*Lift your applications with ease - from development to deployment*

[Installation](#-installation) •
[Usage](#-usage) •
[Modes](#-modes) •
[Templates](#-templates) •
[Cache](#-cache) •
[License](#-license)

</div>

---

## 📖 Overview

App Hoist is a powerful command-line tool built in Rust that simplifies the process of managing and deploying applications across various environments. Whether you're working with individual packages, complex projects, Docker containers, or multiple projects simultaneously, App Hoist provides a unified interface to "hoist" your applications to the next level.

## ✨ Features

| Feature | Description |
|---------|-------------|
| 📦 **Package Mode** | Hoist individual executables and packages |
| 🏗️ **Project Mode** | Manage Python, Go, Rust, and JavaScript/TypeScript projects |
| 🐳 **Docker Support** | Direct Docker operations and Docker-enabled project management |
| 🔄 **Multi-Project** | Parallel operations on multiple projects |
| 🎯 **Interactive Mode** | User-friendly menu-driven interface |
| 📋 **Template System** | Create and manage project templates |
| 💾 **Caching** | Intelligent caching for improved performance |
| ⚡ **Async Operations** | Concurrent processing for efficiency |

## 📦 Installation

### From Crates.io
```bash
cargo install app-hoist
```

### From Source
```bash
git clone https://github.com/CodingInCarhartts/app-hoist
cd app-hoist
cargo build --release
# Binary will be available at target/release/app-hoist
```

## 🚀 Usage

App Hoist supports multiple operational modes. Here are the most common use cases:

### Package Mode
Hoist a specific package/executable:
```bash
app-hoist --package my-executable
```

### Project Mode
Manage a project directory:
```bash
app-hoist --path /path/to/project
```

### Docker Mode
Execute Docker commands directly:
```bash
app-hoist --docker "run hello-world"
```

Manage Docker-enabled projects:
```bash
app-hoist --docker-path /path/to/docker-project
```

### Multi-Project Mode
Operate on multiple projects in parallel:
```bash
app-hoist --multi-path /path/to/project1 /path/to/project2 /path/to/project3
```

### Interactive Mode
Run without arguments for an interactive menu:
```bash
app-hoist
```

## 🎯 Modes

### Package Mode (`--package`)
- Hoists individual executables
- Supports dry-run with `--dry-run`
- Automatic dependency resolution

### Project Mode (`--path`)
- Supports Python, Go, Rust, JavaScript/TypeScript
- Project structure analysis
- Environment setup and management

### Docker Modes
- **Direct Docker** (`--docker`): Execute raw Docker commands
- **Docker Project** (`--docker-path`): Manage containerized projects
- Full Docker CLI compatibility

### Multi-Project Mode (`--multi-path`)
- Parallel processing of multiple projects
- Progress indicators with `indicatif`
- Error aggregation and reporting

### Interactive Mode
- Menu-driven interface using `inquire`
- Guided setup and configuration
- Beginner-friendly

## 📋 Templates

App Hoist includes a powerful template system for project scaffolding:

### List Available Templates
```bash
app-hoist template list
```

### Initialize Project from Template
```bash
app-hoist template init <template-name> <target-directory>
```

### Create New Template
```bash
app-hoist template create <template-name> <source-directory>
```

### Search Templates
```bash
app-hoist template search <query>
```

### Built-in Templates
- **svelte-ts-bun**: SvelteKit project with TypeScript and Bun
- Custom templates can be created from existing projects

## 💾 Cache Management

### View Cache Statistics
```bash
app-hoist cache stats
```

### Clear All Cache
```bash
app-hoist cache clear
```

### Invalidate Specific Path
```bash
app-hoist cache invalidate /path/to/project
```

## 🛠️ Development

### Prerequisites
- Rust 1.70+
- Cargo

### Building
```bash
cargo build
```

### Testing
```bash
cargo test
```

### Running
```bash
cargo run -- [arguments]
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Lint: `cargo clippy`
7. Submit a pull request

## 📜 License

[MIT License](LICENSE) - See LICENSE file for details.

---

<div align="center">
  <p>Built with ❤️ in Rust</p>
</div>