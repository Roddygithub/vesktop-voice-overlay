# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.0.0   | Yes       |

## Reporting

Contact the repository owner privately before sharing vulnerability details.
Do not open an issue containing credentials, Discord tokens, personal
voice-channel data, or a working exploit. GitHub private vulnerability
reporting is not enabled while this repository remains private on its current
account plan.

## Security Model

The overlay must not access the Discord account token. Its local bridge must
export only the active voice participants and speaking state required for
display, through a user-private Unix socket under `$XDG_RUNTIME_DIR`.
