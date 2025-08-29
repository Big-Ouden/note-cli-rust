<div align="center">
    <h1>Minimal CLI Note Manager in Rust</h1>
    <img alt="GitHub License" src="https://shieldsio.bigouden.org/github/license/Big-Ouden/note-cli-rust?style=for-the-badge">
    <img alt="GitHub repo size" src="https://shieldsio.bigouden.org/github/repo-size/Big-Ouden/note-cli-rust?style=for-the-badge"></img>
    <a href="https://belier.iiens.net"><img alt="Personal blog link" src="https://shieldsio.bigouden.org/badge/MY-BLOG-yellow?style=for-the-badge"></img></a>
    <img alt="GitHub last commit" src="https://shieldsio.bigouden.org/github/last-commit/Big-Ouden/note-cli-rust?display_timestamp=author&style=for-the-badge">
    <p>
    </p>
</div>



---

## Overview
`note-cli` is a lightweight command-line tool to manage notes.
Features include adding, listing, editing, tagging, and searching notes. Notes are stored in JSON format, with automatic ID management and optional tags.

---

## Features
- Add, remove, and edit notes
- Add tags to notes
- List notes with sorting options (`id`, `date`, `update`, `content`)
- Search notes by keyword
- Automatic ID reuse for deleted notes
- Pretty table output for easier reading

---

## 💾 Installation
1. Clone the repository:
```
git clone https://github.com/yourusername/note-cli.git
cd note-cli
```
2. Build the project:
```
cargo build --release
```
3. Run the binary:
```
./target/release/note-cli --help
```

---

## ⚙️ Usage

### Add a note
```
note-cli add "My first note" --tag personal --tag rust
```

### List notes
```
note-cli list --sort date
```

### Remove a note
```
note-cli remove 1
```

### Add tags to an existing note
```
note-cli add-tag 1 --tag important
```

### Edit a note
```
note-cli edit 1 --content "Updated note content"
```

### Search notes
```
note-cli search "keyword" --sort content
```

---

## Testing
```
cargo test
```

Tests cover:
- Adding notes
- Removing notes
- Editing content
- Tag management
- ID recycling
- Searching and sorting

---

## File Structure
- `notes.json` — Default storage file for notes
- `src/` — Source code
- `Cargo.toml` — Rust project configuration

---

---

> Contact: BigOuden - [bigouden.org](https://bigouden.org) - contact@bigouden.org

---

**License:** GNU GPLv3
