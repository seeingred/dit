use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use dit_core::figma::{augment_node_path, resolve_command, setup_downloader, FigmaAuth};
use dit_core::git_ops;
use dit_core::repository::DitRepository;
use dit_core::types::DitPaths;

// ── CLI definition ───────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "dit", about = "Design version control", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new DIT repository
    Init,
    /// Show working tree status
    Status,
    /// Record a design snapshot
    Commit {
        /// Commit message
        #[arg(short, long)]
        m: String,
        /// Path to a local .fig file to commit
        #[arg(long)]
        fig: Option<PathBuf>,
        /// Commit on-disk snapshot without downloading from Figma
        #[arg(long)]
        local: bool,
    },
    /// Show commit history
    Log {
        /// Maximum number of commits to show
        #[arg(short, long, default_value = "20")]
        n: usize,
    },
    /// List or create branches
    Branch {
        /// Name of branch to create (omit to list)
        name: Option<String>,
    },
    /// Switch to a branch or commit
    Checkout {
        /// Branch name or commit hash
        #[arg(name = "ref")]
        reference: String,
    },
    /// Merge a branch into the current branch
    Merge {
        /// Branch to merge
        branch: String,
    },
    /// Restore a design file from a commit
    Restore {
        /// Commit hash to restore
        commit: String,
    },
    /// Push to remote repository
    Push {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,
    },
    /// Pull from remote repository
    Pull {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,
    },
    /// Set up the Playwright downloader (install dependencies)
    Setup,
    /// Compare two commits visually
    Diff {
        /// First commit hash
        commit1: String,
        /// Second commit hash
        commit2: String,
    },
}

// ── Entry point ──────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("{} {:#}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Status => cmd_status(),
        Commands::Commit { m, fig, local } => cmd_commit(&m, fig.as_deref(), local),
        Commands::Log { n } => cmd_log(n),
        Commands::Branch { name } => cmd_branch(name),
        Commands::Checkout { reference } => cmd_checkout(&reference),
        Commands::Merge { branch } => cmd_merge(&branch),
        Commands::Restore { commit } => cmd_restore(&commit),
        Commands::Push { remote } => cmd_push(&remote),
        Commands::Pull { remote } => cmd_pull(&remote),
        Commands::Setup => cmd_setup(),
        Commands::Diff { commit1, commit2 } => cmd_diff(&commit1, &commit2),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────

fn cwd() -> Result<PathBuf> {
    env::current_dir().context("failed to get current directory")
}

fn require_dit_repo() -> Result<PathBuf> {
    let path = cwd()?;
    if !git_ops::is_dit_repo(&path) {
        bail!(
            "Not a DIT repository. Run {} to initialize one.",
            style("dit init").cyan()
        );
    }
    Ok(path)
}

