use japinput::dictionary::Dictionary;
use japinput::input_state::InputState;
use japinput::katakana;
use std::io::{self, BufRead, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // --dict オプションで辞書ファイルを指定
    let dict = if let Some(pos) = args.iter().position(|a| a == "--dict") {
        let path = args
            .get(pos + 1)
            .expect("--dict の後に辞書ファイルパスを指定してください");
        match Dictionary::load_from_file(Path::new(path)) {
            Ok(d) => {
                eprintln!("辞書を読み込みました: {path}");
                Some(d)
            }
            Err(e) => {
                eprintln!("辞書の読み込みに失敗: {e}");
                None
            }
        }
    } else {
        None
    };

    println!("japinput - ローマ字→かな変換デモ");
    if dict.is_some() {
        println!("辞書検索モード: ローマ字を入力すると漢字候補も表示します。");
    }
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

        if let Some(ref dict) = dict {
            if let Some(candidates) = dict.lookup(hiragana) {
                let _ = writeln!(stdout, "  変換候補: {}", candidates.join(" / "));
            } else {
                let _ = writeln!(stdout, "  変換候補: (なし)");
            }
        }

        let _ = writeln!(stdout);
    }
}
