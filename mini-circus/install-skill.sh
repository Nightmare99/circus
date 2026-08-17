#!/bin/sh
# Installs the mini-circus Claude Code skill into the CURRENT project.
#
#   curl -fsSL https://raw.githubusercontent.com/Nightmare99/circus/main/mini-circus/install-skill.sh | sh
#
# Detects (or creates) a .claude/skills/ directory in the working directory
# you run this from, and downloads SKILL.md + reference.md into
# .claude/skills/mini-circus/ so Claude Code picks it up in that project.
# This only installs the skill (agent instructions) - it does not install
# the mini-circus binary itself; see mini-circus/install.sh for that.
#
# Env vars:
#   MINI_CIRCUS_SKILL_REF  git ref to install from (default: main)
#   MINI_CIRCUS_SKILL_DIR  target skills directory (default: ./.claude/skills)
set -eu

REPO="Nightmare99/circus"
REF="${MINI_CIRCUS_SKILL_REF:-main}"
SKILLS_DIR="${MINI_CIRCUS_SKILL_DIR:-.claude/skills}"
TARGET_DIR="$SKILLS_DIR/mini-circus"
# Source of truth lives at mini-circus/skill/ in the repo (not under .claude/
# there - that directory is this repo's own, unrelated to what gets
# installed elsewhere). Same Claude Skill format either way.
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

# "Detect" the skills folder: report what's already there before touching
# anything, since that's the whole point of running this from an arbitrary
# project directory rather than a fixed location.
if [ -d "$SKILLS_DIR" ]; then
    say "Found existing skills directory: $(pwd)/$SKILLS_DIR"
else
    say "No $SKILLS_DIR in $(pwd) yet - creating it."
fi

ALREADY_INSTALLED=false
[ -f "$TARGET_DIR/SKILL.md" ] && ALREADY_INSTALLED=true

mkdir -p "$TARGET_DIR"

fetch() {
    # $1 = filename relative to the skill's own directory in the repo
    if ! curl -fsSL "$RAW_BASE/$1" -o "$TARGET_DIR/$1"; then
        err "failed to download $1 from $RAW_BASE/$1"
    fi
}

fetch "SKILL.md"
fetch "reference.md"

if [ "$ALREADY_INSTALLED" = true ]; then
    say "Updated mini-circus skill in $TARGET_DIR"
else
    say "Installed mini-circus skill to $TARGET_DIR"
fi

say ""
say "Claude Code sessions started in this project will pick it up automatically."

if ! command -v mini-circus >/dev/null 2>&1; then
    say ""
    say "Note: the mini-circus binary itself isn't on your PATH yet. Install it with:"
    say "  curl -fsSL https://raw.githubusercontent.com/$REPO/$REF/mini-circus/install.sh | sh"
fi
