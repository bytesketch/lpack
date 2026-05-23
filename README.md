# lpack

A lightweight Linux packaging system and portable package manager for distributing applications across distributions using a standard `.lpk` format.

---

## What is `lpack`?

`lpack` is a production-ready package tool built to simplify Linux application distribution.

It provides:

- A lightweight portable package format
- Easy package building
- Clean install/remove workflows
- Portable app deployment
- Minimal dependencies
- Manifest-driven packaging

The package format used is:

```text
.lpk
```

> NOTE: `.lpk` is a compressed archive with a small obfuscated manifest.

---

# Features

- Build portable `.lpk` packages
- Install packages locally or system-wide
- Remove installed packages cleanly
- Search installed packages
- Desktop entry support
- Symlink executable management
- Version-aware installation and upgrades
- Minimal Rust-based implementation

---

# How It Works

`lpack` packages are composed of:

1. A compressed archive
2. Application files and resources
3. A small obfuscated manifest for metadata

The builder:

- Reads `manifest.lpack`
- Collects application files
- Generates metadata
- Packages everything into `.lpk`

The installer:

- Extracts the package
- Reads the obfuscated manifest
- Installs files
- Creates executable symlinks
- Writes desktop entries

The remover:

- Removes installed files and directories
- Removes symlinks
- Removes desktop entries cleanly

---

# Installation

## Requirements

- Rust 1.70+ (stable)
- Linux

---

## Clone Repository

```bash
git clone https://github.com/bytesketch/lpack.git
cd lpack
```

---

## Build From Source

```bash
make build
```

The compiled binary is available at:

```text
target/release/lpack
```

---

## Install system-wide (sudo needed)

```bash
make install
```

---

# CLI Usage

## Build Package

```bash
lpack build # CWD
```

or

```bash
lpack build /path/to/project
```

### Silent Mode

```bash
lpack build . --silent
```

---

## Install Package

### User install

```bash
lpack install app.lpk
```

### System-wide install

```bash
sudo lpack install app.lpk --system-wide
```

### WARNING

Don't do

```bash
sudo lpack install app.lpk # NOTE: Here is no '--system-wide'
```

If done, remove immedietly by

```bash
sudo lpack search
sudo lpack remove package.app # NOTE: replace package.app with real package
```

---

## Remove Package

### User package

```bash
lpack remove my-package
```

### System-wide package

```bash
sudo lpack remove my-package --system-wide
```

---

## Search Installed Packages

### List packages

```bash
lpack search
```

### Inspect a package

```bash
lpack search my-package
```

> NOTE: To search system-wide installed packages, add `--system-wide`. The default scans local installations.

---

# Package Manifest

`manifest.lpack`

Example:

```json
{
  "package": {
    "name": "My App",
    "package": "my-app",
    "version": "1.0.0",
    "authors": ["yourname"]
  },

  "description": "Example lpack application",
  "build_path": "build",

  "app": {
    "binary": "bin/my-app",
    "entry": "myapp"
  },

  "desktop": {
    "name": "My App",
    "icon": "res/icon.png",
    "exec": "my-app --gui"
  },

  "include": {
    "README.md": "docs/README.md"
  }
}
```

---

# Goals

- Simplicity
- Minimalism
- Cross-distro portability
- Easy developer workflow
- Clean package management
- Fully scriptable

---

# Limitations

Current limitations include:

- No dependency resolution
- No remote repositories
- No package signing
- No sandboxing
- No delta updates
- No automatic updates
- No rollback support

---

# Tech Stack

- Rust
- Clap
- Serde
- Zip
- WalkDir

---

# Why I Built This

This project was built to simplify Linux application distribution with a portable packaging format.

It is designed as:

- A practical package manager
- A distro-agnostic packaging tool
- A Linux application distribution helper

---

# Examples

You can install `.lpk` packages locally or system-wide with:

```bash
sudo lpack install app.lpk --system-wide
```

---

# Contributing

Contributions, ideas, and fixes are welcome.

Feel free to:

- Open issues
- Suggest improvements
- Fork the project
- Test on different distros

---

# License

MIT License. See [LICENSE](LICENSE) for full info.

---

# Author

Ali Ahmad [@bytesketch](https://www.github.com/bytesketch)

Built with Rust and Linux tooling.
