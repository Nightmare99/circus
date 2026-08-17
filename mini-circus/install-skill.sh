#!/bin/sh
# Installs the mini-circus skill into every <dotfolder>/skills/ directory
# found in the current working directory - .claude/skills, .codex/skills,
# or any other tool following the same "dotfolder holding a skills/
# subfolder" convention - not just Claude Code specifically. Falls back to
# creating .claude/skills/ if none exist yet.
#
#   curl -fsSL https://raw.githubusercontent.com/Nightmare99/circus/main/mini-circus/install-skill.sh | sh
#
# This only installs the skill (agent instructions) - it does not install
# the mini-circus binary itself; see mini-circus/install.sh for that.
#
# Env vars:
#   MINI_CIRCUS_SKILL_REF  git ref to install from (default: main)
#   MINI_CIRCUS_SKILL_DIR  force a specific target skills directory
#                          (e.g. .claude/skills), skipping the scan
set -eu

REPO="Nightmare99/circus"
REF="${MINI_CIRCUS_SKILL_REF:-main}"
RAW_BASE="https://raw.githubusercontent.com/$REPO/$REF/mini-circus/skill"

say() { printf '%s\n' "$*"; }
err() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}
need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "need '$1' but it's not installed"
}

need_cmd curl
need_cmd mkdir

fetch() {
    # $1 = target skill dir (e.g. .claude/skills/mini-circus), $2 = filename
    if ! curl -fsSL "$RAW_BASE/$2" -o "$1/$2"; then
        err "failed to download $2 from $RAW_BASE/$2"
    fi
}

# $1 = a "skills" directory to install into (e.g. .claude/skills)
install_into() {
    skills_dir="$1"
    target_dir="$skills_dir/mini-circus"
    already=false
    [ -f "$target_dir/SKILL.md" ] && already=true

    mkdir -p "$target_dir"
    fetch "$target_dir" "SKILL.md"
    fetch "$target_dir" "reference.md"

    if [ "$already" = true ]; then
        say "Updated mini-circus skill in $target_dir"
    else
        say "Installed mini-circus skill to $target_dir"
    fi
}

if [ -n "${MINI_CIRCUS_SKILL_DIR:-}" ]; then
    install_into "$MINI_CIRCUS_SKILL_DIR"
else
    found_any=false
    for entry in .*/; do
        if [ "$entry" = "./" ] || [ "$entry" = "../" ]; then
            continue
        fi
        if [ -d "${entry}skills" ]; then
            found_any=true
            say "Found $(pwd)/${entry}skills"
            install_into "${entry}skills"
        fi
    done

    if [ "$found_any" = false ]; then
        say "No existing <dotfolder>/skills directory in $(pwd) - defaulting to .claude/skills"
        install_into ".claude/skills"
    fi
fi

say ""
say "Any tool that reads skills from the directory/directories above will pick this up in this project."

if ! command -v mini-circus >/dev/null 2>&1; then
    say ""
    say "Note: the mini-circus binary itself isn't on your PATH yet. Install it with:"
    say "  curl -fsSL https://raw.githubusercontent.com/$REPO/$REF/mini-circus/install.sh | sh"
fi
