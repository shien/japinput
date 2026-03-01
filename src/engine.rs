//! 変換エンジン。
//!
//! ローマ字入力 → ひらがな変換 → 辞書検索 → 候補選択 → 確定
//! の一連の変換パイプラインを管理する。

use crate::candidate::CandidateList;
use crate::dictionary::Dictionary;
use crate::input_state::InputState;

/// エンジンの状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// 直接入力（変換なし）
    Direct,
    /// ローマ字→かな変換中（未確定文字列あり）
    Composing,
    /// 候補選択中
    Converting,
}

/// エンジンへのコマンド。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineCommand {
    /// 文字入力
    InsertChar(char),
    /// 変換開始 (Space)
    Convert,
    /// 次の候補
    NextCandidate,
    /// 前の候補
    PrevCandidate,
    /// 確定 (Enter)
    Commit,
    /// キャンセル (Escape)
    Cancel,
    /// 1文字削除
    Backspace,
}

/// エンジンの処理結果。
#[derive(Debug, Clone)]
pub struct EngineOutput {
    /// 確定してアプリケーションに渡すテキスト
    pub committed: String,
    /// 現在の表示用テキスト（未確定文字列 or 選択中の候補）
    pub display: String,
    /// 候補リスト内の選択インデックス
    pub candidate_index: Option<usize>,
}

/// 変換エンジン。
pub struct ConversionEngine {
    state: EngineState,
    input: InputState,
    dict: Option<Dictionary>,
    candidates: Option<CandidateList>,
    /// 変換時の読み（ひらがな）を保持する。
    reading: String,
}

impl ConversionEngine {
    /// 新しい変換エンジンを作成する。
    pub fn new(dict: Option<Dictionary>) -> Self {
        Self {
            state: EngineState::Direct,
            input: InputState::new(),
            dict,
            candidates: None,
            reading: String::new(),
        }
    }

    /// 現在の状態を返す。
    pub fn state(&self) -> EngineState {
        self.state
    }

    /// 変換候補リストを返す（Converting 状態のとき Some）。
    pub fn candidates(&self) -> Option<&[String]> {
        self.candidates.as_ref().map(|cl| cl.candidates())
    }

    /// 変換時の読み（ひらがな）を返す。
    pub fn reading(&self) -> &str {
        &self.reading
    }

    /// コマンドを処理し、結果を返す。
    pub fn process(&mut self, command: EngineCommand) -> EngineOutput {
        match (&self.state, &command) {
            // === Direct ===
            (EngineState::Direct, EngineCommand::InsertChar(ch)) => {
                self.input.feed_char(*ch);
                self.state = EngineState::Composing;
                self.composing_output()
            }
            (EngineState::Direct, _) => self.empty_output(),

            // === Composing ===
            (EngineState::Composing, EngineCommand::InsertChar(ch)) => {
                self.input.feed_char(*ch);
                self.composing_output()
            }
            (EngineState::Composing, EngineCommand::Convert) => self.do_convert(),
            (EngineState::Composing, EngineCommand::Commit) => {
                self.input.flush();
                let committed = self.input.output().to_string();
                self.input.reset();
                self.state = EngineState::Direct;
                EngineOutput {
                    committed,
                    display: String::new(),
                    candidate_index: None,
                }
            }
            (EngineState::Composing, EngineCommand::Cancel) => {
                self.input.reset();
                self.state = EngineState::Direct;
                self.empty_output()
            }
            (EngineState::Composing, EngineCommand::Backspace) => {
                self.input.backspace();
                if self.input.is_empty() {
                    self.state = EngineState::Direct;
                }
                self.composing_output()
            }
            (EngineState::Composing, _) => self.composing_output(),

            // === Converting ===
            (EngineState::Converting, EngineCommand::NextCandidate)
            | (EngineState::Converting, EngineCommand::Convert) => {
                if let Some(ref mut cl) = self.candidates {
                    cl.next();
                }
                self.converting_output()
            }
            (EngineState::Converting, EngineCommand::PrevCandidate) => {
                if let Some(ref mut cl) = self.candidates {
                    cl.prev();
                }
                self.converting_output()
            }
            (EngineState::Converting, EngineCommand::Commit) => {
                let committed = self
                    .candidates
                    .as_ref()
                    .and_then(|cl| cl.select())
                    .unwrap_or_default();
                self.candidates = None;
                self.input.reset();
                self.state = EngineState::Direct;
                EngineOutput {
                    committed,
                    display: String::new(),
                    candidate_index: None,
                }
            }
            (EngineState::Converting, EngineCommand::Cancel) => {
                self.candidates = None;
                self.state = EngineState::Composing;
                self.composing_output()
            }
            (EngineState::Converting, EngineCommand::InsertChar(ch)) => {
                // 現在の候補を確定し、新しい文字で Composing を開始する
                let committed = self
                    .candidates
                    .as_ref()
                    .and_then(|cl| cl.select())
                    .unwrap_or_default();
                self.candidates = None;
                self.input.reset();
                self.input.feed_char(*ch);
                self.state = EngineState::Composing;
                let composing = self.composing_output();
                EngineOutput {
                    committed,
                    display: composing.display,
                    candidate_index: None,
                }
            }
            (EngineState::Converting, EngineCommand::Backspace) => {
                // 変換をキャンセルして Composing に戻る（Cancel と同じ動作）
                self.candidates = None;
                self.state = EngineState::Composing;
                self.composing_output()
            }
        }
    }

