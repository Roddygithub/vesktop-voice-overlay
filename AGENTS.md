# Engineering Constraints

## Mission

Build a reliable Wayland voice activity overlay for Vesktop while minimizing
access to Discord data and avoiding changes to the game process.

## Architecture

- Prefer a small Vencord plugin that reads existing renderer voice stores.
- Send authoritative, versioned snapshots through a Unix socket owned by the
  overlay under `$XDG_RUNTIME_DIR`.
- Keep the overlay independent from Vesktop lifecycle and reconnect safely.
- Use GTK and layer-shell unless validated requirements justify another stack.
- Do not implement Discord's deprecated extended RPC protocol unless the local
  snapshot design is proven insufficient.

## Security

- Never read, log, persist, transmit, or request a Discord account token.
- Never implement a self-bot or a separate Discord Gateway connection.
- Do not export messages, guild lists, channel lists, or session identifiers.
- Restrict local IPC to the current user and validate all payloads and sizes.
- Do not commit credentials, captured voice data, avatars, or personal logs.

## Process Safety

- Never use `pgrep -f` or `pkill -f` — full command-line matching risks killing
  unrelated processes (e.g., Herdr, OpenCode, shell sessions) that happen to
  contain the project name in their arguments.
- Prefer exact process-name matching (`pgrep -x`) when the binary name is unique.
- If the binary name is ambiguous, inspect `ps` output first and kill only the
  confirmed PID.
- Before any destructive process command (`kill`, `pkill`, `killall`), print the
  matching process list first so the operator can confirm.
- Never kill Herdr, OpenCode, the shell/session host, or any process merely
  because its arguments contain the project name.

## Quality

- Keep protocol types explicit and versioned.
- Treat full snapshots as authoritative; do not rely only on edge events.
- Add focused tests for protocol validation, reconnects, and stale-state cleanup.
- Measure overlay overhead before optimizing or changing game configuration.
- Validate speaking state in a real multi-user call before declaring support.

## Licensing

- The repository is GPL-3.0 to remain compatible with planned Discover reuse.
- Preserve upstream notices and identify modified imported files.
- Record the exact upstream repository and revision for copied or adapted code.

## Tooling

- The `memory-bank/` tree holds the durable PRD, tech stack, implementation
  plan, progress, and architecture documents.

> **Always read before writing code:** `memory-bank/prd.md`,
> `memory-bank/tech-stack.md`, `memory-bank/implementation-plan.md`. After each
> validated step, update `memory-bank/progress.md` and
> `memory-bank/architecture.md`.
