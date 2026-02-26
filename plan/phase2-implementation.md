# Phase 2 実装計画: SKK 辞書の読み込みと検索

## 概要

Phase 1（ローマ字→かな変換、68テスト全パス）の上に、SKK 辞書ファイルの読み込み・検索機能を追加する。
TDD サイクル（Red → Green → Refactor）を厳守し、各ステップで動作確認を行う。

---

## ステップ 1: プロジェクト準備

### 1-1. `encoding_rs` クレートの追加

- `Cargo.toml` の `[dependencies]` に `encoding_rs = "0.8"` を追加
- **動作確認:** `cargo build` がエラーなく完了すること

### 1-2. `.gitignore` に `dict/` を追加

- `.gitignore` に `dict/` の行を追加（大容量辞書ファイルの除外）
- **動作確認:** `git status` で `dict/` ディレクトリがトラッキング対象外であること

### 1-3. テスト用辞書ファイルの作成

- `tests/fixtures/test_dict.txt` を作成
- UTF-8 の SKK-JISYO 形式で以下を含む:
  - コメント行（`;` で始まる行）
  - 空行
  - 通常エントリ（例: `かんじ /漢字/感じ/幹事/`）
  - アノテーション付きエントリ（例: `にほん /日本;country/二本/`）
  - 送り仮名付きエントリ（例: `おおきi /大き/`）
  - 単一候補のエントリ
  - 前方一致テスト用に共通プレフィクスを持つ複数エントリ（例: `かん`、`かんじ`、`かんこく`）
- **動作確認:** ファイルが正しい SKK 辞書形式であること（目視確認）

### 1-4. モジュール登録

- `src/lib.rs` に `pub mod dictionary;` を追加
- `src/dictionary.rs` を空ファイルとして作成（コンパイル通過のため）
- **動作確認:** `cargo build` がエラーなく完了し、既存の 68 テストが全パスすること

---

## ステップ 2: 辞書行パーサー (TDD)

SKK 辞書の 1 行をパースする関数を実装する。

### 2-1. Red: パーサーのテストを書く

`src/dictionary.rs` に以下のテストを追加:

```
#[cfg(test)]
mod tests {
    use super::*;

    // === 行パーサー ===

    #[test]
    fn parse_normal_entry()
    // "かんじ /漢字/感じ/幹事/" → Some(("かんじ", vec!["漢字", "感じ", "幹事"]))

    #[test]
    fn parse_single_candidate()
    // "にほん /日本/" → Some(("にほん", vec!["日本"]))

    #[test]
    fn parse_annotation()
    // "にほん /日本;country/二本/" → Some(("にほん", vec!["日本", "二本"]))
    // アノテーション（;以降）は除去して候補のみ返す

    #[test]
    fn parse_comment_line()
    // ";; これはコメント" → None

    #[test]
    fn parse_empty_line()
    // "" → None

    #[test]
    fn parse_okurigana_entry()
    // "おおきi /大き/" → Some(("おおきi", vec!["大き"]))
    // 送り仮名付きエントリもそのまま保持
}
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 2-2. Green: パーサーを実装

`src/dictionary.rs` に `parse_line` 関数を実装:

```rust
/// SKK 辞書の1行をパースする。
/// 読みと候補リストを返す。コメント行・空行は None。
fn parse_line(line: &str) -> Option<(String, Vec<String>)>
```

処理内容:
- 空行 → `None`
- `;` で始まる行 → `None`
- 読みと候補部分を最初の空白で分割
- `/` で区切って候補を抽出
- 各候補の `;` 以降（アノテーション）を除去
- 空でない候補を `Vec<String>` として返す

- **動作確認:** `cargo test` で全テスト（既存68 + 新規パーサーテスト）がパスすること（Green）

### 2-3. Refactor

- 不要なコードの整理
- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 3: Dictionary 構造体 — 基本検索 (TDD)

### 3-1. Red: `Dictionary` 構造体と `lookup` のテストを書く

```
    // === Dictionary 構造体 ===

    #[test]
    fn lookup_found()
    // テスト辞書で "かんじ" を検索 → vec!["漢字", "感じ", "幹事"]

    #[test]
    fn lookup_not_found()
    // テスト辞書で "そんざいしない" を検索 → None (または空)

    #[test]
    fn lookup_single_candidate()
    // テスト辞書で単一候補エントリを検索 → vec!["..."]
