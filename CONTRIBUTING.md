# Contributing

Discuss significant architecture or protocol changes in an issue before
implementation.

## Requirements

- Follow `AGENTS.md` security and licensing constraints.
- Keep changes focused and include tests for changed behavior.
- Do not include Discord tokens, private voice data, logs, or user identifiers.
- Preserve attribution when adapting upstream GPL-3.0 source.
- Use clear commits.

## Development Setup

### Plugin (TypeScript)

```bash
cd plugin && npm ci
npm run typecheck && npm run lint && npm test
```

### Overlay (Rust)

```bash
cd overlay && cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### Build Release

```bash
cd overlay && cargo build --release
cd ../plugin && npm run build && npm pack
```
