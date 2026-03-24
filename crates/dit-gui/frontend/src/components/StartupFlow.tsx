import { useState, useEffect } from "react";
import {
  initRepo,
  openRepo,
  checkDirectory,
  cloneRepo,
  listSshKeys,
  saveCredentials,
} from "../commands";
import type { SshKeyInfo } from "../types";

interface StartupFlowProps {
  onRepoOpened: (path: string) => void;
}

type Step = "pick-folder" | "clone" | "cloning" | "figma-auth" | "figma-auth-only" | "figma-file" | "creating";
type AuthMethod = "cookie" | "email";

/** Extract a Figma file key from any Figma URL or raw key. */
function extractFileKey(input: string): string | null {
  const trimmed = input.trim();
  // https://www.figma.com/design/ABC123/... or /file/ABC123/...
  const urlMatch = trimmed.match(/figma\.com\/(?:design|file|proto|board)\/([a-zA-Z0-9]+)/);
  if (urlMatch) return urlMatch[1];
  // Raw alphanumeric key (at least 10 chars, no spaces)
  if (/^[a-zA-Z0-9]{10,}$/.test(trimmed)) return trimmed;
  return null;
}

export function StartupFlow({ onRepoOpened }: StartupFlowProps) {
  const [step, setStep] = useState<Step>("pick-folder");
  const [folderPath, setFolderPath] = useState("");

  // Auth state
  const [authMethod, setAuthMethod] = useState<AuthMethod>("cookie");
  const [authCookie, setAuthCookie] = useState("");
  const [authEmail, setAuthEmail] = useState("");
  const [authPassword, setAuthPassword] = useState("");

  // File state
  const [figmaLink, setFigmaLink] = useState("");
  const [fileKey, setFileKey] = useState("");
  const [fileName, setFileName] = useState("");

  // Clone state
  const [cloneUrl, setCloneUrl] = useState("");
  const [clonePath, setClonePath] = useState("");
  const [sshKeys, setSshKeys] = useState<SshKeyInfo[]>([]);
  const [selectedSshKey, setSelectedSshKey] = useState<string | null>(null);
  const isCloneSsh = cloneUrl.startsWith("git@") || cloneUrl.startsWith("ssh://");

  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [confirmOverwrite, setConfirmOverwrite] = useState(false);

  // Load SSH keys when entering clone step
  useEffect(() => {
    if (step === "clone") {
      listSshKeys().then((keys) => {
        setSshKeys(keys);
        if (keys.length > 0 && !selectedSshKey) {
          setSelectedSshKey(keys[0].path);
        }
      }).catch(() => setSshKeys([]));
    }
  }, [step]);

  const handlePickFolder = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        setFolderPath(selected as string);
        setError(null);
      }
    } catch {
      // Running outside Tauri (dev in browser)
    }
  };

  const handleOpen = async () => {
    if (!folderPath.trim()) return;
    setLoading(true);
    setError(null);
    try {
      await openRepo(folderPath);
      onRepoOpened(folderPath);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleStartInit = () => {
    if (!folderPath.trim()) return;
    setStep("figma-auth");
    setError(null);
  };

  const handlePickCloneFolder = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        setClonePath(selected as string);
        setError(null);
      }
    } catch {
      // Running outside Tauri (dev in browser)
    }
  };

  const handleClone = async () => {
    if (!cloneUrl.trim() || !clonePath.trim()) return;
    setLoading(true);
    setError(null);
    setStep("cloning");
    try {
      const sshKey = isCloneSsh ? selectedSshKey : null;
      const result = await cloneRepo(cloneUrl, clonePath, sshKey);
      if (result.is_dit_repo && !result.needs_auth) {
        // DIT repo with credentials — open directly
        onRepoOpened(result.path);
      } else if (result.is_dit_repo && result.needs_auth) {
        // DIT repo but missing credentials — ask for auth only
        setFolderPath(result.path);
        setStep("figma-auth-only");
      } else {
        // Plain git repo — need full DIT initialization
        setFolderPath(result.path);
        setStep("figma-auth");
      }
    } catch (e) {
      setError(String(e));
      setStep("clone");
    } finally {
      setLoading(false);
    }
  };

  const hasValidAuth = authMethod === "cookie"
    ? authCookie.trim().length > 0
    : authEmail.trim().length > 0 && authPassword.trim().length > 0;

  const handleAuthNext = () => {
    if (!hasValidAuth) return;
    setStep("figma-file");
    setError(null);
  };

  const handleFileKeyExtract = () => {
    const key = extractFileKey(figmaLink);
    if (!key) {
      setError("Paste a Figma file link, e.g. figma.com/design/ABC123/...");
      return;
    }
    setFileKey(key);
    setError(null);
  };

  const handleCreateRepo = async () => {
    if (!fileKey || !fileName.trim()) return;

    // Only warn if there's an existing DIT repo (not just git — git is fine).
    if (!confirmOverwrite) {
      try {
        const check = await checkDirectory(folderPath);
        if (check.has_dit) {
          setConfirmOverwrite(true);
          return;
        }
      } catch {
        // If check fails, proceed anyway.
      }
    }

    doInit(confirmOverwrite);
  };

  const doInit = async (force: boolean) => {
    if (!fileKey || !fileName.trim()) return;
    setLoading(true);
    setError(null);
    setConfirmOverwrite(false);
    setStep("creating");
    try {
      await initRepo(
        folderPath,
        authMethod === "cookie" ? authCookie : null,
        authMethod === "email" ? authEmail : null,
        authMethod === "email" ? authPassword : null,
        fileKey,
        fileName,
        force,
        selectedSshKey,
      );
      onRepoOpened(folderPath);
    } catch (e) {
      setError(String(e));
      setStep("figma-file");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center justify-center h-screen bg-dit-bg">
      <div className="w-full max-w-lg p-8">
        {/* Logo */}
        <div className="text-center mb-10">
          <h1 className="text-4xl font-bold text-dit-text tracking-tight mb-2">DIT</h1>
          <p className="text-dit-text-muted text-sm">Version control for design files</p>
        </div>

        {/* Step indicator */}
        {step !== "pick-folder" && (
          <div className="flex items-center justify-center gap-2 mb-8">
            {["Folder", "Auth", "File"].map((label, i) => {
              const stepIndex =
                step === "figma-auth" ? 1 : step === "figma-file" || step === "creating" ? 2 : 0;
              const isActive = i <= stepIndex;
              return (
                <div key={label} className="flex items-center gap-2">
                  <div
                    className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold ${
                      isActive
                        ? "bg-dit-accent text-white"
                        : "bg-dit-surface text-dit-text-muted border border-dit-border"
                    }`}
                  >
                    {i + 1}
                  </div>
                  <span className={`text-xs ${isActive ? "text-dit-text" : "text-dit-text-muted"}`}>
                    {label}
                  </span>
                  {i < 2 && (
                    <div className={`w-8 h-px ${isActive ? "bg-dit-accent" : "bg-dit-border"}`} />
                  )}
                </div>
              );
            })}
          </div>
        )}

        {error && (
          <div className="mb-4 px-4 py-3 bg-red-500/10 border border-red-500/30 rounded-lg">
            <p className="text-dit-danger text-sm">{error}</p>
          </div>
        )}

        {/* Step: Pick folder */}
        {step === "pick-folder" && (
          <div className="space-y-4">
            <div className="flex gap-2">
              <input
                type="text"
                value={folderPath}
                onChange={(e) => setFolderPath(e.target.value)}
                placeholder="Select a folder..."
                className="flex-1 bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                           text-dit-text placeholder:text-dit-text-muted text-sm
                           focus:outline-none focus:border-dit-accent transition-colors"
              />
              <button
                onClick={handlePickFolder}
                className="px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                           text-dit-text-muted hover:text-dit-text hover:border-dit-accent
                           transition-colors text-sm"
              >
                Browse
              </button>
            </div>
            <div className="flex gap-3">
              <button
                onClick={handleOpen}
                disabled={!folderPath.trim() || loading}
                className="flex-1 px-4 py-3 bg-dit-accent text-white rounded-lg font-medium
                           hover:bg-dit-accent-hover transition-colors text-sm
                           disabled:opacity-40 disabled:cursor-not-allowed"
              >
                Open Repository
              </button>
              <button
                onClick={handleStartInit}
                disabled={!folderPath.trim() || loading}
                className="flex-1 px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                           text-dit-text hover:border-dit-accent transition-colors text-sm
                           disabled:opacity-40 disabled:cursor-not-allowed"
              >
                Initialize New
              </button>
            </div>
            <div className="relative flex items-center my-4">
              <div className="flex-1 border-t border-dit-border" />
              <span className="px-3 text-dit-text-muted text-xs">or</span>
              <div className="flex-1 border-t border-dit-border" />
            </div>
            <button
              onClick={() => { setStep("clone"); setError(null); }}
              className="w-full px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                         text-dit-text hover:border-dit-accent transition-colors text-sm"
            >
              Clone Repository
            </button>
            <p className="text-dit-text-muted text-xs text-center mt-4">
              Open an existing DIT repo, initialize a new one, or clone from a URL.
            </p>
          </div>
        )}

        {/* Step: Clone */}
        {step === "clone" && (
          <div className="space-y-4">
            <div>
              <label className="block text-dit-text text-sm font-medium mb-2">
                Repository URL
              </label>
              <input
                type="text"
                value={cloneUrl}
                onChange={(e) => setCloneUrl(e.target.value)}
                placeholder="https://github.com/user/repo.git"
                className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                           text-dit-text placeholder:text-dit-text-muted text-sm
                           focus:outline-none focus:border-dit-accent transition-colors"
              />
            </div>
            <div>
              <label className="block text-dit-text text-sm font-medium mb-2">
                Clone into
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={clonePath}
                  onChange={(e) => setClonePath(e.target.value)}
                  placeholder="Select a folder..."
                  className="flex-1 bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                             text-dit-text placeholder:text-dit-text-muted text-sm
                             focus:outline-none focus:border-dit-accent transition-colors"
                />
                <button
                  onClick={handlePickCloneFolder}
                  className="px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                             text-dit-text-muted hover:text-dit-text hover:border-dit-accent
                             transition-colors text-sm"
                >
                  Browse
                </button>
              </div>
            </div>
            {/* SSH key picker — shown when URL is SSH */}
            {isCloneSsh && (
              <div>
                <label className="block text-dit-text text-sm font-medium mb-2">
                  SSH Key
                </label>
                {sshKeys.length > 0 ? (
                  <select
                    value={selectedSshKey ?? ""}
                    onChange={(e) => setSelectedSshKey(e.target.value || null)}
                    className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                               text-dit-text text-sm
                               focus:outline-none focus:border-dit-accent transition-colors"
                  >
                    {sshKeys.map((key) => (
                      <option key={key.path} value={key.path}>
                        {key.name}
                      </option>
                    ))}
                  </select>
                ) : (
                  <p className="text-dit-text-muted text-xs px-1">
                    No SSH keys found in ~/.ssh/
                  </p>
                )}
              </div>
            )}
            <div className="flex gap-3">
              <button
                onClick={() => { setStep("pick-folder"); setError(null); }}
                className="px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                           text-dit-text-muted hover:text-dit-text transition-colors text-sm"
              >
                Back
              </button>
              <button
                onClick={handleClone}
                disabled={!cloneUrl.trim() || !clonePath.trim() || loading || (isCloneSsh && sshKeys.length === 0)}
                className="flex-1 px-4 py-3 bg-dit-accent text-white rounded-lg font-medium
                           hover:bg-dit-accent-hover transition-colors text-sm
                           disabled:opacity-40 disabled:cursor-not-allowed"
              >
                Clone
              </button>
            </div>
          </div>
        )}

        {/* Step: Cloning */}
        {step === "cloning" && (
          <div className="text-center py-8">
            <div className="animate-spin w-8 h-8 border-2 border-dit-accent border-t-transparent rounded-full mx-auto mb-4" />
            <p className="text-dit-text text-sm">Cloning repository...</p>
          </div>
        )}

        {/* Step: Figma Auth */}
        {step === "figma-auth" && (
          <div className="space-y-4">
            <div>
              <label className="block text-dit-text text-sm font-medium mb-3">
                Figma Authentication
              </label>
              <p className="text-dit-text-muted text-xs mb-4">
                DIT downloads .fig files from Figma. Choose how to authenticate:
              </p>

              {/* Auth method toggle */}
              <div className="flex gap-2 mb-4">
                <button
                  onClick={() => setAuthMethod("cookie")}
                  className={`flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-colors border ${
                    authMethod === "cookie"
                      ? "bg-dit-accent/10 border-dit-accent text-dit-accent"
                      : "bg-dit-surface border-dit-border text-dit-text-muted hover:text-dit-text"
                  }`}
                >
                  Browser Cookie
                </button>
                <button
                  onClick={() => setAuthMethod("email")}
                  className={`flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-colors border ${
                    authMethod === "email"
                      ? "bg-dit-accent/10 border-dit-accent text-dit-accent"
                      : "bg-dit-surface border-dit-border text-dit-text-muted hover:text-dit-text"
                  }`}
                >
                  Email + Password
                </button>
              </div>

              {authMethod === "cookie" ? (
                <div>
                  <input
                    type="password"
                    value={authCookie}
                    onChange={(e) => setAuthCookie(e.target.value)}
                    placeholder="Paste your Figma auth cookie..."
                    className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                               text-dit-text placeholder:text-dit-text-muted text-sm
                               focus:outline-none focus:border-dit-accent transition-colors"
                  />
                  <p className="text-dit-text-muted text-xs mt-2">
                    Copy from your browser's cookies for figma.com (cookie name: __Host-figma.authn)
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  <input
                    type="email"
                    value={authEmail}
                    onChange={(e) => setAuthEmail(e.target.value)}
                    placeholder="Figma email address"
                    className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                               text-dit-text placeholder:text-dit-text-muted text-sm
                               focus:outline-none focus:border-dit-accent transition-colors"
                  />
                  <input
                    type="password"
                    value={authPassword}
                    onChange={(e) => setAuthPassword(e.target.value)}
                    placeholder="Figma password"
                    className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                               text-dit-text placeholder:text-dit-text-muted text-sm
                               focus:outline-none focus:border-dit-accent transition-colors"
                  />
                </div>
              )}
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => { setStep("pick-folder"); setError(null); }}
                className="px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                           text-dit-text-muted hover:text-dit-text transition-colors text-sm"
              >
                Back
              </button>
              <button
                onClick={handleAuthNext}
                disabled={!hasValidAuth}
                className="flex-1 px-4 py-3 bg-dit-accent text-white rounded-lg font-medium
                           hover:bg-dit-accent-hover transition-colors text-sm
                           disabled:opacity-40 disabled:cursor-not-allowed"
              >
                Next
              </button>
            </div>
          </div>
        )}

        {/* Step: Figma Auth Only (cloned DIT repo, just needs credentials) */}
        {step === "figma-auth-only" && (
          <div className="space-y-4">
            <div className="px-4 py-3 bg-green-500/10 border border-green-500/30 rounded-lg">
              <p className="text-green-400 text-sm font-medium">Repository cloned successfully</p>
              <p className="text-dit-text-muted text-xs mt-1">Set up Figma credentials to enable commits.</p>
            </div>
            <div>
              <label className="block text-dit-text text-sm font-medium mb-3">
                Figma Authentication
              </label>
              <div className="flex gap-2 mb-4">
                <button
                  onClick={() => setAuthMethod("cookie")}
                  className={`flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-colors border ${
                    authMethod === "cookie"
                      ? "bg-dit-accent/10 border-dit-accent text-dit-accent"
                      : "bg-dit-surface border-dit-border text-dit-text-muted hover:text-dit-text"
                  }`}
                >
                  Browser Cookie
                </button>
                <button
                  onClick={() => setAuthMethod("email")}
                  className={`flex-1 px-3 py-2 rounded-lg text-sm font-medium transition-colors border ${
                    authMethod === "email"
                      ? "bg-dit-accent/10 border-dit-accent text-dit-accent"
                      : "bg-dit-surface border-dit-border text-dit-text-muted hover:text-dit-text"
                  }`}
                >
                  Email + Password
                </button>
              </div>
              {authMethod === "cookie" ? (
                <div>
                  <input
                    type="password"
                    value={authCookie}
                    onChange={(e) => setAuthCookie(e.target.value)}
                    placeholder="Paste your Figma auth cookie..."
                    className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                               text-dit-text placeholder:text-dit-text-muted text-sm
                               focus:outline-none focus:border-dit-accent transition-colors"
                  />
                  <p className="text-dit-text-muted text-xs mt-2">
                    Copy from your browser's cookies for figma.com (cookie name: __Host-figma.authn)
                  </p>
                </div>
              ) : (
                <div className="space-y-3">
                  <input
                    type="email"
                    value={authEmail}
                    onChange={(e) => setAuthEmail(e.target.value)}
                    placeholder="Figma email address"
                    className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                               text-dit-text placeholder:text-dit-text-muted text-sm
                               focus:outline-none focus:border-dit-accent transition-colors"
                  />
                  <input
                    type="password"
                    value={authPassword}
                    onChange={(e) => setAuthPassword(e.target.value)}
                    placeholder="Figma password"
                    className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                               text-dit-text placeholder:text-dit-text-muted text-sm
                               focus:outline-none focus:border-dit-accent transition-colors"
                  />
                </div>
              )}
            </div>
            <div className="flex gap-3">
              <button
                onClick={() => {
                  // Skip auth — open without credentials (restore-only mode)
                  onRepoOpened(folderPath);
                }}
                className="px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                           text-dit-text-muted hover:text-dit-text transition-colors text-sm"
              >
                Skip
              </button>
              <button
                onClick={async () => {
                  if (!hasValidAuth) return;
                  setLoading(true);
                  setError(null);
                  try {
                    await saveCredentials(
                      authMethod === "cookie" ? authCookie : null,
                      authMethod === "email" ? authEmail : null,
                      authMethod === "email" ? authPassword : null,
                    );
                    onRepoOpened(folderPath);
                  } catch (e) {
                    setError(String(e));
                  } finally {
                    setLoading(false);
                  }
                }}
                disabled={!hasValidAuth || loading}
                className="flex-1 px-4 py-3 bg-dit-accent text-white rounded-lg font-medium
                           hover:bg-dit-accent-hover transition-colors text-sm
                           disabled:opacity-40 disabled:cursor-not-allowed"
              >
                Save & Open
              </button>
            </div>
          </div>
        )}

        {/* Step: Figma File Link */}
        {step === "figma-file" && (
          <div className="space-y-4">
            <div className="flex items-center gap-2 px-3 py-2 bg-green-500/10 border border-green-500/30 rounded-lg">
              <span className="text-green-400 text-sm">Auth configured</span>
              <span className="text-dit-text text-sm font-medium">
                {authMethod === "cookie" ? "Cookie" : authEmail}
              </span>
            </div>

            <div>
              <label className="block text-dit-text text-sm font-medium mb-2">
                Figma File Link
              </label>
              <input
                type="text"
                value={figmaLink}
                onChange={(e) => { setFigmaLink(e.target.value); setFileKey(""); }}
                placeholder="https://www.figma.com/design/ABC123/My-Design"
                className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                           text-dit-text placeholder:text-dit-text-muted text-sm
                           focus:outline-none focus:border-dit-accent transition-colors"
                onKeyDown={(e) => e.key === "Enter" && handleFileKeyExtract()}
              />
              <p className="text-dit-text-muted text-xs mt-2">
                Right-click a file in Figma, then "Copy link"
              </p>
            </div>

            {!fileKey && (
              <button
                onClick={handleFileKeyExtract}
                disabled={!figmaLink.trim()}
                className="w-full px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                           text-dit-text hover:border-dit-accent transition-colors text-sm
                           disabled:opacity-40 disabled:cursor-not-allowed"
              >
                Extract File Key
              </button>
            )}

            {fileKey && (
              <div className="space-y-3">
                <div className="rounded-lg border border-dit-accent/50 px-4 py-3 bg-dit-surface">
                  <p className="text-dit-text text-sm">
                    File key: <span className="font-mono text-dit-accent">{fileKey}</span>
                  </p>
                </div>
                <div>
                  <label className="block text-dit-text text-sm font-medium mb-2">
                    Project Name
                  </label>
                  <input
                    type="text"
                    value={fileName}
                    onChange={(e) => setFileName(e.target.value)}
                    placeholder="My Design Project"
                    className="w-full bg-dit-surface border border-dit-border rounded-lg px-4 py-3
                               text-dit-text placeholder:text-dit-text-muted text-sm
                               focus:outline-none focus:border-dit-accent transition-colors"
                  />
                </div>
              </div>
            )}

            {/* Overwrite confirmation */}
            {confirmOverwrite && (
              <div className="px-4 py-3 bg-red-500/10 border border-red-500/30 rounded-lg space-y-3">
                <p className="text-dit-danger text-sm font-medium">
                  This folder already contains a repository.
                </p>
                <p className="text-dit-text-muted text-xs">
                  Re-initializing will delete all existing version history in this directory. This action is irreversible.
                </p>
                <div className="flex gap-2">
                  <button
                    onClick={() => setConfirmOverwrite(false)}
                    className="flex-1 px-3 py-2 bg-dit-surface border border-dit-border rounded-lg
                               text-dit-text text-sm hover:border-dit-accent transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={() => doInit(true)}
                    className="flex-1 px-3 py-2 bg-red-600 text-white rounded-lg text-sm font-medium
                               hover:bg-red-700 transition-colors"
                  >
                    Delete and Re-initialize
                  </button>
                </div>
              </div>
            )}

            <div className="flex gap-3">
              <button
                onClick={() => { setStep("figma-auth"); setError(null); setConfirmOverwrite(false); }}
                className="px-4 py-3 bg-dit-surface border border-dit-border rounded-lg
                           text-dit-text-muted hover:text-dit-text transition-colors text-sm"
              >
                Back
              </button>
              <button
                onClick={handleCreateRepo}
                disabled={!fileKey || !fileName.trim() || loading || confirmOverwrite}
                className="flex-1 px-4 py-3 bg-dit-accent text-white rounded-lg font-medium
                           hover:bg-dit-accent-hover transition-colors text-sm
                           disabled:opacity-40 disabled:cursor-not-allowed"
              >
                Initialize Repository
              </button>
            </div>
          </div>
        )}

        {/* Step: Creating */}
        {step === "creating" && (
          <div className="text-center py-8">
            <div className="animate-spin w-8 h-8 border-2 border-dit-accent border-t-transparent rounded-full mx-auto mb-4" />
            <p className="text-dit-text text-sm">Initializing repository...</p>
          </div>
        )}
      </div>
    </div>
  );
}
