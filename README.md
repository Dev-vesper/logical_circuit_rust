# Build

## Requirements

- Rust toolchain (stable)
- GTK 3 development headers

### Ubuntu / Debian

```bash
sudo apt install libgtk-3-dev pkg-config
```

### Fedora

```bash
sudo dnf install gtk3-devel
```

### macOS (Homebrew)

```bash
brew install gtk+3 pkg-config
```

## Build

```bash
cargo build --release
```

## Run the GUI

```bash
cargo run -p ui --bin ui_main
```

## Render example boards to PNG

```bash
cargo run -p ui --example ui_preview
```

Writes: `shapes.png`, `board.png`, `junction.png`, `shortened.png`, `moved.png`, `partial.png`.

## Run the logic example

```bash
cargo run -p logic --example majority
```

## Test

```bash
cargo test
```
