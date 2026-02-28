use japinput::dictionary::Dictionary;
use japinput::engine::{ConversionEngine, EngineCommand};
use japinput::katakana;
use std::io::{self, BufRead, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // --dict オプションで辞書ファイルを指定
    let dict = if let Some(pos) = args.iter().position(|a| a == "--dict") {
        let Some(path) = args.get(pos + 1) else {
            eprintln!("エラー: --dict の後に辞書ファイルパスを指定してください");
            std::process::exit(1);
        };
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

    let has_dict = dict.is_some();
    let mut engine = ConversionEngine::new(dict);

    println!("japinput - ローマ字→かな変換デモ");
    if has_dict {
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

        // 各文字を InsertChar で処理
        for ch in line.chars() {
            engine.process(EngineCommand::InsertChar(ch));
        }

        // Convert 前にひらがなを取得（Commit でひらがな確定→取得）
        let composing_output = engine.process(EngineCommand::Commit);
        let hiragana = composing_output.committed;
        let katakana_display = katakana::to_katakana(&hiragana);
        let _ = writeln!(stdout, "  ひらがな: {hiragana}");
        let _ = writeln!(stdout, "  カタカナ: {katakana_display}");

        // 辞書がある場合は再度入力して変換を試行
        if has_dict {
            for ch in line.chars() {
                engine.process(EngineCommand::InsertChar(ch));
            }
            let output = engine.process(EngineCommand::Convert);

            if let Some(ref candidates) = output.candidates {
                let _ = writeln!(stdout, "  変換候補: {}", candidates.join(" / "));
                let commit_output = engine.process(EngineCommand::Commit);
                let _ = writeln!(stdout, "  確定: {}", commit_output.committed);
            } else {
                let _ = writeln!(stdout, "  変換候補: (なし)");
            }
        }

        let _ = writeln!(stdout);
    }
}
