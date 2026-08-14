#!/bin/sh
set -eu

command -v curl >/dev/null 2>&1 || {
    printf '%s\n' 'curl is required.' >&2
    exit 1
}

# Vibe Coding guide by Nicolas Zullo (EnzeD/vibe-coding), pinned revision.
# The guide is a single README hosted upstream and is fetched read-only.
vibe_repo="EnzeD/vibe-coding"
vibe_rev="8b650568ea41515950f75bee255d03ac96db8e62"

dest_dir="${1:-./_vibe-coding}"
dest_file="$dest_dir/vibe-coding.md"

mkdir -p "$dest_dir"
printf '%s\n' "Fetching $vibe_repo@$vibe_rev into $dest_file ..."
curl --fail --location --silent --show-error \
    --output "$dest_file" \
    "https://raw.githubusercontent.com/$vibe_repo/$vibe_rev/README.md"
printf '%s\n' "Saved $dest_file ($(wc -l < "$dest_file") lines)."
printf '%s\n' 'The fetched guide is local tooling and intentionally ignored by Git.'