fn get_figma_auth() -> Result<FigmaAuth> {
    let _ = dotenvy::dotenv();

    // Try cookie first
    if let Ok(cookie) = env::var("FIGMA_AUTH_COOKIE") {
        return Ok(FigmaAuth::Cookie(cookie));
    }

    // Try email/password
    if let (Ok(email), Ok(password)) = (env::var("FIGMA_EMAIL"), env::var("FIGMA_PASSWORD")) {
        return Ok(FigmaAuth::EmailPassword { email, password });
    }

    bail!(
        "No Figma credentials found. Set {} or both {} and {} in a {} file.",
        style("FIGMA_AUTH_COOKIE").cyan(),
        style("FIGMA_EMAIL").cyan(),
        style("FIGMA_PASSWORD").cyan(),
        style(".env").cyan()
    )
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

fn read_config(repo_root: &Path) -> Result<dit_core::types::DitConfig> {
    let config_path = repo_root.join(DitPaths::CONFIG_FILE);
    let json = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read {}", config_path.display()))?;
    serde_json::from_str(&json).context("failed to parse config")
}

// ── Commands ─────────────────────────────────────────────────────────

fn cmd_init() -> Result<()> {
    let path = cwd()?;

    if git_ops::is_dit_repo(&path) {
        println!(
            "{} Already a DIT repository.",
            style("!").yellow().bold()
        );
        return Ok(());
    }

    // 1. Prompt for design platform.
    let platforms = &["Figma"];
    let selection = dialoguer::Select::new()
        .with_prompt("Select design platform")
        .items(platforms)
        .default(0)
        .interact()
        .context("selection cancelled")?;

    if selection != 0 {
        bail!("Only Figma is supported in this version.");
    }

    // 2. Ask for file key.
    let file_key: String = dialoguer::Input::new()
        .with_prompt("Enter Figma file key (from the URL)")
        .interact_text()
        .context("input cancelled")?;

    // 3. Ask for project name.
    let file_name: String = dialoguer::Input::new()
        .with_prompt("Enter project name")
        .interact_text()
        .context("input cancelled")?;

    // 4. Collect Figma auth for .fig downloads (Playwright-based).
    println!();
    println!(
        "  {} DIT uses Playwright to download .fig files from Figma.",
        style("→").cyan().bold()
    );
    println!("  Choose an authentication method:");
    let auth_methods = &["Browser cookie (FIGMA_AUTH_COOKIE)", "Email + password"];
    let auth_choice = dialoguer::Select::new()
        .with_prompt("Authentication method")
        .items(auth_methods)
        .default(0)
        .interact()
        .context("selection cancelled")?;

    let mut env_lines = Vec::new();
    if auth_choice == 0 {
        let cookie: String = dialoguer::Password::new()
            .with_prompt("Figma auth cookie value")
            .interact()
            .context("input cancelled")?;
        env_lines.push(format!("FIGMA_AUTH_COOKIE={cookie}"));
    } else {
        let email: String = dialoguer::Input::new()
            .with_prompt("Figma email")
            .interact_text()
            .context("input cancelled")?;
        let password: String = dialoguer::Password::new()
            .with_prompt("Figma password")
            .interact()
            .context("input cancelled")?;
        env_lines.push(format!("FIGMA_EMAIL={email}"));
        env_lines.push(format!("FIGMA_PASSWORD={password}"));
    }

    // 5. Initialize git repo + DIT structure.
    let sp = spinner("Initializing repository...");
    git_ops::init_repository(&path)?;

    // Write .dit/config.json.
    let config = dit_core::types::DitConfig {
        file_key: file_key.clone(),
        name: file_name.clone(),
        figma_token: None,
        schema_version: 1,
    };
    let config_json = serde_json::to_string_pretty(&config)?;
    std::fs::write(path.join(DitPaths::CONFIG_FILE), &config_json)?;

    // Write .env with credentials (git-ignored).
    let env_content = env_lines.join("\n") + "\n";
    std::fs::write(path.join(".env"), &env_content)?;

    // Add .env to .gitignore.
    let gitignore_path = path.join(".gitignore");
    let existing = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    if !existing.contains(".env") {
        std::fs::write(&gitignore_path, format!("{existing}.env\n"))?;
    }

    sp.finish_with_message("Repository initialized");

    // 6. Set up Playwright downloader (non-fatal).
    let sp = spinner("Setting up downloader...");
    match setup_downloader() {
        Ok(_) => sp.finish_with_message("Downloader ready"),
        Err(e) => {
            sp.finish_with_message("Downloader setup skipped");
            eprintln!("  {} Could not set up downloader: {e:#}", style("!").yellow().bold());
            eprintln!("  Run {} to retry.", style("dit setup").cyan());
        }
    }

    println!();
    println!(
        "  {} DIT repository initialized for {}",
        style("✓").green().bold(),
        style(&file_name).cyan()
    );
    println!(
        "  Run {} to take your first snapshot.",
        style("dit commit -m \"Initial snapshot\"").cyan()
    );

    Ok(())
}

fn cmd_setup() -> Result<()> {
    // 1. Set up downloader scripts + node_modules.
    let sp = spinner("Setting up downloader...");
    let downloader_dir = setup_downloader()
        .context("Failed to set up downloader")?;
    sp.finish_with_message("Downloader ready");

    // 2. Install Playwright's Chromium browser.
    let sp = spinner("Installing Playwright Chromium...");
    let npx = resolve_command("npx")
        .context("Failed to find npx")?;
    let mut npx_cmd = std::process::Command::new(&npx);
    npx_cmd
        .args(["playwright", "install", "chromium"])
        .current_dir(&downloader_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());
    augment_node_path(&npx, &mut npx_cmd);
    let status = npx_cmd
        .status()
        .context("Failed to run npx playwright install chromium")?;

    if !status.success() {
        sp.finish_with_message("Playwright install failed");
        bail!(
            "Playwright Chromium installation failed (exit code {}).\n  \
             Try running manually: cd {} && npx playwright install chromium",
            status.code().unwrap_or(-1),
            downloader_dir.display()
        );
    }
    sp.finish_with_message("Playwright Chromium installed");

    println!();
    println!(
        "  {} Downloader is ready at {}",
        style("✓").green().bold(),
        style(downloader_dir.display()).cyan()
    );

    Ok(())
}

fn cmd_status() -> Result<()> {
    let path = require_dit_repo()?;
    let status = git_ops::get_status(&path)?;

    println!("On branch {}", style(&status.branch).cyan().bold());

    if let Some(ref head) = status.head {
        println!("HEAD at {}", style(&head[..7.min(head.len())]).yellow());
    }

    if status.is_dirty {
        println!();
        println!("Changes:");
        for change in &status.changes {
            let (marker, color) = match change.change_type {
                dit_core::types::ChangeType::Added => ("+", console::Color::Green),
                dit_core::types::ChangeType::Modified => ("~", console::Color::Yellow),
                dit_core::types::ChangeType::Deleted => ("-", console::Color::Red),
            };
            println!(
                "  {} {}",
                style(marker).fg(color).bold(),
                style(&change.path).fg(color)
            );
        }
    } else {
        println!("Working tree clean");
    }

    Ok(())
}

fn cmd_commit(message: &str, fig_path: Option<&Path>, local: bool) -> Result<()> {
    let path = require_dit_repo()?;
    let repo = DitRepository::open(&path)?;
    let config = read_config(&path)?;

    if let Some(fig_path) = fig_path {
        // Commit from a local .fig file
        let sp = spinner("Converting .fig file...");
        let hash = repo.commit_from_local_fig(fig_path, &config.file_key, message)?;
        sp.finish_with_message(format!(
            "Committed {}",
            style(&hash[..7.min(hash.len())]).yellow()
        ));

        println!();
        println!(
            "  {} {} {}",
            style("✓").green().bold(),
            style(&hash[..7.min(hash.len())]).yellow(),
            message
        );
    } else if !local {
        // Download .fig from Figma and commit
        let auth = get_figma_auth()?;

        let sp = spinner("Downloading .fig from Figma...");
        let hash = repo.commit_from_fig(&config.file_key, &auth, message, Some(&|msg: &str| {
            sp.set_message(msg.to_string());
        }))?;
        sp.finish_with_message(format!(
            "Committed {}",
            style(&hash[..7.min(hash.len())]).yellow()
        ));

        println!();
        println!(
            "  {} {} {}",
            style("✓").green().bold(),
            style(&hash[..7.min(hash.len())]).yellow(),
            message
        );
    } else {
        // Local mode: commit the on-disk snapshot
        let project_file = path.join(DitPaths::PROJECT_FILE);
        if !project_file.exists() {
            bail!(
                "No snapshot on disk. Provide a .fig file with {} or download from Figma.",
                style("--fig <path>").cyan()
            );
        }
        println!(
            "  {} Using on-disk snapshot",
            style("→").cyan().bold()
        );

        // Git commit.
        let sp = spinner("Committing...");
        let hash = git_ops::commit_all(&path, message)?;
        sp.finish_with_message(format!(
            "Committed {}",
            style(&hash[..7.min(hash.len())]).yellow()
        ));

        println!();
        println!(
            "  {} {} {}",
            style("✓").green().bold(),
            style(&hash[..7.min(hash.len())]).yellow(),
            message
        );
    }

    Ok(())
}

fn cmd_log(max_count: usize) -> Result<()> {
    let path = require_dit_repo()?;
    let log = git_ops::get_log(&path, max_count)?;

    if log.is_empty() {
        println!("No commits yet.");
        return Ok(());
    }

    for entry in &log {
        let short_hash = &entry.hash[..7.min(entry.hash.len())];
        println!(
            "{} {} — {}",
            style(short_hash).yellow(),
            &entry.message,
            &entry.timestamp
        );
    }

    Ok(())
}

fn cmd_branch(name: Option<String>) -> Result<()> {
    let path = require_dit_repo()?;

    if let Some(name) = name {
        git_ops::create_branch(&path, &name)?;
        println!(
            "  {} Created branch {}",
            style("✓").green().bold(),
            style(&name).cyan()
        );
    } else {
        let branches = git_ops::list_branches(&path)?;
        for b in &branches {
            let marker = if b.is_current { "* " } else { "  " };
            let name_styled = if b.is_current {
                style(&b.name).green().bold().to_string()
            } else {
                b.name.clone()
            };
            let short_head = &b.head[..7.min(b.head.len())];
            println!("{}{} {}", marker, name_styled, style(short_head).yellow());
        }
    }

    Ok(())
}

fn cmd_checkout(reference: &str) -> Result<()> {
    let path = require_dit_repo()?;
    git_ops::checkout(&path, reference)?;
    println!(
        "  {} Switched to {}",
        style("✓").green().bold(),
        style(reference).cyan()
    );
    Ok(())
}

#[allow(unreachable_code, unused_variables)]
fn cmd_merge(branch: &str) -> Result<()> {
    println!(
        "  {} Merge is not implemented yet.",
        style("!").yellow().bold()
    );
    return Ok(());

    let path = require_dit_repo()?;
    let result = git_ops::merge(&path, branch)?;

    if result.success {
        if result.fast_forward {
            println!(
                "  {} Merged {} successfully (fast-forward)",
                style("✓").green().bold(),
                style(branch).cyan()
            );
            if let Some(ref hash) = result.commit_hash {
                let short = &hash[..7.min(hash.len())];
                println!("  Now at: {}", style(short).yellow());
            }
            // For fast-forward, the target commit's .fig already exists.
            if let Some(ref fig) = result.fig_snapshots.theirs {
                println!("  .fig snapshot: {}", style(fig).dim());
            }
        } else {
            println!(
                "  {} Merged {} successfully",
                style("✓").green().bold(),
                style(branch).cyan()
            );
            if let Some(ref hash) = result.commit_hash {
                let short = &hash[..7.min(hash.len())];
                println!("  Merge commit: {}", style(short).yellow());
            }
            // After a real merge, no .fig exists for the merge commit.
            print_fig_snapshot_guidance(&result, branch);
        }
    } else {
        println!(
            "  {} Merge conflicts in {} files:",
            style("✗").red().bold(),
            result.conflicts.len()
        );
        for p in &result.conflicts {
            println!("    {} {}", style("C").red(), p);
        }
        println!();
        print_fig_snapshot_guidance(&result, branch);
        println!(
            "  To resolve:",
        );
        println!("    1. Fix the JSON conflicts in the files above");
        println!("    2. Open a .fig file as your starting point in Figma");
        println!(
            "    3. Run {} to finish",
            style("dit commit -m \"Resolve merge\"").cyan()
        );
    }

    Ok(())
}

/// Print available .fig snapshots and guidance after a merge.
fn print_fig_snapshot_guidance(result: &git_ops::MergeResult, their_branch: &str) {
    let snaps = &result.fig_snapshots;
    let has_any = snaps.ours.is_some() || snaps.theirs.is_some();
    if !has_any {
        return;
    }
    println!();
    println!("  Available .fig snapshots:");
    if let Some(ref fig) = snaps.ours {
        let label = snaps
            .ours_commit
            .as_deref()
            .map(|h| &h[..7.min(h.len())])
            .unwrap_or("current");
        println!(
            "    Current branch ({}): {}",
            style(label).yellow(),
            style(fig).dim()
        );
    }
    if let Some(ref fig) = snaps.theirs {
        let label = snaps
            .theirs_commit
            .as_deref()
            .map(|h| &h[..7.min(h.len())])
            .unwrap_or(their_branch);
        println!(
            "    {} ({}): {}",
            style(their_branch).cyan(),
            style(label).yellow(),
            style(fig).dim()
        );
    }
    if result.success {
        println!();
        println!("  No .fig file exists for the merged state.");
        println!("  To complete the merge visually:");
        println!("    1. Open a .fig file above in Figma as your starting point");
        println!("    2. Adjust the design to match the merged state");
        println!(
            "    3. Run {} to capture the merged .fig",
            style("dit commit").cyan()
        );
    }
}

fn cmd_restore(commit: &str) -> Result<()> {
    let path = require_dit_repo()?;
    let repo = DitRepository::open(&path)?;

    let sp = spinner("Restoring snapshot...");
    let result = repo.restore(commit)?;
    sp.finish_with_message(format!(
        "Restored to {}",
        style(&commit[..7.min(commit.len())]).yellow()
    ));

    println!();
    println!(
        "  {} Design restored! {} ({} pages)",
        style("✓").green().bold(),
        style(&result.snapshot.project.name).cyan(),
        result.snapshot.pages.len()
    );

    if let Some(ref fig_path) = result.fig_file_path {
        println!();
        println!(
            "  Open this file in Figma: {}",
            style(fig_path.display()).cyan().bold()
        );

        // On macOS, offer to open the file directly.
        if cfg!(target_os = "macos") {
            let open_it = dialoguer::Confirm::new()
                .with_prompt("Open .fig file now?")
                .default(true)
                .interact()
                .unwrap_or(false);

            if open_it {
                std::process::Command::new("open")
                    .arg(fig_path)
                    .spawn()
                    .context("failed to open .fig file")?;
            }
        }
    } else {
        println!("  No .fig file available for this commit.");
        println!("  Design JSON files are in the working directory.");
    }

    Ok(())
}

fn cmd_push(remote: &str) -> Result<()> {
    let path = require_dit_repo()?;
    let status = git_ops::get_status(&path)?;

    let sp = spinner(&format!("Pushing {} to {}...", &status.branch, remote));
    git_ops::push(&path, remote, &status.branch)?;
    sp.finish_with_message(format!(
        "Pushed {} to {}",
        style(&status.branch).cyan(),
        style(remote).cyan()
    ));

    Ok(())
}

fn cmd_pull(remote: &str) -> Result<()> {
    let path = require_dit_repo()?;
    let status = git_ops::get_status(&path)?;

    let sp = spinner(&format!("Pulling {} from {}...", &status.branch, remote));
    git_ops::pull(&path, remote, &status.branch)?;
    sp.finish_with_message(format!(
        "Pulled {} from {}",
        style(&status.branch).cyan(),
        style(remote).cyan()
    ));

    Ok(())
}

fn cmd_diff(commit1: &str, commit2: &str) -> Result<()> {
    let _path = require_dit_repo()?;
    let short1 = &commit1[..7.min(commit1.len())];
    let short2 = &commit2[..7.min(commit2.len())];

    println!(
        "Comparing {} vs {}",
        style(short1).yellow(),
        style(short2).yellow()
    );
    println!();
    println!(
        "  Visual diff is available in the DIT GUI ({}).",
        style("dit-gui").cyan()
    );
    println!(
        "  JSON diff: run {} {} {}",
        style("git diff").cyan(),
        commit1,
        commit2
    );

    Ok(())
}
