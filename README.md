# Rust-edu
[![Build Status](https://github.com/rust-edu/rust-edu.github.io/actions/workflows/main.yml/badge.svg)](https://github.com/rust-edu/rust-edu.github.io/actions)
[![Zulip](https://img.shields.io/badge/chat-on%20Zulip-9146FF?logo=zulip&logoColor=white)](https://rust-edu.zulip.cs.pdx.edu/)


Rust Edu is an organization dedicated to supporting Rust education in the academic environment. This website is built using [Zola](https://www.getzola.org/), a blazing-fast static site generator in Rust.

## 🚀 Setting Up on Local Machine
To contribute or run this website locally, follow the steps below.

### ✅ Requirements
- [Rust](https://www.rust-lang.org/tools/install) installed
- Zola (`cargo install zola`) installed

## 📦 Installation Steps
### 1. Clone the Repository
```bash
git clone https://github.com/rust-edu/rust-edu.github.io.git
cd rust-edu.github.io
```

### 2. Install Zola
Install Zola using Cargo:
``` bash
cargo install zola
```
If that fails, download a binary from [Zola Releases](https://github.com/getzola/zola/releases).

### 3. Initialize Submodules
To load the external CSS themes (Bulma & Bulmaswatch):
```bash
git submodule update --init --recursive
```
## 💻 Run Locally
Start a local development server:
```bash
zola serve
```
Your site will be live at: http://127.0.0.1:1111

Zola will auto-reload when changes are made.

## 🐛 Common Issues
### ❌ `Can't find stylesheet to import` error?
Ensure submodules are initialized:
```bash
git submodule update --init --recursive
```
### ❌ `Variable config.extra.navbar not found`?
Ensure your `config.toml` includes a `[extra]` section:
```toml
[extra]
navbar = [
  { name = "Home", url = "/" },
  { name = "News", url = "/news" },,
  { name = "Resources", url = "/resources" }
]

```
## 📬 Get Involved
Interested in contributing or learning more?
* 🌐 Visit: [https://rust-edu.org/](https://rust-edu.org/)
* 📧 Email: [contact@rust-edu.org](mailto:contact@rust-edu.org)
Let’s build the future of Rust education together!