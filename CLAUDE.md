# CLAUDE.md — japinput

This file provides guidance for AI assistants (including Claude Code) working in this repository.

## Project Overview

**japinput** is a Japanese input project. The repository is in its early stages of development.

- **License:** MIT (Copyright 2026 shien)
- **Default branch:** `main`

## Repository Structure

```
japinput/
├── LICENSE          # MIT license
└── CLAUDE.md        # This file — AI assistant guide
```

This is a newly initialized repository. As the project grows, update this section to reflect the directory layout.

## Development Setup

No build system, dependencies, or tooling have been configured yet. When they are added, document the setup steps here:

```sh
# Example (update when applicable):
# git clone <repo-url>
# cd japinput
# <install dependencies command>
# <build command>
```

## Common Commands

No commands are configured yet. As scripts are added (e.g., in `package.json`, `Makefile`, or similar), list them here:

| Command | Description |
|---------|-------------|
| _TBD_   | _TBD_       |

## Code Conventions

No code has been written yet. When development begins, document:

- **Language(s):** _(e.g., TypeScript, Rust, Python)_
- **Formatting/Linting:** _(e.g., Prettier, ESLint, rustfmt)_
- **Testing framework:** _(e.g., Jest, pytest, cargo test)_
- **Style guide:** _(any project-specific conventions)_

## Git Workflow

- The default branch is `main`.
- Feature branches should use descriptive names (e.g., `feat/romaji-to-kana`, `fix/input-lag`).
- Write clear, concise commit messages that explain the "why" behind changes.

## Guidelines for AI Assistants

- **Read before modifying.** Always read existing files before proposing changes.
- **Keep changes minimal.** Only modify what is necessary for the task at hand.
- **Do not over-engineer.** Avoid adding abstractions, utilities, or features beyond what is explicitly requested.
- **Preserve existing conventions.** Match the style, formatting, and patterns already present in the codebase.
- **No unnecessary files.** Do not create documentation, config files, or boilerplate unless explicitly asked.
- **Security first.** Do not introduce command injection, XSS, SQL injection, or other common vulnerabilities.
- **Update this file.** When new tooling, structure, or conventions are added to the project, update this CLAUDE.md to reflect the current state.
