# CLAUDE.md — japinput

This file provides guidance for AI assistants (including Claude Code) working in this repository.

## Project Overview

**japinput** is a Windows 向け日本語入力システム (IME) written in Rust.

- **License:** MIT (Copyright 2026 shien)
- **Default branch:** `main`

## Repository Structure

```
japinput/
├── Cargo.toml       # Rust package manifest
├── src/
│   ├── lib.rs       # Crate root (module declarations)
│   └── romaji.rs    # ローマ字 → ひらがな変換
├── LICENSE
└── CLAUDE.md
```

## Development Setup

```sh
git clone <repo-url>
cd japinput
cargo build
```

## Common Commands

| Command | Description |
|---------|-------------|
| `cargo build` | プロジェクトをビルドする |
| `cargo test` | 全テストを実行する |
| `cargo test -- --nocapture` | テスト実行時に stdout を表示する |
| `cargo test <test_name>` | 特定のテストのみ実行する |
| `cargo clippy` | lint チェックを実行する |
| `cargo fmt` | コードをフォーマットする |
| `cargo fmt -- --check` | フォーマット差分があるかチェックする（CI向け） |

## Code Conventions

- **Language:** Rust (Edition 2024)
- **Formatting:** `rustfmt` (デフォルト設定)
- **Linting:** `clippy`
- **Testing:** `cargo test` (Rust 標準のテストフレームワーク)

## Testing Rules

以下のルールに従ってテストを書くこと。

### テスト実行

- コードを変更したら `cargo test` で全テストが通ることを必ず確認する。
- テストが失敗した状態でコミットしてはならない。

### テストの配置

- **ユニットテスト**は、対象モジュールと同じファイル内の `#[cfg(test)] mod tests { ... }` ブロックに書く。
- **結合テスト**（将来必要になった場合）は `tests/` ディレクトリに配置する。

### テストの書き方

- テスト関数名は `snake_case` で、テスト対象の挙動を簡潔に表す名前にする。
  - 良い例: `fn sokuon_kk()`, `fn n_before_consonant()`
  - 悪い例: `fn test1()`, `fn it_works()`
- テストはカテゴリごとにコメントで区切る（例: `// === 促音（っ） ===`）。
- `assert_eq!` を基本とし、期待値を右辺に書く: `assert_eq!(actual, expected)`。
- 新しい機能を追加するときは、最低限以下のテストを含める:
  - 正常系（基本的な入力）
  - エッジケース（空入力、境界値など）
  - 既知の特殊ケース（その機能固有の注意点）

### テスト追加のタイミング

- 新しい public 関数には必ずテストを書く。
- バグを修正するときは、そのバグを再現するテストを先に書く（回帰テスト）。
- 変換テーブルにエントリを追加したら、対応するテストも追加する。

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
- **Run tests.** Before committing, always run `cargo test` and confirm all tests pass.
- **Update this file.** When new tooling, structure, or conventions are added to the project, update this CLAUDE.md to reflect the current state.
