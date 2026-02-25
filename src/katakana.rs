/// ひらがな→カタカナ変換。
///
/// ひらがな (U+3041〜U+3096) をカタカナ (U+30A1〜U+30F6) に変換する。
/// ひらがな以外の文字（長音記号、ASCII、句読点など）はそのまま保持する。
pub fn to_katakana(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            // ひらがな範囲: U+3041 (ぁ) 〜 U+3096 (ゖ)
            let cp = ch as u32;
            if (0x3041..=0x3096).contains(&cp) {
                // カタカナはひらがな + 0x60 のオフセット
                char::from_u32(cp + 0x60).unwrap_or(ch)
            } else {
                ch
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // === 基本変換 ===

    #[test]
    fn basic_hiragana() {
        assert_eq!(to_katakana("あいうえお"), "アイウエオ");
    }

    #[test]
    fn ka_row_katakana() {
        assert_eq!(to_katakana("かきくけこ"), "カキクケコ");
    }

    #[test]
    fn dakuon_katakana() {
        assert_eq!(to_katakana("がぎぐげご"), "ガギグゲゴ");
    }

    #[test]
    fn youon_katakana() {
        assert_eq!(to_katakana("きゃきゅきょ"), "キャキュキョ");
    }

    // === 特殊文字の保持 ===

    #[test]
    fn chouon_preserved() {
        assert_eq!(to_katakana("らーめん"), "ラーメン");
    }

    #[test]
    fn ascii_preserved() {
        assert_eq!(to_katakana("abc"), "abc");
    }

    #[test]
    fn mixed_input() {
        assert_eq!(to_katakana("こーど123"), "コード123");
    }

    // === エッジケース ===

    #[test]
    fn empty_input() {
        assert_eq!(to_katakana(""), "");
    }

    #[test]
    fn sokuon_katakana() {
        assert_eq!(to_katakana("がっこう"), "ガッコウ");
    }

    #[test]
    fn vu_katakana() {
        assert_eq!(to_katakana("ゔ"), "ヴ");
    }
}
