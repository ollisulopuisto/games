# 🎮 WASM Games

A collection of self-contained, 60 FPS WebAssembly games built with Rust and Macroquad, deployed to GitHub Pages.

## 🕹 Play Online

**[Open the Games Portal →](https://ollisulopuisto.github.io/games/)**

| Game | Description | Play |
|------|-------------|------|
| **Bubbles** | Bubble shooter | [Play](https://ollisulopuisto.github.io/games/bubbles/) |
| **Gravitris** | Tetris with gravity wells | [Play](https://ollisulopuisto.github.io/games/gravitris/) |
| **Jetpac** | Jetpac clone | [Play](https://ollisulopuisto.github.io/games/jetpac/) |
| **Lumines** | Lumines-style puzzle game | [Play](https://ollisulopuisto.github.io/games/lumines/) |
| **VoxelDash** | 2.5D procedural voxel platformer | [Play](https://ollisulopuisto.github.io/games/voxeldash/) |
| **Zookeeper** | Zookeeper clone (match-3) | [Play](https://ollisulopuisto.github.io/games/zookeeper/) |
| **Kissagotchi** | Cat Tamagotchi | [Play](https://ollisulopuisto.github.io/games/kissagotchi/) |

## 🛠 Tech Stack

- **Language:** [Rust](https://www.rust-lang.org/)
- **Target:** WebAssembly (`wasm32-unknown-unknown`)
- **Game Engine:** [Macroquad](https://macroquad.rs/) (WebGL, 60 FPS)
- **Build:** [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) + GNU Make
- **Asset Tooling:** Python with [uv](https://github.com/astral-sh/uv)
- **Deployment:** GitHub Pages via GitHub Actions
- **Versioning:** [CalVer](https://calver.org/) (`YY.M.patch`)

## 📁 Repository Layout

```
.
├── Cargo.toml          # Root Cargo workspace (zookeeper, bubbles, lumines)
├── games_repo/         # WASM build workspace
│   ├── Cargo.toml      # Inner workspace
│   ├── Makefile        # Top-level build targets (setup / build / test)
│   ├── docs/           # Build output → served by GitHub Pages
│   └── games/
│       ├── shared/     # Shared Rust helpers for multiple games
│       ├── bubbles/
│       ├── gravitris/
│       ├── jetpac/
│       ├── lumines/
│       ├── kissagotchi/
│       ├── voxeldash/
│       └── zookeeper/
└── scripts/            # Project-wide utility scripts
```

## 🚀 Building Locally

### Prerequisites

- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [uv](https://github.com/astral-sh/uv) (Python toolchain manager)
- [`wasm-opt`](https://github.com/WebAssembly/binaryen) (optional, for size optimisation)

### Commands

```bash
cd games_repo

# Install tooling dependencies
make setup

# Build all games into docs/
make build

# Run the test suite
make test
```

## 🤝 Contributing

1. Branch off `main` using a descriptive prefix (`feat/`, `fix/`, `refactor/`).
2. Make your changes and run `make test` locally.
3. Open a pull request — CI must pass before merging.
4. Prefer *Squash and merge* to keep the `main` history clean.

## 📄 License

MIT / Public Domain
