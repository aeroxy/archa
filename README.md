<p align="center">
  <img src="frontend/public/logo.svg" alt="Archa Logo" width="120" />
</p>

<p align="center">
  <strong>Archa: Agentic CLI Session Chronicle</strong>
</p>

<p align="center">
  Browse, read, and export your Agent sessions through a high-fidelity academic interface.
</p>

---

**Archa** is a local-first explorer for your Agent conversation history. It transforms fragmented JSONL session data into a readable, searchable, and exportable chronicle.

## Features

- **Three-Column Layout**: Global COI switcher, Project Explorer, and high-precision Reader.
- **Project Discovery**: Automatically maps `~/.claude/projects` and extracts true project paths.
- **Academic Typography**: Designed for focus with Inter (UI), Newsreader (Serif Body), and Space Grotesk (Code).
- **Markdown Export**: Save any session as a clean Markdown file for your own records.
- **Single Binary**: The React frontend is embedded directly into the Rust backend.
- **Dynamic Port Selection**: Automatically finds an available port to avoid conflicts.

## Installation

### Via Homebrew (macOS)

```bash
brew install aeroxy/tap/archa
```

### From Source

```bash
# Requires Rust and Bun (or NPM)
git clone https://github.com/aeroxy/archa.git
cd archa
make build

# The binary will be at ./target/release/archa
```

## Usage

Simply run the binary:

```bash
archa
```

### Options

```bash
archa --help

# Usage: archa [OPTIONS]
#
# Options:
#   -p, --port <PORT>                 Port to listen on [default: 3000]
#   -P, --projects-path <PROJECTS_PATH>  Custom path to Claude projects
#   -h, --help                        Print help
#   -V, --version                     Print version
```

## Development

```bash
# Start both backend and frontend dev server with HMR
make dev
```

## License

MIT
