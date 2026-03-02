# Dex - A Simple Agent Framework for Rust

Dex is a lightweight, extensible agent framework for building AI-powered command-line tools. It allows you to create agents that can interact with LLMs and use tools to perform tasks.

## Features

- 🤖 **LLM Integration**: Works with OpenAI-compatible APIs (OpenAI, Azure, custom endpoints)
- 🛠️ **Tool System**: Extensible tool system with built-in file system and shell tools
- 💬 **Conversation Management**: Maintains conversation context across interactions
- ⚡ **Async**: Built on Tokio for high performance
- 🎨 **Interactive CLI**: Rich command-line interface with syntax highlighting
- 🔧 **Configurable**: Configuration via files or environment variables

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/Codu07/dex.git
cd dex

# Build the project
cargo build --release

# The binary will be at target/release/dex
```

### Configuration

1. Initialize configuration:
```bash
dex init
```

2. Edit `~/.config/dex/config.toml` to add your API key:
```toml
[agent]
model = "gpt-4o-mini"
api_key = "your-api-key-here"
```

Or set environment variable:
```bash
export OPENAI_API_KEY="your-api-key-here"
```

### Usage

#### Run a single task:
```bash
dex "List all files in the current directory"
```

#### Interactive chat mode:
```bash
dex chat
```

#### Run task from file:
```bash
dex run task.txt
```

#### List available tools:
```bash
dex tools
```

## Available Tools

### File System Tool (`fs`)
- `read`: Read file contents
- `write`: Write content to file
- `list`: List directory contents

### Shell Tool (`shell`)
- Execute shell commands (restricted to safe commands)
- Configurable allowed command list

## Architecture

```
dex/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── agent.rs         # Core agent logic
│   ├── types.rs         # Core types (Message, Tool, etc.)
│   ├── config.rs        # Configuration management
│   ├── llm/             # LLM module
│   │   ├── mod.rs
│   │   ├── provider.rs  # LLM provider trait
│   │   ├── client.rs    # HTTP client
│   │   └── openai.rs    # OpenAI implementation
│   └── tools/           # Tool system
│       ├── mod.rs
│       ├── executor.rs  # Tool execution
│       ├── fs.rs        # File system tool
│       └── shell.rs     # Shell tool
```

## Example

```rust
use dex::{Agent, AgentBuilder, ToolRegistry};
use dex::llm::create_provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create LLM provider
    let llm = create_provider(
        std::env::var("OPENAI_API_KEY")?,
        "gpt-4o-mini",
        None,
    )?;

    // Build agent with default tools
    let mut agent = AgentBuilder::new()
        .with_llm(llm)
        .with_tools(ToolRegistry::with_defaults())
        .with_system_prompt("You are a helpful coding assistant.")
        .build()?;

    // Run a task
    let response = agent.chat("Create a hello world Rust program").await?;
    println!("{}", response);

    Ok(())
}
```

## Creating Custom Tools

```rust
use dex::tools::ToolImplementation;
use dex::types::{Tool, ToolResult};
use async_trait::async_trait;

pub struct MyTool;

#[async_trait]
impl ToolImplementation for MyTool {
    fn definition(&self) -> Tool {
        Tool::new(
            "my_tool",
            "Description of my tool",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "arg": {"type": "string"}
                },
                "required": ["arg"]
            }),
        )
    }

    async fn execute(&self, args: &str) -> anyhow::Result<ToolResult> {
        // Implement tool logic
        Ok(ToolResult::Success("Done!".to_string()))
    }

    fn name(&self) -> &str {
        "my_tool"
    }
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | OpenAI API key |
| `DEX_API_KEY` | Alternative API key |
| `DEX_MODEL` | Default model to use |
| `DEX_API_BASE` | Custom API base URL |

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
