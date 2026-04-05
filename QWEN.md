# AI-RPA Project

## Project Overview

**ai-rpa** is a Rust-based RPA (Robotic Process Automation) library that leverages Windows UI Automation API to interact with desktop applications. The project provides a modular architecture for building automation tools that can manipulate UI elements like clicking, typing, and reading values from Windows applications.

### Key Features
- **UI Element Selection**: Flexible selector system supporting classname, control type, name, and composite (OR) conditions
- **Tool Architecture**: Unified `Tool` trait interface for implementing automation actions (click, type, etc.)
- **Execution Context**: Shared memory between automation steps with variables and execution logging
- **Windows Integration**: Built on the `uiautomation` crate for native Windows UI Automation API access

### Architecture

```
src/
├── lib.rs          # Library entry point, module exports
├── tool.rs         # Core Tool trait and ExecutionContext
├── selector.rs     # UI element selector system
├── click_tool.rs   # Click tool implementation
└── main1.rs        # Example/main entry point
```

**Core Components:**
- **`Tool` trait**: Interface all automation tools must implement (`name()`, `description()`, `execute()`)
- **`ExecutionContext`**: Shared state containing variables (`HashMap<String, serde_json::Value>`) and execution log
- **`Selector` enum**: Declarative UI element matching with multiple strategies
- **`ClickTool`**: Reference implementation of the Tool trait for clicking UI elements

## Technologies

- **Language**: Rust (Edition 2024)
- **Async Runtime**: Tokio
- **Serialization**: Serde + Serde JSON
- **UI Automation**: `uiautomation` v0.24.4 (Windows UI Automation API)
- **Error Handling**: Anyhow + ThisError
- **Logging**: Tracing + tracing-subscriber

## Building and Running

### Prerequisites
- Rust toolchain (edition 2024+)
- Windows OS (required for UI Automation)

### Commands

```bash
# Build the project
cargo build

# Run the main example (main1.rs)
cargo run

# Run a specific example
cargo run --example click_notepad

# Run tests
cargo test

# Check without building
cargo check
```

## Development Conventions

- **Language**: Code uses Russian comments and documentation
- **Error Handling**: Uses `anyhow::Result` for flexible error propagation
- **Module Structure**: Clean separation between core abstractions (`tool.rs`), selectors (`selector.rs`), and concrete implementations (`click_tool.rs`)
- **Trait-based Design**: `Tool` trait enables extensible tool system

## Known Issues

- `selector.rs` line 37: Incomplete method chain (`.classname(classname).` followed by `.scope(...)`) - missing method call
- `selector.rs` line 70: Match arm for `_` is unreachable due to non-exhaustive enum pattern
- `examples/click_notepad.rs` line 26: `println` missing `!` macro suffix

## Project Status

Early development (v0.1.0). The core architecture is in place with a working example in `main1.rs` that demonstrates Notepad automation. Some code in `selector.rs` appears incomplete and needs finishing.
