# Phase 3 実装計画: 変換エンジンの統合

## 概要

Phase 1（ローマ字→かな変換、68テスト）と Phase 2（SKK 辞書検索、18テスト、合計86テスト全パス）の上に、
変換エンジンを構築する。SKK 方式の入力フロー（ローマ字入力→変換→候補選択→確定）を統合し、
状態機械・候補管理・コマンド体系を実装する。

TDD サイクル（Red → Green → Refactor）を厳守し、各ステップで動作確認を行う。

---

## アーキテクチャ設計

### 状態遷移図

```
         InsertChar            Convert              Commit
Direct ──────────→ Composing ──────────→ Converting ──────────→ Direct
                   ↑      │              ↑    │ ↑              │
                   │      │ Commit       │    │ │              │
                   │      ╰──→ Direct    │    │ │ Next/Prev    │
                   │         Cancel      │    ╰─╯              │
                   ╰─────────────────────╯                     │
                          Cancel                               │
```

### 状態と操作の対応

| 状態 | InsertChar | Convert | Next/Prev | Commit | Cancel | Backspace |
|------|-----------|---------|-----------|--------|--------|-----------|
| Direct | → Composing | 無視 | 無視 | 無視 | 無視 | 無視 |
| Composing | 文字追加 | → Converting | 無視 | ひらがな確定 → Direct | 破棄 → Direct | 1文字削除 |
| Converting | 無視 | Next と同じ | 候補移動 | 候補確定 → Direct | → Composing | 無視 |

### ファイル構成

```
src/
├── candidate.rs   # NEW: CandidateList 管理
├── engine.rs      # NEW: ConversionEngine, 状態機械, コマンド処理
├── lib.rs         # EDIT: pub mod candidate; pub mod engine; 追加
├── main.rs        # EDIT: ConversionEngine を使った CLI に更新
├── dictionary.rs  # 既存（変更なし）
├── input_state.rs # EDIT: backspace() メソッド追加
├── romaji.rs      # 既存（変更なし）
└── katakana.rs    # 既存（変更なし）
```

---

## ステップ 1: プロジェクト準備

### 1-1. モジュール登録

- `src/lib.rs` に `pub mod candidate;` と `pub mod engine;` を追加
- `src/candidate.rs` を空ファイルとして作成
- `src/engine.rs` を空ファイルとして作成
- **動作確認:** `cargo build` がエラーなく完了し、既存の 86 テストが全パスすること

---

## ステップ 2: CandidateList (TDD)

候補リストと選択インデックスを管理する構造体を実装する。

### 2-1. Red: CandidateList のテストを書く

`src/candidate.rs` に以下のテストを追加:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // === 基本操作 ===

    #[test]
    fn new_with_candidates() {
        let cl = CandidateList::new(vec!["漢字".to_string(), "感じ".to_string(), "幹事".to_string()]);
        assert_eq!(cl.current(), Some("漢字"));
        assert_eq!(cl.index(), 0);
        assert_eq!(cl.len(), 3);
    }

    #[test]
    fn new_empty() {
        let cl = CandidateList::new(vec![]);
        assert_eq!(cl.current(), None);
        assert!(cl.is_empty());
        assert_eq!(cl.len(), 0);
    }

    // === next / prev ===

    #[test]
    fn next_moves_forward() {
        let mut cl = CandidateList::new(vec!["漢字".to_string(), "感じ".to_string(), "幹事".to_string()]);
        cl.next();
        assert_eq!(cl.current(), Some("感じ"));
        assert_eq!(cl.index(), 1);
    }

    #[test]
    fn prev_moves_backward() {
        let mut cl = CandidateList::new(vec!["漢字".to_string(), "感じ".to_string(), "幹事".to_string()]);
        cl.next();
        cl.next();
        cl.prev();
        assert_eq!(cl.current(), Some("感じ"));
        assert_eq!(cl.index(), 1);
    }

    // === 境界値 ===

    #[test]
    fn next_at_end_wraps() {
        let mut cl = CandidateList::new(vec!["漢字".to_string(), "感じ".to_string()]);
        cl.next(); // index=1
        cl.next(); // index=0 (ラップ)
        assert_eq!(cl.current(), Some("漢字"));
        assert_eq!(cl.index(), 0);
    }

    #[test]
    fn prev_at_start_wraps() {
        let mut cl = CandidateList::new(vec!["漢字".to_string(), "感じ".to_string()]);
        cl.prev(); // index=1 (ラップ)
        assert_eq!(cl.current(), Some("感じ"));
        assert_eq!(cl.index(), 1);
    }

    #[test]
    fn next_on_empty_no_panic() {
        let mut cl = CandidateList::new(vec![]);
        cl.next(); // パニックしない
        assert_eq!(cl.current(), None);
    }

    #[test]
    fn prev_on_empty_no_panic() {
        let mut cl = CandidateList::new(vec![]);
        cl.prev(); // パニックしない
        assert_eq!(cl.current(), None);
    }

    // === select ===

    #[test]
    fn select_returns_current() {
        let mut cl = CandidateList::new(vec!["漢字".to_string(), "感じ".to_string()]);
        cl.next();
        assert_eq!(cl.select(), Some("感じ".to_string()));
    }

    #[test]
    fn select_on_empty() {
        let cl = CandidateList::new(vec![]);
        assert_eq!(cl.select(), None);
    }

    // === candidates() ===

    #[test]
    fn candidates_returns_all() {
        let cl = CandidateList::new(vec!["漢字".to_string(), "感じ".to_string()]);
        assert_eq!(cl.candidates(), &["漢字", "感じ"]);
    }
}
```

公開 API:

```rust
pub struct CandidateList {
    candidates: Vec<String>,
    index: usize,
}

