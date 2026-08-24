# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.1.0   | Yes       |
| 1.0.0   | No        |

## Reporting

Please use GitHub **private vulnerability reporting** (Security → Report a
vulnerability), which is enabled on this repository. Do not open a public
issue containing credentials, Discord tokens, personal voice-channel data,
or a working exploit.

## Security Model

The overlay must not access the Discord account token. Its local bridge must
export only the active voice participants and speaking state required for
display, through a user-private Unix socket under `$XDG_RUNTIME_DIR`.
