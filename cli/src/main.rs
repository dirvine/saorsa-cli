mod config;
mod downloader;
mod menu;
mod platform;
mod runner;

use anyhow::{Context, Result};
use clap::Parser;
use config::Config;
use downloader::Downloader;
use menu::{Menu, MenuChoice};
use platform::Platform;
use runner::BinaryRunner;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "saorsa",
    about = "Interactive CLI menu for Saorsa tools",
    version,
    author
)]
struct Args {
    /// Disable automatic update checks
    #[arg(long)]
    no_update_check: bool,

    /// Use system-installed binaries instead of downloading
    #[arg(long)]
    use_system: bool,

    /// Force re-download of binaries
    #[arg(long)]
    force_download: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Run a specific tool directly (sb or sdisk)
    #[arg(short, long)]
    run: Option<String>,

    /// Arguments to pass to the tool (when using --run)
    #[arg(trailing_var_arg = true)]
    tool_args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let filter = if args.verbose {
        EnvFilter::from_default_env()
            .add_directive("cli=debug".parse()?)
            .add_directive("saorsa=debug".parse()?)
    } else {
        EnvFilter::from_default_env()
            .add_directive("cli=info".parse()?)
            .add_directive("saorsa=info".parse()?)
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Load configuration
    let mut config = Config::load().unwrap_or_default();
    config.update_from_cli(args.no_update_check, args.use_system);
    config.ensure_directories()?;

    // Detect platform
    let platform = Platform::detect()
        .context("Failed to detect platform")?;
    
    tracing::debug!("Detected platform: {:?}", platform);

    // Initialize components
    let downloader = Downloader::new(
        config.github.owner.clone(),
        config.github.repo.clone(),
    )?;
    
    let runner = BinaryRunner::new();

    // Handle direct run mode
    if let Some(tool) = args.run.as_ref() {
        return run_tool_directly(
            &tool,
            args.tool_args,
            &config,
            &platform,
            &downloader,
            &runner,
            args.force_download,
        )
        .await;
    }

    // Main menu loop
    let mut menu = Menu::new();
    
    loop {
        // Check for binaries and update menu
        let (sb_path, sdisk_path) = check_binaries(&config, &platform, &downloader, &runner).await?;
        menu.set_binary_paths(sb_path.clone(), sdisk_path.clone());

        // Show menu and get choice
        let choice = menu.run().await?;

        match choice {
            MenuChoice::RunSB => {
                if let Some(path) = sb_path {
                    println!("Starting Saorsa Browser...");
                    runner.run_interactive(&path, vec![])?;
                } else {
                    println!("Saorsa Browser not installed. Downloading...");
                    let path = downloader
                        .download_binary("sb", &platform, false)
                        .await?;
                    runner.run_interactive(&path, vec![])?;
                }
            }
            MenuChoice::RunSDisk => {
                if let Some(path) = sdisk_path {
                    println!("Starting Saorsa Disk...");
                    runner.run_interactive(&path, vec![])?;
                } else {
                    println!("Saorsa Disk not installed. Downloading...");
                    let path = downloader
                        .download_binary("sdisk", &platform, false)
                        .await?;
                    runner.run_interactive(&path, vec![])?;
                }
            }
            MenuChoice::UpdateBinaries => {
                println!("Updating binaries...");
                update_binaries(&platform, &downloader).await?;
                println!("Update complete! Press Enter to continue...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
            }
            MenuChoice::Settings => {
                show_settings(&config)?;
            }
            MenuChoice::Exit => {
                println!("Goodbye!");
                break;
            }
        }
    }

    Ok(())
}

async fn check_binaries(
    config: &Config,
    platform: &Platform,
    downloader: &Downloader,
    runner: &BinaryRunner,
) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    let mut sb_path = None;
    let mut sdisk_path = None;

    // Check for sb binary
    if config.behavior.use_system_binaries {
        sb_path = runner.which("sb");
    }
    if sb_path.is_none() {
        let cache_path = downloader.binary_path("sb", platform);
        if runner.check_binary_exists(&cache_path) {
            sb_path = Some(cache_path);
        }
    }

    // Check for sdisk binary
    if config.behavior.use_system_binaries {
        sdisk_path = runner.which("sdisk");
    }
    if sdisk_path.is_none() {
        let cache_path = downloader.binary_path("sdisk", platform);
        if runner.check_binary_exists(&cache_path) {
            sdisk_path = Some(cache_path);
        }
    }

    Ok((sb_path, sdisk_path))
}

async fn update_binaries(platform: &Platform, downloader: &Downloader) -> Result<()> {
    println!("Downloading latest sb binary...");
    downloader.download_binary("sb", platform, true).await?;
    
    println!("Downloading latest sdisk binary...");
    downloader.download_binary("sdisk", platform, true).await?;
    
    Ok(())
}

async fn run_tool_directly(
    tool: &str,
    args: Vec<String>,
    config: &Config,
    platform: &Platform,
    downloader: &Downloader,
    runner: &BinaryRunner,
    force_download: bool,
) -> Result<()> {
    let binary_name = match tool {
        "sb" | "saorsa-browser" => "sb",
        "sdisk" | "saorsa-disk" => "sdisk",
        _ => {
            anyhow::bail!("Unknown tool: {}. Available tools: sb, sdisk", tool);
        }
    };

    // Try to find the binary
    let mut binary_path = None;
    
    if config.behavior.use_system_binaries && !force_download {
        binary_path = runner.which(binary_name);
    }
    
    if binary_path.is_none() {
        let cache_path = downloader.binary_path(binary_name, platform);
        if runner.check_binary_exists(&cache_path) && !force_download {
            binary_path = Some(cache_path);
        } else {
            println!("Downloading {} binary...", binary_name);
            binary_path = Some(
                downloader
                    .download_binary(binary_name, platform, force_download)
                    .await?,
            );
        }
    }

    if let Some(path) = binary_path {
        runner.run_interactive(&path, args)?;
    } else {
        anyhow::bail!("Failed to find or download {} binary", binary_name);
    }

    Ok(())
}

fn show_settings(config: &Config) -> Result<()> {
    println!("\n=== Current Settings ===\n");
    println!("GitHub Repository: {}/{}", config.github.owner, config.github.repo);
    println!("Check Prereleases: {}", config.github.check_prerelease);
    println!("Cache Directory: {:?}", config.cache_dir()?);
    println!("Auto Update Check: {}", config.behavior.auto_update_check);
    println!("Use System Binaries: {}", config.behavior.use_system_binaries);
    println!("Prefer Local Build: {}", config.behavior.prefer_local_build);
    println!("\nConfig file: {:?}", Config::config_path()?);
    println!("\nPress Enter to continue...");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    Ok(())
}