    /// 変換を実行する。候補があれば Converting へ、なければひらがな確定。
    fn do_convert(&mut self) -> EngineOutput {
        self.input.flush();
        let hiragana = self.input.output().to_string();
        self.reading = hiragana.clone();

        let candidates = self
            .dict
            .as_ref()
            .and_then(|d| d.lookup(&hiragana))
            .map(|c| c.iter().map(|s| s.to_string()).collect::<Vec<_>>());

        match candidates {
            Some(cands) if !cands.is_empty() => {
                let cl = CandidateList::new(cands);
                let display = cl.current().unwrap_or("").to_string();
                let idx = cl.index();
                self.candidates = Some(cl);
                self.state = EngineState::Converting;
                EngineOutput {
                    committed: String::new(),
                    display,
                    candidate_index: Some(idx),
                }
            }
            _ => {
                // 候補なし → ひらがなを確定
                let committed = hiragana;
                self.input.reset();
                self.state = EngineState::Direct;
                EngineOutput {
                    committed,
                    display: String::new(),
                    candidate_index: None,
                }
            }
        }
    }

    /// Composing 状態の EngineOutput を組み立てる。
    fn composing_output(&self) -> EngineOutput {
        let display = format!("{}{}", self.input.output(), self.input.pending());
        EngineOutput {
            committed: String::new(),
            display,
            candidate_index: None,
        }
    }

    /// Converting 状態の EngineOutput を組み立てる。
    fn converting_output(&self) -> EngineOutput {
        match &self.candidates {
            Some(cl) => EngineOutput {
                committed: String::new(),
                display: cl.current().unwrap_or("").to_string(),
                candidate_index: Some(cl.index()),
            },
            None => self.empty_output(),
        }
    }

