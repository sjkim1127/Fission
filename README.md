# Fission 🔬

> **"Split the Binary, Fuse the Power."**

**Fission** is a next-generation hybrid dynamic analysis platform that unifies the best features of x64dbg, Frida, Radare2, and Ghidra into a single Rust-powered binary.

## 🎯 Target Users

- Malware Analysts
- Vulnerability Researchers  
- Reverse Engineers

## ✨ Core Features

- **Hybrid Interface**: GPU-accelerated GUI (egui) + Radare2-style CLI running in perfect sync
- **Ghidra-Powered Analysis**: Sleigh engine integration for P-Code lifting (planned)
- **Python Scripting**: Inline hooking with full access to internal state via PyO3
- **Cross-Platform Debugging**: Windows (Debug API) and Linux (ptrace) support

## 🛠️ Tech Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | Rust 2021 | Memory safety, C++ performance |
| GUI | egui + wgpu | GPU-accelerated, immediate mode |
| CLI | reedline | Syntax highlighting, autocomplete |
| Disassembly | iced-x86 | Fastest x86/x64 decoder |
| Binary Parsing | goblin | PE/ELF/Mach-O support |
| Scripting | Python 3 (PyO3) | User-friendly automation |

## 🚀 Quick Start

```bash
# Build in release mode
cargo build --release

# Run with GUI
./target/release/fission

# Run in headless CLI mode
./target/release/fission --headless

# Load a target binary
./target/release/fission --target ./malware.exe
```

## 📁 Project Structure

```
Fission/
├── Cargo.toml              # Dependencies
├── PyFission/              # Python scripting module (Phase 3)
├── src/
│   ├── main.rs             # Entry point
│   ├── app.rs              # Application state
│   ├── core/               # Debugger backend
│   │   ├── debugger.rs     # OS Debug API wrapper
│   │   └── memory.rs       # Memory operations
│   ├── disasm/             # Disassembly layer
│   │   └── engine.rs       # iced-x86 wrapper
│   ├── script/             # Python integration
│   │   └── bridge.rs       # Rust <-> Python bridge
│   └── ui/                 # Interface layer
│       ├── cli.rs          # reedline REPL
│       └── gui.rs          # egui rendering
```

## 📅 Development Roadmap

- [x] **Phase 1**: CLI Base - Binary loader, disassembler, REPL
- [ ] **Phase 2**: GUI & Debug Loop - Attach, detach, breakpoints
- [ ] **Phase 3**: Ghidra & Scripting - P-Code analysis, Python API
- [ ] **Phase 4**: Advanced Features - Time travel debugging, plugins

## 📜 License

MIT License - See [LICENSE](LICENSE) for details.
