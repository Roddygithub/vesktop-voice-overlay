# Contributing

The project is in its planning phase. Discuss significant architecture or
protocol changes in an issue before implementation.

## Requirements

- Follow `AGENTS.md` security and licensing constraints.
- Keep changes focused and include tests for changed behavior.
- Do not include Discord tokens, private voice data, logs, or user identifiers.
- Preserve attribution when adapting upstream GPL-3.0 source.
- Use clear commits and keep generated BMAD files out of version control.

## Development Setup

Install Node.js 20.12 or newer, npm/npx, Git, Python 3, and `uv`, then run:

```bash
./scripts/bootstrap-bmad.sh
```

Application build and test commands will be documented after the BMAD
architecture phase selects the implementation stack.