impl CandidateList {
    pub fn new(candidates: Vec<String>) -> Self;
    pub fn current(&self) -> Option<&str>;
    pub fn next(&mut self);
    pub fn prev(&mut self);
    pub fn select(&self) -> Option<String>;
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
    pub fn index(&self) -> usize;
    pub fn candidates(&self) -> &[String];
}
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 2-2. Green: CandidateList を実装

実装内容:
- `new()`: candidates と index=0 で初期化
- `current()`: `candidates.get(index)` で現在の候補を返す
- `next()`: index を +1、末尾を超えたら 0 にラップ（空リストなら何もしない）
- `prev()`: index を -1、0 未満なら末尾にラップ（空リストなら何もしない）
- `select()`: `current()` の結果を `String` として返す
- `is_empty()` / `len()` / `index()` / `candidates()`: そのまま

- **動作確認:** `cargo test` で全テスト（既存86 + CandidateList 11テスト）がパスすること（Green）

### 2-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 3: InputState への backspace 追加 (TDD)

Composing 状態でのバックスペースに対応するため、InputState に機能を追加する。

### 3-1. Red: backspace テストを書く

`src/input_state.rs` のテストブロックに以下を追加:

```rust
    // === backspace ===

    #[test]
    fn backspace_removes_pending() {
        // pending がある場合は pending の末尾を削除
        let mut state = InputState::new();
        state.feed_char('k');
        assert_eq!(state.pending(), "k");
        state.backspace();
        assert_eq!(state.pending(), "");
        assert_eq!(state.output(), "");
    }

    #[test]
    fn backspace_removes_output_char() {
        // pending が空で output がある場合は output の末尾1文字を削除
        let mut state = InputState::new();
        state.feed_char('k');
        state.feed_char('a');
        assert_eq!(state.output(), "か");
        assert_eq!(state.pending(), "");
        state.backspace();
        assert_eq!(state.output(), "");
        assert_eq!(state.pending(), "");
    }

    #[test]
    fn backspace_on_empty_does_nothing() {
        let mut state = InputState::new();
        state.backspace();
        assert_eq!(state.output(), "");
        assert_eq!(state.pending(), "");
    }

    #[test]
    fn backspace_multi_char_output() {
        // output に複数文字ある場合は末尾1文字のみ削除
        let mut state = InputState::new();
        for ch in "ka".chars() {
            state.feed_char(ch);
        }
        for ch in "ki".chars() {
            state.feed_char(ch);
        }
        assert_eq!(state.output(), "かき");
        state.backspace();
        assert_eq!(state.output(), "か");
    }

    #[test]
    fn is_empty_after_input() {
        let mut state = InputState::new();
        assert!(state.is_empty());
        state.feed_char('k');
        assert!(!state.is_empty());
    }
```