```

`Dictionary` 構造体の公開 API:

```rust
pub struct Dictionary {
    entries: HashMap<String, Vec<String>>,
}

impl Dictionary {
    pub fn new() -> Self;
    pub fn lookup(&self, reading: &str) -> Option<&[String]>;
}
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 3-2. Green: `Dictionary` 構造体と `lookup` を実装

- `HashMap<String, Vec<String>>` で読み→候補リストを保持
- `lookup(reading)` → `Option<&[String]>` を返す
- テスト内では直接 `Dictionary` にエントリを追加して検証

- **動作確認:** `cargo test` で全テストがパスすること（Green）

### 3-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 4: ファイル読み込み (TDD)

### 4-1. Red: `load_from_file` のテストを書く

```
    // === ファイル読み込み ===

    #[test]
    fn load_from_utf8_file()
    // tests/fixtures/test_dict.txt を読み込み、既知のエントリを検索できること

    #[test]
    fn load_skips_comments_and_empty_lines()
    // tests/fixtures/test_dict.txt を読み込み、コメント行・空行が無視されていること
    // (存在しないキーで検索して None を確認)

    #[test]
    fn load_nonexistent_file()
    // 存在しないファイルパスで load_from_file → Err を返すこと
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 4-2. Green: `load_from_file` を実装

```rust
impl Dictionary {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, DictionaryError>;
}
```

処理内容:
- ファイルをバイト列として読み込む
- `encoding_rs` で EUC-JP デコードを試み、BOM / ヘッダー判定で UTF-8 との切り替え
  - 実際の判定ロジック: ファイル先頭バイトが有効な UTF-8 ならそのまま、そうでなければ EUC-JP としてデコード
- 各行を `parse_line` でパースし、`HashMap` に挿入
- 同じ読みが複数行にある場合は候補を連結（`extend`）

エラー型:

```rust
#[derive(Debug)]
pub enum DictionaryError {
    Io(std::io::Error),
}
```

- **動作確認:** `cargo test` で全テストがパスすること（Green）

### 4-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 5: 前方一致検索 (TDD)

### 5-1. Red: `lookup_prefix` のテストを書く

```
    // === 前方一致検索 ===

    #[test]
    fn lookup_prefix_found()
    // "かん" で前方一致 → "かんじ", "かんこく" など複数ヒット

    #[test]
    fn lookup_prefix_no_match()
    // "zzz" で前方一致 → 空の結果

    #[test]
    fn lookup_prefix_exact_match_included()
    // "かんじ" で前方一致 → "かんじ" 自身もヒットに含まれる
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 5-2. Green: `lookup_prefix` を実装

```rust
impl Dictionary {
    pub fn lookup_prefix(&self, prefix: &str) -> Vec<(&str, &[String])>;
}
```

処理内容:
- `HashMap` の全キーを走査し、`prefix` で始まるエントリを収集
- `Vec<(&str, &[String])>` として返す（読みと候補リストのペア）
- キーのアルファベット順（UTF-8 辞書順）でソートして返す

> 注: Phase 2 時点では HashMap の全走査で十分。大辞書でのパフォーマンスが問題になった場合、後のフェーズで `BTreeMap` や `trie` に変更する。

- **動作確認:** `cargo test` で全テストがパスすること（Green）

### 5-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 6: EUC-JP 辞書の読み込みテスト (TDD)

### 6-1. Red: EUC-JP テストを書く

```
    // === EUC-JP 対応 ===

    #[test]
    fn load_from_eucjp_file()
    // tests/fixtures/test_dict_eucjp.bin を読み込み、正しくデコードされること
```

- `tests/fixtures/test_dict_eucjp.bin`: EUC-JP エンコードのテスト辞書ファイルを Rust コード内で生成（`encoding_rs` で UTF-8 → EUC-JP エンコードしたバイト列を一時ファイルに書き出してテスト）
- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 6-2. Green: EUC-JP 判定ロジックを完成

