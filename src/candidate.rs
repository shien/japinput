//! 変換候補リストの管理。
//!
//! 候補の一覧と現在の選択インデックスを保持し、
//! 前後ナビゲーション・選択を提供する。

pub struct CandidateList {
    candidates: Vec<String>,
    index: usize,
}

impl CandidateList {
    /// 候補リストを作成する。初期選択は先頭。
    pub fn new(candidates: Vec<String>) -> Self {
        Self {
            candidates,
            index: 0,
        }
    }

    /// 現在選択中の候補を返す。
    pub fn current(&self) -> Option<&str> {
        self.candidates.get(self.index).map(|s| s.as_str())
    }

    /// 現在の選択インデックスを返す。
    pub fn index(&self) -> usize {
        self.index
    }

    /// 次の候補に移動する。末尾の場合は先頭にラップする。
    pub fn next(&mut self) {
        if !self.candidates.is_empty() {
            self.index = (self.index + 1) % self.candidates.len();
        }
    }

    /// 前の候補に移動する。先頭の場合は末尾にラップする。
    pub fn prev(&mut self) {
        if !self.candidates.is_empty() {
            if self.index == 0 {
                self.index = self.candidates.len() - 1;
            } else {
                self.index -= 1;
            }
        }
    }

    /// 現在の候補を確定して返す。
    pub fn select(&self) -> Option<String> {
        self.current().map(|s| s.to_string())
    }

    /// 候補が空かどうか。
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// 候補数を返す。
    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    /// 全候補のスライスを返す。
    pub fn candidates(&self) -> &[String] {
        &self.candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === 基本操作 ===

    #[test]
    fn new_with_candidates() {
        let cl = CandidateList::new(vec![
            "漢字".to_string(),
            "感じ".to_string(),
            "幹事".to_string(),
        ]);
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
        let mut cl = CandidateList::new(vec![
            "漢字".to_string(),
            "感じ".to_string(),
            "幹事".to_string(),
        ]);
        cl.next();
        assert_eq!(cl.current(), Some("感じ"));
        assert_eq!(cl.index(), 1);
    }

    #[test]
    fn prev_moves_backward() {
        let mut cl = CandidateList::new(vec![
            "漢字".to_string(),
            "感じ".to_string(),
            "幹事".to_string(),
        ]);
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
        cl.next();
        assert_eq!(cl.current(), None);
    }

    #[test]
    fn prev_on_empty_no_panic() {
        let mut cl = CandidateList::new(vec![]);
        cl.prev();
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
