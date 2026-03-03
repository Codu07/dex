//! Dex CLI - A simple agent framework

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dex::agent::{Agent, AgentBuilder};
use dex::config::{init_config, Config};
use dex::llm::create_provider;
use dex::tools::ToolRegistry;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(name = "dex")]
#[command(about = "A simple agent framework for CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Task to execute (if no subcommand)
    #[arg(value_name = "TASK")]
    task: Option<String>,

    /// Model to use
    #[arg(short, long, default_value = "gpt-4o-mini")]
    model: String,

    /// API key (or set OPENAI_API_KEY env var)
    #[arg(short, long)]
    api_key: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize configuration
    Init,
    
    /// Run in interactive chat mode
    Chat,
    
    /// Run a task from file
    Run {
        /// Path to task file
        #[arg(value_name = "FILE")]
        file: String,
    },
    
    /// List available tools
    Tools,
    
    /// Show configuration
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup tracing - output to both console and file
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("dex.log")
        .context("Failed to open dex.log")?;

    let log_level = if cli.verbose { "debug" } else { "info" };

    // Console layer (with colors)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_env_filter(log_level);

    // File layer (without colors, with timestamps)
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    match cli.command {
        Some(Commands::Init) => {
            init_config()?;
            return Ok(());
        }
        Some(Commands::Tools) => {
            list_tools().await?;
            return Ok(());
        }
        Some(Commands::Config) => {
            show_config().await?;
            return Ok(());
        }
        _ => {}
    }

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;
    
    // Get API key
    let api_key = cli.api_key
        .or_else(|| Some(config.agent.api_key.clone()))
        .filter(|k| !k.is_empty())
        .context("API key is required. Set OPENAI_API_KEY env var or use --api-key")?;

    // Create LLM provider
    let model = cli.model.clone();
    let api_base = config.agent.api_base.clone();
    
    info!("Initializing LLM provider with model: {}", model);
    
    let llm = create_provider(&api_key, &model, api_base)
        .context("Failed to create LLM provider")?;

    // Create tool registry
    let tool_registry = ToolRegistry::with_defaults();

    // Build agent
    let system_prompt = config.agent.system_prompt.clone().unwrap_or_else(|| {
        "You are a helpful AI assistant with access to tools. \
         When you need to perform actions like reading files or running commands, \
         use the available tools. Otherwise, provide direct answers.".to_string()
    });

    let mut agent = AgentBuilder::new()
        .with_llm(llm)
        .with_tools(tool_registry)
        .with_system_prompt(system_prompt)
        .build()
        .context("Failed to build agent")?;

    match cli.command {
        Some(Commands::Chat) => {
            run_interactive_chat(&mut agent).await?;
        }
        Some(Commands::Run { file }) => {
            run_task_file(&mut agent, &file).await?;
        }
        _ => {
            // Run single task from command line
            if let Some(task) = cli.task {
                println!("\n{} {}", "Task:".bold().blue(), task);
                println!("{}", "─".repeat(50).dimmed());
                
                match agent.chat(&task).await {
                    Ok(result) => {
                        println!("\n{}", "─".repeat(50).dimmed());
                        println!("{}", "Done!".green().bold());
                    }
                    Err(e) => {
                        error!("Task failed: {}", e);
                        eprintln!("\n{} {}", "Error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                // No task provided, show help
                run_interactive_chat(&mut agent).await?;
            }
        }
    }

    Ok(())
}

async fn run_interactive_chat(agent: &mut Agent) -> Result<()> {
    use rustyline::DefaultEditor;
    use colored::Colorize;

    println!("\n{}", "╔══════════════════════════════════════╗".cyan());
    println!("{}", "║        Dex Agent - Chat Mode         ║".cyan());
    println!("{}", "╚══════════════════════════════════════╝".cyan());
    println!("{}", "Type 'exit' or 'quit' to exit\n".dimmed());

    let mut rl = DefaultEditor::new()?;
    let prompt = "you> ".cyan().to_string();

    loop {
        match rl.readline(&prompt) {
            Ok(line) => {
                let input = line.trim();
                
                if input.is_empty() {
                    continue;
                }

                if input == "exit" || input == "quit" {
                    println!("{}", "Goodbye!".green());
                    break;
                }

                if input == "clear" {
                    agent.clear_history();
                    println!("{}", "Conversation history cleared.".yellow());
                    continue;
                }

                if input == "tools" {
                    let tools = agent.available_tools();
                    println!("\n{} {}", "Available tools:".yellow(), tools.join(", "));
                    continue;
                }

                rl.add_history_entry(input)?;

                println!("{} {}", "agent>".green(), "...".dimmed());
                
                match agent.chat(input).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("\n{} {}", "Error:".red().bold(), e);
                    }
                }
                
                println!();
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("\n{}", "Interrupted".yellow());
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("\n{}", "Goodbye!".green());
                break;
            }
            Err(err) => {
                eprintln!("{} {}", "Error:".red(), err);
                break;
            }
        }
    }

    Ok(())
}

async fn run_task_file(agent: &mut Agent, file: &str) -> Result<()> {
    let content = tokio::fs::read_to_string(file)
        .await
        .with_context(|| format!("Failed to read task file: {}", file))?;

    println!("\n{} {}", "Running task from:".bold().blue(), file);
    println!("{}", "─".repeat(50).dimmed());

    match agent.chat(&content).await {
        Ok(_) => {
            println!("\n{}", "─".repeat(50).dimmed());
            println!("{}", "Task completed!".green().bold());
        }
        Err(e) => {
            error!("Task failed: {}", e);
            eprintln!("\n{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn list_tools() -> Result<()> {
    let registry = ToolRegistry::with_defaults();
    
    println!("\n{}", "Available Tools:".bold().blue());
    println!("{}", "═".repeat(50).blue());
    
    for tool_def in registry.definitions() {
        println!("\n{} {}", "•".cyan(), tool_def.function.name.bold());
        println!("  {}", tool_def.function.description);
    }
    
    Ok(())
}

async fn show_config() -> Result<()> {
    match Config::load() {
        Ok(config) => {
            println!("\n{}", "Current Configuration:".bold().blue());
            println!("{}", "═".repeat(50).blue());
            println!("{}: {}", "Model".cyan(), config.agent.model);
            println!("{}: {}", "API Key".cyan(), 
                if config.agent.api_key.is_empty() { 
                    "<not set>".red() 
                } else { 
                    "<set>".green() 
                }
            );
            if let Some(base) = config.agent.api_base {
                println!("{}: {}", "API Base".cyan(), base);
            }
        }
        Err(e) => {
            println!("\n{}: {}", "No configuration found".yellow(), e);
            println!("\nRun {} to create default config.", "dex init".cyan());
        }
    }
    
    Ok(())
}
