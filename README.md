# Rust-edu
[![Build Status](https://github.com/rust-edu/rust-edu.github.io/actions/workflows/main.yml/badge.svg)](https://github.com/rust-edu/rust-edu.github.io/actions)
[![Zulip](https://img.shields.io/badge/chat-on%20Zulip-9146FF?logo=zulip&logoColor=white)](https://rust-edu.zulip.cs.pdx.edu/)


Rust Edu is an organization dedicated to supporting Rust education in the academic environment. This website is built using [Zola](https://www.getzola.org/), a blazing-fast static site generator in Rust.

## 🚀 Setting Up on Local Machine
To contribute or run this website locally, follow the steps below.

To change only the content, see [docs/editing.md](docs/editing.md): every page is a Markdown file under `content/`.

### ✅ Requirements
- Zola **0.22.1** installed, the version CI builds and deploys with

## 📦 Installation Steps
### 1. Clone the Repository
```bash
git clone https://github.com/rust-edu/rust-edu.github.io.git
cd rust-edu.github.io
```

### 2. Install Zola 0.22.1
Download the binary for your system from [Zola 0.22.1](https://github.com/getzola/zola/releases/tag/v0.22.1),
or, with [Rust](https://www.rust-lang.org/tools/install) installed, build it with Cargo:
```bash
cargo install --locked --git https://github.com/getzola/zola --tag v0.22.1 zola
```

## 💻 Run Locally
Start a local development server:
```bash
zola serve
```
Your site will be live at: http://127.0.0.1:1111

Zola will auto-reload when changes are made.

## 🎨 Layout and Style
- `templates/`: the HTML layout of each kind of page (Zola's Tera templates)
- `static/style.css`: the whole stylesheet, plain CSS with its colors in one token block at the top
- `config.toml`: the site title and description, and the menu and footer links

## 📬 Get Involved
Interested in contributing or learning more?
* 🌐 Visit: [https://rust-edu.org/](https://rust-edu.org/)
* 📧 Email: [contact@rust-edu.org](mailto:contact@rust-edu.org)
Let’s build the future of Rust education together!