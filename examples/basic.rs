//! Example: Basic agent usage

use dex::{AgentBuilder, ToolRegistry};
use dex::llm::create_provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable must be set");

    // Create LLM provider
    let llm = create_provider(api_key, "gpt-4o-mini", None)?;

    // Build agent
    let mut agent = AgentBuilder::new()
        .with_llm(llm)
        .with_tools(ToolRegistry::with_defaults())
        .with_system_prompt(
            "You are a helpful coding assistant. \
             You can read files, write code, and execute shell commands."
        )
        .build()?;

    // Example 1: Simple chat
    println!("=== Example 1: Simple Chat ===");
    let response = agent.chat("Hello! What can you do?").await?;
    println!("Response: {}\n", response);

    // Example 2: File operation task
    println!("=== Example 2: File Operations ===");
    let task = r#"
Create a simple Rust program that prints "Hello, World!" 
and save it to /tmp/hello.rs. Then show me the file contents.
"#;
    
    agent.chat(task).await?;

    // Example 3: Shell command
    println!("\n=== Example 3: Shell Commands ===");
    agent.chat("List the files in /tmp directory").await?;

    Ok(())
}
