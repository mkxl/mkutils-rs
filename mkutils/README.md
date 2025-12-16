# mkutils

A versatile Rust utility library providing extension traits, async utilities, and feature-gated functionality for common development tasks.

## Features

mkutils is designed with modularity in mind. All major functionality is gated behind optional features that can be enabled as needed:

### Core Features

- **`utils`** (always available) - The `Utils` trait providing 150+ extension methods for working with standard library types, futures, I/O, serialization, and more.
- **`active-vec`** (always available) - The `ActiveVec` type for managing a vector with a tracked "active" element.

### Optional Features

- **`async`** - Async utilities including `Event`, async I/O helpers, and stream operations. Requires tokio runtime.
- **`fmt`** - Display formatting helpers like `Debugged` and `OptionalDisplay`.
- **`fs`** - File system utilities with UTF-8 path support via camino.
- **`serde`** - Serialization helpers for JSON, YAML, and MessagePack.
- **`rmp`** - MessagePack serialization support.
- **`tui`** - Terminal UI utilities using ratatui, including `Terminal`, `Screen`, and geometry types like `Point`.
- **`process`** - The `Process` type for managing child processes with tokio.
- **`socket`** - Unix socket communication with the `Socket` type and `Request` trait.
- **`tracing`** - Integration with the tracing ecosystem via `Status` and `Tracing`.
- **`ropey`** - Text rope data structure support with `RopeBuilder`.
- **`reqwest`** - HTTP client utilities.
- **`poem`** - Web framework utilities for the Poem framework.
- **`output`** - The `Output` type for tri-state results (Ok/EndOk/EndErr).

## Installation

Add mkutils to your `Cargo.toml` with the features you need:

```toml
[dependencies]
mkutils = { version = "0.1", features = ["async", "serde", "tui"] }
```

## Usage Examples

### Utils Trait

The `Utils` trait is the heart of mkutils, providing extension methods on all types:

```rust
use mkutils::Utils;

// String manipulation
let greeting = "hello".cat(" world"); // "hello world"

// Convenient conversions
let opt = 42.some(); // Some(42)
let res = "value".ok::<String, ()>(); // Ok("value")

// Wrapping types
let arc_value = "data".arc(); // Arc<&str>
let boxed = vec![1, 2, 3].pin(); // Pin<Box<Vec<i32>>>

// Tuple operations
let pair = 1.pair("one"); // (1, "one")
let reversed = (1, 2).reversed(); // (2, 1)
```

### Async Utilities

```rust
use mkutils::{Event, Utils};

#[tokio::main]
async fn main() {
    // Event synchronization
    let mut event = Event::new();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        event.set();
    });

    event.wait().await; // Waits until set() is called

    // Async I/O utilities
    let content = "/path/to/file.txt".read_to_string_async().await?;

    // Stream operations
    let stream = tokio_stream::iter(vec![1, 2, 3]);
    let filtered = stream.filter_sync(|x| x % 2 == 0);
}
```

### Terminal UI

```rust
use mkutils::{Terminal, Point, Utils};

fn main() -> Result<(), std::io::Error> {
    let mut terminal = Terminal::new(Point::new(80, 24))?;

    terminal.set_title("My App")?;

    let output = terminal.draw(|frame| {
        // Use ratatui to draw to the frame
        Ok(())
    })?;

    // output contains the rendered bytes
    std::io::stdout().write_all(&output)?;
    Ok(())
}
```

### Process Management

```rust
use mkutils::{Process, Utils};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut process = Process::new(
        "ls",
        vec!["-la"],
        vec![],
        Some("/tmp"),
    )?;

    // Access stdin, stdout, stderr
    let stdout = process.stdout_mut();

    // Wait for completion
    let status = process.run().await?;
    Ok(())
}
```

### Socket Communication

```rust
use mkutils::{Socket, Request, Utils};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyRequest {
    data: String,
}

impl Request for MyRequest {
    type Response = String;
    type Serialized = Self;
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut socket = Socket::connect("/path/to/socket").await?;

    let response = socket.request(MyRequest {
        data: "hello".to_string()
    }).await?;

    println!("Response: {}", response);
    Ok(())
}
```

### Serialization

```rust
use mkutils::Utils;

// JSON serialization
let json_str = vec![1, 2, 3].to_json_str()?;
let value: Vec<i32> = json_str.as_bytes().to_value_from_json_slice()?;

// MessagePack (with 'rmp' feature)
let packed = data.to_rmp_byte_str()?;
let unpacked: MyType = packed.to_value_from_rmp_slice()?;
```

## Design Philosophy

mkutils embraces the extension trait pattern to provide ergonomic utilities that feel natural in Rust. The `Utils` trait is implemented for all types (`impl<T: ?Sized> Utils for T`), making its methods available everywhere while relying on type bounds to ensure they're only callable when appropriate.

This approach allows for:

- **Discoverability**: IDE autocomplete shows relevant methods directly on values
- **Composability**: Chain multiple operations together fluently
- **Type Safety**: Methods are only available when type constraints are satisfied
- **Zero Cost**: Extension trait methods inline just like regular methods

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the same terms as specified in the workspace `Cargo.toml`.

## Documentation

For detailed API documentation, run:

```bash
cargo doc --open --all-features
```