追加する公開 API:

```rust
impl InputState {
    pub fn backspace(&mut self);
    pub fn is_empty(&self) -> bool;
}
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 3-2. Green: backspace / is_empty を実装

```rust
pub fn backspace(&mut self) {
    if !self.pending.is_empty() {
        self.pending.pop();
    } else {
        self.output.pop();
    }
}

pub fn is_empty(&self) -> bool {
    self.output.is_empty() && self.pending.is_empty()
}
```

- **動作確認:** `cargo test` で全テストがパスすること（Green）

### 3-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 4: エンジンの型定義と状態遷移 (TDD)

### 4-1. Red: 状態遷移のテストを書く

`src/engine.rs` にエンジンの型定義と基本的な状態遷移テストを書く:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_engine() -> ConversionEngine {
        let dict = crate::dictionary::Dictionary::load_from_file(
            Path::new("tests/fixtures/test_dict.txt"),
        ).unwrap();
        ConversionEngine::new(Some(dict))
    }

    fn engine_without_dict() -> ConversionEngine {
        ConversionEngine::new(None)
    }

    // === 初期状態 ===

    #[test]
    fn initial_state_is_direct() {
        let engine = test_engine();
        assert_eq!(engine.state(), EngineState::Direct);
    }

    // === Direct → Composing ===

    #[test]
    fn insert_char_transitions_to_composing() {
        let mut engine = test_engine();
        engine.process(EngineCommand::InsertChar('k'));
        assert_eq!(engine.state(), EngineState::Composing);
    }

    // === Composing → Converting ===

    #[test]
    fn convert_transitions_to_converting() {
        let mut engine = test_engine();
        // "kanji" → "かんじ"
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Convert);
        assert_eq!(engine.state(), EngineState::Converting);
        // 候補が返される
        assert!(output.candidates.is_some());
    }

    // === Converting → Direct (Commit) ===

    #[test]
    fn commit_in_converting_transitions_to_direct() {
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        let output = engine.process(EngineCommand::Commit);
        assert_eq!(engine.state(), EngineState::Direct);
        assert!(!output.committed.is_empty());
    }

    // === Composing → Direct (Commit: ひらがな確定) ===

    #[test]
    fn commit_in_composing_confirms_hiragana() {
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Commit);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "かんじ");
    }

    // === Composing → Direct (Cancel) ===

    #[test]
    fn cancel_in_composing_discards_input() {
        let mut engine = test_engine();
        for ch in "ka".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Cancel);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "");
        assert_eq!(output.display, "");
    }

    // === Converting → Composing (Cancel) ===

    #[test]
    fn cancel_in_converting_returns_to_composing() {
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        let output = engine.process(EngineCommand::Cancel);
        assert_eq!(engine.state(), EngineState::Composing);
        assert_eq!(output.display, "かんじ");
    }

    // === Direct では無視されるコマンド ===

    #[test]
    fn convert_in_direct_is_noop() {
        let mut engine = test_engine();
        let output = engine.process(EngineCommand::Convert);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "");
    }

    #[test]
    fn commit_in_direct_is_noop() {
        let mut engine = test_engine();
        let output = engine.process(EngineCommand::Commit);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "");
    }
}
```

型定義:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Direct,
    Composing,
    Converting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineCommand {
    InsertChar(char),
    Convert,
    NextCandidate,
    PrevCandidate,
    Commit,
    Cancel,
    Backspace,
}

#[derive(Debug, Clone)]
pub struct EngineOutput {
    /// 確定してアプリケーションに渡すテキスト
    pub committed: String,
    /// 現在の表示用テキスト（未確定文字列 or 選択中の候補）
    pub display: String,
    /// 候補リスト（Converting 状態のとき Some）
    pub candidates: Option<Vec<String>>,
    /// 候補リスト内の選択インデックス
    pub candidate_index: Option<usize>,
}

pub struct ConversionEngine { ... }

impl ConversionEngine {
    pub fn new(dict: Option<Dictionary>) -> Self;
    pub fn process(&mut self, command: EngineCommand) -> EngineOutput;
    pub fn state(&self) -> EngineState;
}
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 4-2. Green: 状態遷移を実装

