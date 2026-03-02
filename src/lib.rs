//! Dex - A simple agent framework for Rust
//!
//! This crate provides a framework for building AI agents that can:
//! - Interact with LLMs (Large Language Models)
//! - Use tools to perform actions
//! - Maintain conversation context
//!
//! # Example
//!
//! ```rust,no_run
//! use dex::{Agent, AgentBuilder, ToolRegistry};
//! use dex::llm::create_provider;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create LLM provider
//!     let llm = create_provider(
//!         "your-api-key",
//!         "gpt-4o-mini",
//!         None,
//!     )?;
//!
//!     // Build agent
//!     let mut agent = AgentBuilder::new()
//!         .with_llm(llm)
//!         .with_tools(ToolRegistry::with_defaults())
//!         .build()?;
//!
//!     // Run a task
//!     let result = agent.chat("Hello!").await?;
//!     println!("{}", result);
//!
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod config;
pub mod llm;
pub mod tools;
pub mod types;

pub use agent::{Agent, AgentBuilder};
pub use config::Config;
pub use tools::ToolRegistry;
