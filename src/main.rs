mod cli;
mod agent;
mod graph;
mod llm;
mod memory;
mod tools;
mod tui;
mod types;
mod config;
mod models;

use clap::Parser;
use env_logger::Env;
use log::info;
use tokio::signal;

use crate::cli::commands::Commands;
use crate::models::server::Provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Commands::parse();

    match cli.command {
        crate::cli::commands::Cmd::Run { goal } => {
            info!("Starting agent run: {}", goal);
            let mut sa = agent::super_agent::SuperAgent::new();
            sa.run_goal(goal).await?;
        }
        crate::cli::commands::Cmd::Chat => {
            println!("chat mode not implemented yet; start TUI with `agent tui` or use `agent run --goal`.");
        }
        crate::cli::commands::Cmd::Graph => {
            println!("Showing graph (text mode):");
            let sa = agent::super_agent::SuperAgent::new();
            sa.graph.print();
        }
        crate::cli::commands::Cmd::Logs => {
            println!("Logs are written to stdout via env_logger.");
        }
        crate::cli::commands::Cmd::Tui => {
            info!("Starting TUI...");
            let config = crate::config::RuntimeConfig::load();
            let mut app = tui::app::TuiApp::new(config)?;
            tokio::select! {
                res = app.run() => { res?; }
                _ = signal::ctrl_c() => {
                    info!("received ctrl-c, exiting tui");
                }
            }
        }
        crate::cli::commands::Cmd::Models { cmd } => {
            let cfg = crate::config::RuntimeConfig::load();
            let mgr = crate::models::ModelManager::new(Some(cfg.model_dir.clone()))?;
            match cmd {
                crate::cli::commands::ModelCmd::List => {
                    let ms = mgr.discover()?;
                    println!("Models:");
                    for m in ms { println!("- {} ({}, {} bytes)", m.name, m.format, m.size); }
                }
                crate::cli::commands::ModelCmd::Import { path } => {
                    let p = std::path::Path::new(&path);
                    let mi = mgr.import(p)?;
                    println!("Imported model: {} -> {}", mi.name, mi.path.display());
                }
                crate::cli::commands::ModelCmd::Remove { name } => {
                    mgr.remove(&name)?;
                    println!("Removed model {}", name);
                }
                crate::cli::commands::ModelCmd::Serve { action, model } => {
                    let mgr = std::sync::Arc::new(mgr);
                    let server = crate::models::ModelServer::new(mgr.clone(), cfg.model_server_addr);
                    if action == "start" {
                        server.start_local_server().await?;
                        if let Some(mn) = model {
                            // try to start a real Llama provider if binary available, else fallback to mock
                            let ms = mgr.discover()?;
                            if let Some(minfo) = ms.into_iter().find(|m| m.name == mn) {
                                let lp = crate::models::server::LlamaProvider::new(None, minfo.path.clone(), cfg.model_server_addr);
                                match lp.start().await {
                                    Ok(_) => {
                                        server.register_provider(&mn, std::sync::Arc::new(lp)).await?;
                                        println!("Model server started and llama provider registered for {}", mn);
                                    }
                                    Err(e) => {
                                        // fallback mock
                                        server.register_mock_for_model(&mn).await?;
                                        println!("Started server but llama provider failed; registered mock for {}: {}", mn, e);
                                    }
                                }
                            } else {
                                println!("model {} not found", mn);
                            }
                        } else {
                            println!("Model server started on {}", server.addr);
                        }
                    } else if action == "stop" {
                        println!("stop action not implemented in v0.1");
                    }
                }
                crate::cli::commands::ModelCmd::Install { tool } => {
                    if tool.as_deref() == Some("llama") || tool.is_none() {
                        // try to run helper script if present
                        let script = std::path::Path::new("./scripts/install_llama.sh");
                        if script.exists() {
                            println!("Running install script: {:?}", script);
                            match std::process::Command::new("/bin/sh").arg(script).status() {
                                Ok(s) => { println!("Installer exited: {}", s); }
                                Err(e) => { println!("Failed to run installer: {}", e); }
                            }
                        } else {
                            println!("No install script found at ./scripts/install_llama.sh. See docs/INSTALL.md for guidance.");
                        }
                    } else {
                        println!("Unknown tool: {:?}. Supported: llama", tool);
                    }
                }
            }
        }
        crate::cli::commands::Cmd::Exit => {
            println!("exiting");
        }
    }

    Ok(())
}