`ConversionEngine` の内部構造:

```rust
pub struct ConversionEngine {
    state: EngineState,
    input: InputState,
    dict: Option<Dictionary>,
    candidates: Option<CandidateList>,
}
```

`process()` の実装方針:
- `match (self.state, &command)` で状態とコマンドの組み合わせを処理
- 各遷移で適切な `EngineOutput` を返す
- 無効な組み合わせ（例: Direct で Convert）は空の EngineOutput を返す

- **動作確認:** `cargo test` で全テストがパスすること（Green）

### 4-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 5: コマンド処理 — 候補ナビゲーションと Backspace (TDD)

### 5-1. Red: 候補ナビゲーションとバックスペースのテストを書く

`src/engine.rs` のテストに追加:

```rust
    // === 候補ナビゲーション ===

    #[test]
    fn next_candidate_moves_selection() {
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        let output = engine.process(EngineCommand::NextCandidate);
        assert_eq!(engine.state(), EngineState::Converting);
        // "かんじ" → ["漢字", "感じ", "幹事"], next → "感じ"
        assert_eq!(output.candidate_index, Some(1));
    }

    #[test]
    fn prev_candidate_moves_selection() {
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        engine.process(EngineCommand::NextCandidate); // index=1
        let output = engine.process(EngineCommand::PrevCandidate); // index=0
        assert_eq!(output.candidate_index, Some(0));
    }

    #[test]
    fn convert_in_converting_acts_as_next() {
        // Converting 状態で Convert (Space) → NextCandidate と同じ
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert); // index=0
        let output = engine.process(EngineCommand::Convert); // index=1
        assert_eq!(output.candidate_index, Some(1));
    }

    #[test]
    fn commit_in_converting_confirms_candidate() {
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        engine.process(EngineCommand::NextCandidate);
        let output = engine.process(EngineCommand::Commit);
        assert_eq!(output.committed, "感じ");
        assert_eq!(engine.state(), EngineState::Direct);
    }

    // === Backspace ===

    #[test]
    fn backspace_in_composing_removes_char() {
        let mut engine = test_engine();
        for ch in "ka".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        // "か" が output に入っている
        let output = engine.process(EngineCommand::Backspace);
        assert_eq!(output.display, "");
        assert_eq!(engine.state(), EngineState::Direct);
    }

    #[test]
    fn backspace_in_composing_with_pending() {
        let mut engine = test_engine();
        engine.process(EngineCommand::InsertChar('k'));
        // pending = "k"
        let output = engine.process(EngineCommand::Backspace);
        assert_eq!(output.display, "");
        assert_eq!(engine.state(), EngineState::Direct);
    }

    #[test]
    fn backspace_in_composing_partial_removal() {
        // "kak" → output="か", pending="k" → backspace → output="か", pending=""
        let mut engine = test_engine();
        for ch in "kak".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Backspace);
        assert_eq!(output.display, "か");
        assert_eq!(engine.state(), EngineState::Composing);
    }

    // === 辞書なしでの変換 ===

    #[test]
    fn convert_without_dict_confirms_hiragana() {
        // 辞書なしで Convert → 候補なし → ひらがなをそのまま確定
        let mut engine = engine_without_dict();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Convert);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "かんじ");
    }

    #[test]
    fn convert_no_candidates_confirms_hiragana() {
        // 辞書はあるが候補がない → ひらがなをそのまま確定
        let mut engine = test_engine();
        for ch in "aaaaa".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Convert);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "あああああ");
    }
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）

### 5-2. Green: 候補ナビゲーションと Backspace を実装

`process()` の追加分岐:
- `(Converting, NextCandidate)` / `(Converting, Convert)`: `candidates.next()` → 候補情報を返す
- `(Converting, PrevCandidate)`: `candidates.prev()` → 候補情報を返す
- `(Composing, Backspace)`: `input.backspace()` → 空になったら Direct に遷移
- 辞書なし / 候補なし → Convert 時にひらがなを直接確定

- **動作確認:** `cargo test` で全テストがパスすること（Green）

### 5-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 6: 統合テスト (TDD)

### 6-1. Red: エンドツーエンドの統合テストを書く

`src/engine.rs` のテストに追加:

```rust
    // === 統合テスト: 一連のフロー ===

    #[test]
    fn full_flow_kanji_convert_commit() {
        // "kanji" → Space → 1番目の候補「漢字」を確定
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Convert);
        assert_eq!(output.display, "漢字");
        assert!(output.candidates.is_some());

        let output = engine.process(EngineCommand::Commit);
        assert_eq!(output.committed, "漢字");
        assert_eq!(engine.state(), EngineState::Direct);
    }

    #[test]
    fn full_flow_select_second_candidate() {
        // "kanji" → Space → Next → 2番目の候補「感じ」を確定
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        engine.process(EngineCommand::NextCandidate);
        let output = engine.process(EngineCommand::Commit);
        assert_eq!(output.committed, "感じ");
    }

    #[test]
    fn full_flow_cancel_and_re_edit() {
        // "kanji" → Space → Cancel → "ha" 追加 → Commit
        // → "かんじは" をひらがなで確定（Cancel で Composing に戻り、追加入力後確定）
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        engine.process(EngineCommand::Cancel); // → Composing, display="かんじ"

        // "は" を追加
        for ch in "ha".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Commit);
        assert_eq!(output.committed, "かんじは");
    }

    #[test]
    fn full_flow_consecutive_conversions() {
        // 1回目: "kanji" → Convert → Commit → "漢字"
        // 2回目: "nihon" → Convert → Commit → "日本"
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        let output1 = engine.process(EngineCommand::Commit);
        assert_eq!(output1.committed, "漢字");

        for ch in "nihon".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        let output2 = engine.process(EngineCommand::Commit);
        assert_eq!(output2.committed, "日本");
    }

    #[test]
    fn display_updates_during_composing() {
        // 逐次入力中に display が更新される
        let mut engine = test_engine();
        let output = engine.process(EngineCommand::InsertChar('k'));
        assert_eq!(output.display, "k"); // pending

        let output = engine.process(EngineCommand::InsertChar('a'));
        assert_eq!(output.display, "か"); // output

        let output = engine.process(EngineCommand::InsertChar('n'));
        assert_eq!(output.display, "かn"); // output + pending
    }