- ファイル読み込み時の UTF-8 / EUC-JP 自動判定を実装
  - まず UTF-8 としてデコードを試みる（`std::str::from_utf8`）
  - 失敗したら `encoding_rs::EUC_JP` でデコード
- **動作確認:** `cargo test` で全テストがパスすること（Green）

### 6-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 7: CLI 辞書検索モード (TDD)

### 7-1. CLI に辞書検索モードを追加

`src/main.rs` を拡張:

- コマンドライン引数で辞書ファイルパスを受け取る: `cargo run -- --dict path/to/dict.txt`
- 辞書ファイルが指定された場合:
  1. ローマ字入力 → ひらがな変換
  2. ひらがなで辞書検索
  3. 変換候補を表示
- 辞書ファイルが指定されない場合: 既存のローマ字→かな変換デモをそのまま動作

出力例:
```
> kanji
  ひらがな: かんじ
  カタカナ: カンジ
  変換候補: 漢字 / 感じ / 幹事
```

- **動作確認:** `cargo build` がエラーなく完了すること。`cargo run -- --dict tests/fixtures/test_dict.txt` で手動確認

---

## ステップ 8: 最終確認・コミット

### 8-1. 全テスト実行

```sh
cargo test
```

- 既存の 68 テスト + 新規辞書テスト（約 15 テスト）が全パスすること

### 8-2. コード品質チェック

```sh
cargo clippy
cargo fmt -- --check
```

- 警告なし、フォーマット差分なしであること

### 8-3. `lib.rs` の確認

- `pub mod dictionary;` が追加されていること

### 8-4. CLAUDE.md の更新

- ファイル構成に `src/dictionary.rs`、`tests/fixtures/`、`dict/` を追記
- 依存クレートに `encoding_rs` を追記

### 8-5. コミット

- テスト全パスを確認した上でコミット

---

## ファイル変更一覧

| ファイル | 操作 | 内容 |
|---------|------|------|
| `Cargo.toml` | 編集 | `encoding_rs = "0.8"` 追加 |
| `.gitignore` | 編集 | `dict/` 追加 |
| `src/lib.rs` | 編集 | `pub mod dictionary;` 追加 |
| `src/dictionary.rs` | **新規** | パーサー、Dictionary 構造体、検索 |
| `src/main.rs` | 編集 | 辞書検索モード追加 |
| `tests/fixtures/test_dict.txt` | **新規** | UTF-8 テスト辞書 |
| `CLAUDE.md` | 編集 | 構成・依存情報更新 |

## 追加テスト一覧（予定）

| # | テスト名 | 分類 | 内容 |
|---|---------|------|------|
| 1 | `parse_normal_entry` | 行パーサー | 複数候補の通常行 |
| 2 | `parse_single_candidate` | 行パーサー | 単一候補 |
| 3 | `parse_annotation` | 行パーサー | アノテーション除去 |
| 4 | `parse_comment_line` | 行パーサー | コメント行スキップ |
| 5 | `parse_empty_line` | 行パーサー | 空行スキップ |
| 6 | `parse_okurigana_entry` | 行パーサー | 送り仮名付き |
| 7 | `lookup_found` | 検索 | 完全一致ヒット |
| 8 | `lookup_not_found` | 検索 | 完全一致ミス |
| 9 | `lookup_single_candidate` | 検索 | 単一候補検索 |
| 10 | `load_from_utf8_file` | ファイル | UTF-8 読み込み |
| 11 | `load_skips_comments_and_empty_lines` | ファイル | コメント・空行スキップ |
| 12 | `load_nonexistent_file` | ファイル | 存在しないファイル |
| 13 | `lookup_prefix_found` | 前方一致 | プレフィクス検索ヒット |
| 14 | `lookup_prefix_no_match` | 前方一致 | プレフィクス検索ミス |
| 15 | `lookup_prefix_exact_match_included` | 前方一致 | 完全一致も含む |
| 16 | `load_from_eucjp_file` | EUC-JP | EUC-JP デコード |

## 依存クレート

| クレート | バージョン | 用途 |
|---------|-----------|------|
| `encoding_rs` | 0.8 | EUC-JP 辞書のデコード |
