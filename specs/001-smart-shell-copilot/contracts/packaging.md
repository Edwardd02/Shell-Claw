# Contract: Packaging, Service, and Hook Lifecycle

## Install Contract

After package-manager install on supported platforms:

- Daemon binary is installed in a stable package-managed location.
- Service registration is created for the current user.
- Service starts automatically.
- New supported shell sessions load the hook automatically without manual user
  edits to `.zshrc`, `.bashrc`, or equivalent personal startup files.
- If the daemon is not reachable, the shell hook silently falls back to native
  shell behavior.

## Service Manager Contract

- macOS uses launchd through Homebrew Services.
- Linux uses user-level systemd through Homebrew Services where available.
- Service must define socket path, model path/config path, diagnostics path, and
  restart policy.
- Service start failure must be visible to service diagnostics, not terminal hook
  output.

## Shell Loading Contract

- Hook load point must be package-managed and reversible.
- Hook file must perform a fast daemon/socket probe with a short timeout.
- Hook must no-op silently when unsupported shell mode or unavailable daemon is
  detected.

## Uninstall Contract

Package-manager uninstall must:

- Stop the daemon service.
- Remove service registration.
- Remove shell hook load integration.
- Leave no running daemon process.
- Leave no active socket file owned by the service.
- Avoid deleting unrelated user shell configuration.

## Validation Matrix

- macOS Intel + Zsh
- macOS Intel + Bash
- macOS Apple Silicon + Zsh
- macOS Apple Silicon + Bash
- Ubuntu Linux + Zsh
- Ubuntu Linux + Bash
- Arch Linux + Zsh
- Arch Linux + Bash