```

- **動作確認:** `cargo test` で新しいテストが**失敗する**こと（Red）
- 注: ステップ4〜5が正しく実装されていれば、大半のテストは既にパスするはず。display の組み立てのテストのみ追加で対応が必要な場合がある。

### 6-2. Green: 必要な修正

- `EngineOutput.display` が Composing 時に `output + pending` を正しく返すことを確認
- 統合テストがすべてパスするまで修正

- **動作確認:** `cargo test` で全テストがパスすること（Green）

### 6-3. Refactor

- **動作確認:** `cargo test` 全パス、`cargo clippy` 警告なし、`cargo fmt -- --check` 差分なし

---

## ステップ 7: CLI デモの更新

### 7-1. main.rs を ConversionEngine ベースに更新

`src/main.rs` を以下の方針で更新:

- `ConversionEngine` を使用して入力処理する
- 入力行の各文字を `InsertChar` で処理
- Enter で自動的に `Convert` → 候補があれば表示 → 1番目で `Commit`
  （対話的な候補選択は Phase 5 の UI で実装。Phase 3 では行単位の簡易デモ）
- `--dict` オプションは引き続きサポート

出力例:
```
> kanji
  ひらがな: かんじ
  カタカナ: カンジ
  変換候補: 漢字 / 感じ / 幹事
  確定: 漢字
