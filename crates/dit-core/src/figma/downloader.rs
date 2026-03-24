//! .fig file downloader using Playwright browser automation.
//!
//! Embeds `scripts/download-fig.mjs` and `scripts/package.json` at compile
//! time, writes them to `~/.dit/downloader/`, installs npm dependencies
//! automatically, and shells out to Node.js to perform the download.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

const DOWNLOAD_SCRIPT: &str = include_str!("../../../../scripts/download-fig.mjs");
const PACKAGE_JSON: &str = include_str!("../../../../scripts/package.json");

/// Resolve the full path to a command (`node`, `npm`, `npx`).
///
/// On macOS, GUI apps (`.app` bundles) don't inherit the user's shell PATH,
/// so tools installed via Homebrew, nvm, Volta, etc. aren't found. We try
/// multiple strategies: direct lookup, the user's login shell, and common
/// installation paths.
pub fn resolve_command(name: &str) -> Result<PathBuf> {
    // 1. Try directly (works from terminal where PATH is correct).
    if let Ok(status) = std::process::Command::new(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        if status.success() {
            return Ok(PathBuf::from(name));
        }
    }

    // 2. Try the user's actual shell (zsh/bash) with login+interactive flags.
    //    - Login (-l) sources .zprofile/.bash_profile (Homebrew, Volta)
    //    - Interactive (-i) sources .zshrc/.bashrc (nvm)
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
    let cmd_str = format!("command -v {name}");
    for flags in &[vec!["-lic", &cmd_str], vec!["-lc", &cmd_str]] {
        if let Ok(output) = std::process::Command::new(&shell)
            .args(flags)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
        {
            if output.status.success() {
                let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !resolved.is_empty() && Path::new(&resolved).exists() {
                    return Ok(PathBuf::from(resolved));
                }
            }
        }
    }

    // 3. Check common installation directories.
    for dir in &["/opt/homebrew/bin", "/usr/local/bin"] {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 4. Check nvm versions (~/.nvm/versions/node/*/bin/).
    if let Some(home) = dirs::home_dir() {
        let nvm_dir = home.join(".nvm/versions/node");
        if nvm_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                let mut versions: Vec<_> = entries
                    .flatten()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .collect();
                // Sort descending to prefer the latest version.
                versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
                for v in versions {
                    let candidate = v.path().join("bin").join(name);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }

        // 5. Check Volta (~/.volta/bin/).
        let volta = home.join(".volta/bin").join(name);
        if volta.exists() {
            return Ok(volta);
        }
    }

    bail!(
        "`{name}` not found. Install Node.js from https://nodejs.org/"
    )
}

/// Add the resolved binary's directory to a child process's PATH.
///
/// When we resolve e.g. `/Users/x/.nvm/versions/node/v20/bin/npm`, the npm
/// script itself needs `node` on PATH. This ensures sibling binaries are
/// discoverable by the child process.
pub fn augment_node_path(bin_path: &Path, cmd: &mut std::process::Command) {
    if let Some(bin_dir) = bin_path.parent() {
        if bin_dir.as_os_str().is_empty() {
            return;
        }
        let current = std::env::var("PATH").unwrap_or_default();
        cmd.env("PATH", format!("{}:{current}", bin_dir.display()));
    }
}

/// Authentication method for Figma.
#[derive(Debug, Clone)]
pub enum FigmaAuth {
    /// Authenticate using the `__Host-figma.authn` cookie value.
    Cookie(String),
    /// Authenticate using email and password.
    EmailPassword { email: String, password: String },
}

/// Ensure the downloader directory at `~/.dit/downloader/` is ready.
///
/// - Creates the directory if it doesn't exist
/// - Writes (or overwrites) `download-fig.mjs` and `package.json`
/// - Runs `npm install` if `node_modules/` is missing
///
/// Returns the path to `download-fig.mjs`.
fn ensure_downloader_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let downloader_dir = home.join(".dit").join("downloader");

    std::fs::create_dir_all(&downloader_dir)
        .context("failed to create ~/.dit/downloader/")?;

    let script_path = downloader_dir.join("download-fig.mjs");
    let package_path = downloader_dir.join("package.json");

    std::fs::write(&script_path, DOWNLOAD_SCRIPT)
        .context("failed to write download-fig.mjs")?;
    std::fs::write(&package_path, PACKAGE_JSON)
        .context("failed to write package.json")?;

    let node_modules = downloader_dir.join("node_modules");
    if !node_modules.exists() {
        let npm = resolve_command("npm")?;
        let mut npm_cmd = std::process::Command::new(&npm);
        npm_cmd.arg("install").current_dir(&downloader_dir);
        augment_node_path(&npm, &mut npm_cmd);
        let output = npm_cmd
            .output()
            .with_context(|| format!("failed to run `{} install`", npm.display()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "`npm install` failed in ~/.dit/downloader/ (exit {}):\n{}",
                output.status.code().unwrap_or(-1),
                stderr,
            );
        }
    }

    Ok(script_path)
}

/// Set up the downloader directory and return its path.
///
/// This creates `~/.dit/downloader/`, writes the embedded scripts,
/// and installs npm dependencies if needed.
pub fn setup_downloader() -> Result<PathBuf> {
    let script_path = ensure_downloader_dir()?;
    let downloader_dir = script_path
        .parent()
        .expect("script_path must have a parent")
        .to_path_buf();
    Ok(downloader_dir)
}

/// Download a .fig file from Figma.
///
/// Uses Playwright (via Node.js) to navigate to the Figma editor,
/// trigger "Save as .fig", and save the result to `output_path`.
/// Optionally captures a preview screenshot of the editor canvas.
///
/// If `on_progress` is provided, it will be called with each progress
/// message from the download script (e.g. "Launching browser...").
/// Otherwise, progress is logged via `tracing::info!`.
///
/// If `on_2fa` is provided and the Figma login requires two-factor
/// authentication, it will be called to obtain the 2FA code from the user.
/// If `on_2fa` is None and 2FA is required, the download will fail.
pub fn download_fig_file(
    file_key: &str,
    output_path: &Path,
    auth: &FigmaAuth,
    preview_output_path: Option<&Path>,
    on_progress: Option<&dyn Fn(&str)>,
    on_2fa: Option<&dyn Fn() -> Option<String>>,
) -> Result<()> {
    let script_path = ensure_downloader_dir()
        .context("failed to set up downloader scripts")?;

    let node = resolve_command("node")?;
    let mut cmd = std::process::Command::new(&node);
    augment_node_path(&node, &mut cmd);
    cmd.arg(&script_path)
        .arg("--file-key")
        .arg(file_key)
        .arg("--output")
        .arg(output_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped());

    if let Some(preview_path) = preview_output_path {
        cmd.arg("--preview-output").arg(preview_path);
    }

    match auth {
        FigmaAuth::Cookie(cookie) => {
            cmd.arg("--cookie").arg(cookie);
        }
        FigmaAuth::EmailPassword { email, password } => {
            cmd.arg("--email").arg(email).arg("--password").arg(password);
        }
    }

    let mut child = cmd
        .spawn()
        .context("failed to execute download-fig.mjs — is Node.js installed?")?;

    // Take ownership of stdin so we can write to it if 2FA is needed.
    let mut child_stdin = child.stdin.take();

    // Read stderr in a background thread to prevent pipe buffer deadlocks.
    let stderr_handle = child.stderr.take().map(|stderr| {
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = String::new();
            let mut stderr = stderr;
            stderr.read_to_string(&mut buf).ok();
            buf
        })
    });

    // Read stdout lines in real-time, forwarding [DIT] progress messages
    // and handling 2FA requests.
    if let Some(stdout) = child.stdout.take() {
        use std::io::{BufRead, Write};
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                if line == "[DIT:2FA_REQUIRED]" {
                    // 2FA code requested by the script
                    if let Some(ref cb) = on_2fa {
                        if let Some(code) = cb() {
                            if let Some(ref mut stdin) = child_stdin {
                                writeln!(stdin, "{}", code).ok();
                                stdin.flush().ok();
                            }
                        } else {
                            // User cancelled 2FA
                            if let Some(ref mut stdin) = child_stdin {
                                writeln!(stdin, "").ok();
                                stdin.flush().ok();
                            }
                        }
                    } else {
                        bail!("Figma requires two-factor authentication but no 2FA handler is available");
                    }
                    // Drop stdin so the child process can exit cleanly
                    child_stdin = None;
                } else if let Some(msg) = line.strip_prefix("[DIT] ") {
                    if let Some(cb) = on_progress {
                        cb(msg);
                    } else {
                        tracing::info!("{}", msg);
                    }
                }
            }
        }
    }

    let status = child
        .wait()
        .context("failed to wait for download-fig.mjs")?;

    let stderr_output = stderr_handle
        .map(|h| h.join().unwrap_or_default())
        .unwrap_or_default();

    if !status.success() {
        bail!(
            "download-fig.mjs failed (exit {}):\nstderr: {}",
            status.code().unwrap_or(-1),
            stderr_output,
        );
    }

    if !output_path.exists() {
        bail!(
            ".fig file was not created at {}",
            output_path.display()
        );
    }

    Ok(())
}
