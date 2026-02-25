//! 入力状態管理。
//!
//! ローマ字を1文字ずつ受け取り、逐次的にひらがなへ変換する。
//! バッファと確定済み出力を保持する。

use crate::romaji;

/// 入力状態を管理する構造体。
#[derive(Debug, Clone)]
pub struct InputState {
    /// 確定したひらがな出力
    output: String,
    /// まだ確定していないローマ字バッファ
    pending: String,
}

impl InputState {
    /// 新しい InputState を作成する。
    pub fn new() -> Self {
        Self {
            output: String::new(),
            pending: String::new(),
        }
    }

    /// 1文字入力する。確定したひらがながあれば output に追加される。
    pub fn feed_char(&mut self, ch: char) {
        self.pending.push(ch);
        let result = romaji::convert(&self.pending);
        self.output.push_str(&result.output);
        self.pending = result.pending;
    }

    /// 未確定バッファを確定する（末尾の "n" → "ん"）。
    pub fn flush(&mut self) {
        if self.pending == "n" {
            self.output.push('ん');
            self.pending.clear();
        } else if !self.pending.is_empty() {
            self.output.push_str(&self.pending);
            self.pending.clear();
        }
    }

    /// バッファと出力をクリアする。
    pub fn reset(&mut self) {
        self.output.clear();
        self.pending.clear();
    }

    /// 確定済みの出力を返す。
    pub fn output(&self) -> &str {
        &self.output
    }

    /// 未確定のバッファを返す。
    pub fn pending(&self) -> &str {
        &self.pending
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === 基本的な逐次入力 ===

    #[test]
    fn feed_single_vowel() {
        let mut state = InputState::new();
        state.feed_char('a');
        assert_eq!(state.output(), "あ");
        assert_eq!(state.pending(), "");
    }

    #[test]
    fn feed_consonant_then_vowel() {
        let mut state = InputState::new();
        state.feed_char('k');
        assert_eq!(state.output(), "");
        assert_eq!(state.pending(), "k");
        state.feed_char('a');
        assert_eq!(state.output(), "か");
        assert_eq!(state.pending(), "");
    }

    #[test]
    fn feed_sequence_aiueo() {
        let mut state = InputState::new();
        for ch in "aiueo".chars() {
            state.feed_char(ch);
        }
        assert_eq!(state.output(), "あいうえお");
        assert_eq!(state.pending(), "");
    }

    // === 促音 ===

    #[test]
    fn feed_sokuon() {
        let mut state = InputState::new();
        for ch in "kakko".chars() {
            state.feed_char(ch);
        }
        assert_eq!(state.output(), "かっこ");
    }

    // === 「ん」処理 ===

    #[test]
    fn feed_nn() {
        let mut state = InputState::new();
        state.feed_char('n');
        state.feed_char('n');
        assert_eq!(state.output(), "ん");
        assert_eq!(state.pending(), "n");
    }

    #[test]
    fn feed_n_before_consonant() {
        let mut state = InputState::new();
        for ch in "kanta".chars() {
            state.feed_char(ch);
        }
        assert_eq!(state.output(), "かんた");
    }

    // === flush ===

    #[test]
    fn flush_trailing_n() {
        let mut state = InputState::new();
        for ch in "kan".chars() {
            state.feed_char(ch);
        }
        assert_eq!(state.output(), "か");
        assert_eq!(state.pending(), "n");
        state.flush();
        assert_eq!(state.output(), "かん");
        assert_eq!(state.pending(), "");
    }

    #[test]
    fn flush_empty_pending() {
        let mut state = InputState::new();
        for ch in "ka".chars() {
            state.feed_char(ch);
        }
        state.flush();
        assert_eq!(state.output(), "か");
        assert_eq!(state.pending(), "");
    }

    // === reset ===

    #[test]
    fn reset_clears_all() {
        let mut state = InputState::new();
        for ch in "ka".chars() {
            state.feed_char(ch);
        }
        state.reset();
        assert_eq!(state.output(), "");
        assert_eq!(state.pending(), "");
    }

    // === convert() との一致確認 ===

    #[test]
    fn matches_batch_convert() {
        let input = "konnichiwa";
        let batch = romaji::convert(input);

        let mut state = InputState::new();
        for ch in input.chars() {
            state.feed_char(ch);
        }
        state.flush();

        // flush 後の output は convert の output + pending を確定した結果と一致する
        assert_eq!(state.output(), batch.output);
    }

    #[test]
    fn matches_batch_convert_toukyou() {
        let input = "toukyou";
        let batch = romaji::convert(input);

        let mut state = InputState::new();
        for ch in input.chars() {
            state.feed_char(ch);
        }
        state.flush();

        assert_eq!(state.output(), batch.output);
    }
}
