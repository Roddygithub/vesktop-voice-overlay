---
name: Bug report
about: Report a problem with the overlay
title: ''
labels: bug
assignees: ''
---

**Describe the bug**
A clear description of what happens vs what you expected.

**To reproduce**
Steps to reproduce the behavior.

**Environment** (please fill all fields)
- Distro: [e.g. Arch Linux, Omarchy 4.0]
- Compositor: [e.g. Hyprland 0.56, sway 1.10]
- Vesktop version: [output of `vesktop --version` or check Settings]
- Vencord: [built-in Vesktop Vencord or custom build? if custom, which revision?]
- Overlay version: [output of `vesktop-voice-overlay --version`]
- Session type: [Wayland / X11]
- Game tested (if applicable): [name + display mode: fullscreen/borderless/windowed]

**Logs**
<details>
<summary>Overlay journal</summary>

```
journalctl --user -u vesktop-voice-overlay.service --since "-30 min" --no-pager
```

Paste relevant output here.
</details>

**Additional context**
Any other information (screenshots, related issues, etc).