    /// 空の EngineOutput を返す。
    fn empty_output(&self) -> EngineOutput {
        EngineOutput {
            committed: String::new(),
            display: String::new(),
            candidate_index: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_engine() -> ConversionEngine {
        let dict = Dictionary::load_from_file(Path::new("tests/fixtures/test_dict.txt")).unwrap();
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
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        assert_eq!(engine.state(), EngineState::Converting);
        assert!(engine.candidates().is_some());
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
        // "かんじ" → ["漢字", "感じ", "幹事"], next → index=1 "感じ"
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
        // output="か", pending=""
        let output = engine.process(EngineCommand::Backspace);
        assert_eq!(output.display, "");
        assert_eq!(engine.state(), EngineState::Direct);
    }

    #[test]
    fn backspace_in_composing_with_pending() {
        let mut engine = test_engine();
        engine.process(EngineCommand::InsertChar('k'));
        // pending="k"
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

    // === Converting で InsertChar → 自動確定+新規入力 ===

    #[test]
    fn insert_char_in_converting_auto_commits() {
        // "kanji" → Convert → 'a' → 候補「漢字」が確定され、'a' の入力が開始
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert); // Converting, display="漢字"
        let output = engine.process(EngineCommand::InsertChar('a'));
        assert_eq!(output.committed, "漢字");
        assert_eq!(engine.state(), EngineState::Composing);
        assert_eq!(output.display, "あ");
    }

    #[test]
    fn insert_char_in_converting_after_next() {
        // 2番目の候補を選択中に文字入力 → 2番目の候補が確定
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        engine.process(EngineCommand::NextCandidate); // "感じ"
        let output = engine.process(EngineCommand::InsertChar('k'));
        assert_eq!(output.committed, "感じ");
        assert_eq!(engine.state(), EngineState::Composing);
        assert_eq!(output.display, "k");
    }

    // === Converting で Backspace → Composing に戻る ===

    #[test]
    fn backspace_in_converting_returns_to_composing() {
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        let output = engine.process(EngineCommand::Backspace);
        assert_eq!(engine.state(), EngineState::Composing);
        assert_eq!(output.display, "かんじ");
    }

    // === reading() getter ===

    #[test]
    fn reading_available_after_convert() {
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        assert_eq!(engine.reading(), "かんじ");
    }

    // === 辞書なし / 候補なしでの変換 ===

    #[test]
    fn convert_without_dict_confirms_hiragana() {
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
        let mut engine = test_engine();
        for ch in "aaaaa".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        let output = engine.process(EngineCommand::Convert);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "あああああ");
    }

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
        assert!(engine.candidates().is_some());

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
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        engine.process(EngineCommand::Cancel); // → Composing, display="かんじ"

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

    // === Emacs キーバインド統合テスト ===
    //
    // key_mapping のユニットテストに加え、エンジン経由で
    // Emacs キーバインドのコマンドが正しく動作することを確認する。
    // ここでは EngineCommand を直接渡すため key_mapping は経由しないが、
    // Ctrl+キーで発行されるコマンドがエンジンの各状態で正しく処理されることを検証する。

    #[test]
    fn emacs_ctrl_j_composing_commits_hiragana() {
        // Composing 状態で Commit (Ctrl+J 相当) → ひらがな確定 → Direct
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        assert_eq!(engine.state(), EngineState::Composing);
        let output = engine.process(EngineCommand::Commit);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "かんじ");
    }

    #[test]
    fn emacs_ctrl_j_converting_commits_candidate() {
        // Converting 状態で Commit (Ctrl+J 相当) → 候補確定 → Direct
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        assert_eq!(engine.state(), EngineState::Converting);
        let output = engine.process(EngineCommand::Commit);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "漢字");
    }

    #[test]
    fn emacs_ctrl_g_composing_cancels() {
        // Composing 状態で Cancel (Ctrl+G 相当) → 入力破棄 → Direct
        let mut engine = test_engine();
        for ch in "ka".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        assert_eq!(engine.state(), EngineState::Composing);
        let output = engine.process(EngineCommand::Cancel);
        assert_eq!(engine.state(), EngineState::Direct);
        assert_eq!(output.committed, "");
        assert_eq!(output.display, "");
    }

    #[test]
    fn emacs_ctrl_g_converting_returns_to_composing() {
        // Converting 状態で Cancel (Ctrl+G 相当) → Composing に戻る
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        assert_eq!(engine.state(), EngineState::Converting);
        let output = engine.process(EngineCommand::Cancel);
        assert_eq!(engine.state(), EngineState::Composing);
        assert_eq!(output.display, "かんじ");
    }

    #[test]
    fn emacs_ctrl_n_p_converting_navigates() {
        // Converting 状態で NextCandidate/PrevCandidate (Ctrl+N/P 相当) → 候補移動
        let mut engine = test_engine();
        for ch in "kanji".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        engine.process(EngineCommand::Convert);
        // Ctrl+N → 次の候補
        let output = engine.process(EngineCommand::NextCandidate);
        assert_eq!(output.candidate_index, Some(1));
        assert_eq!(output.display, "感じ");
        // Ctrl+P → 前の候補に戻る
        let output = engine.process(EngineCommand::PrevCandidate);
        assert_eq!(output.candidate_index, Some(0));
        assert_eq!(output.display, "漢字");
    }

    #[test]
    fn emacs_ctrl_h_composing_backspace() {
        // Composing 状態で Backspace (Ctrl+H 相当) → 1文字削除
        let mut engine = test_engine();
        for ch in "kak".chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }
        // output="か", pending="k"
        let output = engine.process(EngineCommand::Backspace);
        assert_eq!(output.display, "か");
        assert_eq!(engine.state(), EngineState::Composing);
    }
}
