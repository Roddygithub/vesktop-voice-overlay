#!/bin/sh
set -eu

command -v node >/dev/null 2>&1 || {
    printf '%s\n' 'Node.js 20.12 or newer is required.' >&2
    exit 1
}
command -v npx >/dev/null 2>&1 || {
    printf '%s\n' 'npm/npx is required.' >&2
    exit 1
}
command -v uv >/dev/null 2>&1 || {
    printf '%s\n' 'uv is required: https://docs.astral.sh/uv/' >&2
    exit 1
}

user_name=${BMAD_USER_NAME:-${USER:-Developer}}

npx --yes bmad-method@6.11.0 install \
    --directory . \
    --modules bmm \
    --tools opencode \
    --yes \
    --set "core.user_name=$user_name" \
    --set core.communication_language=French \
    --set core.document_output_language=French