```

- **動作確認:** `cargo build` がエラーなく完了すること。`cargo run -- --dict tests/fixtures/test_dict.txt` で手動確認

---

## ステップ 8: 最終確認・コミット

### 8-1. 全テスト実行

```sh
cargo test
```

- 既存の 86 テスト + 新規テスト（約 30 テスト）が全パスすること

### 8-2. コード品質チェック

```sh
cargo clippy
cargo fmt -- --check
```

- 警告なし、フォーマット差分なしであること

### 8-3. `lib.rs` の確認

- `pub mod candidate;` と `pub mod engine;` が追加されていること

### 8-4. CLAUDE.md の更新

- ファイル構成に `src/candidate.rs`、`src/engine.rs` を追記
- EngineCommand / EngineState の概要を追記

### 8-5. コミット

- テスト全パスを確認した上でコミット

---

## 追加テスト一覧（予定）

| # | テスト名 | モジュール | 分類 | 内容 |
|---|---------|-----------|------|------|
| 1 | `new_with_candidates` | candidate | 基本操作 | 候補リスト作成、初期インデックス |
| 2 | `new_empty` | candidate | 基本操作 | 空リスト |
| 3 | `next_moves_forward` | candidate | next/prev | 次候補移動 |
| 4 | `prev_moves_backward` | candidate | next/prev | 前候補移動 |
| 5 | `next_at_end_wraps` | candidate | 境界値 | 末尾でラップ |
| 6 | `prev_at_start_wraps` | candidate | 境界値 | 先頭でラップ |
| 7 | `next_on_empty_no_panic` | candidate | 境界値 | 空リストで next |
| 8 | `prev_on_empty_no_panic` | candidate | 境界値 | 空リストで prev |
| 9 | `select_returns_current` | candidate | select | 現在の候補を返す |
| 10 | `select_on_empty` | candidate | select | 空リストで select |
| 11 | `candidates_returns_all` | candidate | 参照 | 全候補取得 |
| 12 | `backspace_removes_pending` | input_state | backspace | pending 削除 |
| 13 | `backspace_removes_output_char` | input_state | backspace | output 末尾削除 |
| 14 | `backspace_on_empty_does_nothing` | input_state | backspace | 空でも安全 |
| 15 | `backspace_multi_char_output` | input_state | backspace | 複数文字から1つ削除 |
| 16 | `is_empty_after_input` | input_state | is_empty | 空判定 |
| 17 | `initial_state_is_direct` | engine | 初期状態 | Direct で開始 |
| 18 | `insert_char_transitions_to_composing` | engine | 状態遷移 | Direct → Composing |
| 19 | `convert_transitions_to_converting` | engine | 状態遷移 | Composing → Converting |
| 20 | `commit_in_converting_transitions_to_direct` | engine | 状態遷移 | Converting → Direct |
| 21 | `commit_in_composing_confirms_hiragana` | engine | 状態遷移 | ひらがな確定 |
| 22 | `cancel_in_composing_discards_input` | engine | 状態遷移 | 入力破棄 |
| 23 | `cancel_in_converting_returns_to_composing` | engine | 状態遷移 | 候補選択キャンセル |
| 24 | `convert_in_direct_is_noop` | engine | 無視 | Direct で Convert |
| 25 | `commit_in_direct_is_noop` | engine | 無視 | Direct で Commit |
| 26 | `next_candidate_moves_selection` | engine | 候補操作 | 次候補 |
| 27 | `prev_candidate_moves_selection` | engine | 候補操作 | 前候補 |
| 28 | `convert_in_converting_acts_as_next` | engine | 候補操作 | Space で次候補 |
| 29 | `commit_in_converting_confirms_candidate` | engine | 候補操作 | 候補確定 |
| 30 | `backspace_in_composing_removes_char` | engine | Backspace | 確定済み文字削除 |
| 31 | `backspace_in_composing_with_pending` | engine | Backspace | pending 削除 |
| 32 | `backspace_in_composing_partial_removal` | engine | Backspace | 部分削除 |
| 33 | `convert_without_dict_confirms_hiragana` | engine | 辞書なし | ひらがな直接確定 |
| 34 | `convert_no_candidates_confirms_hiragana` | engine | 辞書なし | 候補なし→確定 |
| 35 | `full_flow_kanji_convert_commit` | engine | 統合 | 基本フロー |
| 36 | `full_flow_select_second_candidate` | engine | 統合 | 2番目候補選択 |
| 37 | `full_flow_cancel_and_re_edit` | engine | 統合 | Cancel後に再編集 |
| 38 | `full_flow_consecutive_conversions` | engine | 統合 | 連続変換 |
| 39 | `display_updates_during_composing` | engine | 統合 | 逐次表示更新 |

合計: 約 39 テスト追加（既存 86 + 新規 39 = 約 125 テスト）

---

## 依存クレート

変更なし（既存の `encoding_rs = "0.8"` のみ）
