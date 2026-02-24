use japinput::input_state::InputState;
use japinput::katakana;
use std::io::{self, BufRead, Write};

fn main() {
    println!("japinput - ローマ字→かな変換デモ");
    println!("ローマ字を入力して Enter で変換します。");
    println!("空行または Ctrl+C で終了します。");
    println!();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            break;
        }

        let mut state = InputState::new();
        for ch in line.chars() {
            state.feed_char(ch);
        }
        state.flush();

        let hiragana = state.output();
        let katakana = katakana::to_katakana(hiragana);

        let _ = writeln!(stdout, "  ひらがな: {hiragana}");
        let _ = writeln!(stdout, "  カタカナ: {katakana}");
        let _ = writeln!(stdout);
    }
}
