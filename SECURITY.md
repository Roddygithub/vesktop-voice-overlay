# Security Policy

## Supported Versions

The project has no released version yet.

## Reporting

Report vulnerabilities privately through GitHub's private vulnerability
reporting feature. Do not open a public issue containing credentials, Discord
tokens, personal voice-channel data, or a working exploit.

## Security Model

The overlay must not access the Discord account token. Its local bridge must
export only the active voice participants and speaking state required for
display, through a user-private Unix socket under `$XDG_RUNTIME_DIR`.
