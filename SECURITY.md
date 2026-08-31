# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.2.0   | Yes       |
| 1.1.0   | No        |
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

The transmitted participant fields are Discord user ID, display name, avatar
URL, mute/deaf state, and speaking state. Message content, guild/channel lists,
session identifiers, credentials, and account tokens are not read or sent.
Avatar HTTP requests are restricted to HTTPS URLs on `cdn.discordapp.com`, do
not follow redirects, and are subject to response and decode limits.
