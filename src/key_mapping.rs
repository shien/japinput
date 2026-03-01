//! 仮想キーコードから EngineCommand への変換。
//!
//! Windows の仮想キーコード (VK_*) と修飾キー状態を受け取り、
//! ConversionEngine に渡す EngineCommand に変換する。
//! プラットフォーム非依存のため、どの OS でもテスト可能。

use crate::engine::EngineCommand;

// === 仮想キーコード定数 ===

pub const VK_BACK: u16 = 0x08;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_SHIFT: u16 = 0x10;
pub const VK_CONTROL: u16 = 0x11;
pub const VK_MENU: u16 = 0x12; // Alt
pub const VK_ESCAPE: u16 = 0x1B;
pub const VK_SPACE: u16 = 0x20;
pub const VK_UP: u16 = 0x26;
pub const VK_DOWN: u16 = 0x28;
pub const VK_0: u16 = 0x30;
pub const VK_9: u16 = 0x39;
pub const VK_A: u16 = 0x41;
pub const VK_Z: u16 = 0x5A;
pub const VK_F1: u16 = 0x70;
pub const VK_OEM_COMMA: u16 = 0xBC;
pub const VK_OEM_MINUS: u16 = 0xBD;
pub const VK_OEM_PERIOD: u16 = 0xBE;

/// 修飾キーの状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
        }
    }

    pub fn shift() -> Self {
        Self {
            shift: true,
            ctrl: false,
            alt: false,
        }
    }

    pub fn ctrl() -> Self {
        Self {
            shift: false,
            ctrl: true,
            alt: false,
        }
    }

    pub fn alt() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: true,
        }
    }
}

/// 仮想キーコードと修飾キー状態を EngineCommand に変換する。
///
/// - `vk`: Windows 仮想キーコード
/// - `modifiers`: 修飾キーの状態
/// - `ime_on`: IME がオンかどうか
///
/// 戻り値: 対応する EngineCommand。処理しないキーの場合は None。
pub fn map_key(vk: u16, modifiers: &Modifiers, ime_on: bool) -> Option<EngineCommand> {
    if !ime_on {
        return None;
    }

    if modifiers.ctrl || modifiers.alt {
        return None;
    }

    match vk {
        VK_A..=VK_Z => {
            let base = (b'a' + (vk - VK_A) as u8) as char;
            let ch = if modifiers.shift {
                base.to_ascii_uppercase()
            } else {
                base
            };
            Some(EngineCommand::InsertChar(ch))
        }
        VK_0..=VK_9 if !modifiers.shift => {
            let ch = (b'0' + (vk - VK_0) as u8) as char;
            Some(EngineCommand::InsertChar(ch))
        }
        VK_SPACE => Some(EngineCommand::Convert),
        VK_RETURN => Some(EngineCommand::Commit),
        VK_ESCAPE => Some(EngineCommand::Cancel),
        VK_BACK => Some(EngineCommand::Backspace),
        VK_DOWN => Some(EngineCommand::NextCandidate),
        VK_UP => Some(EngineCommand::PrevCandidate),
        VK_OEM_MINUS => Some(EngineCommand::InsertChar('-')),
        VK_OEM_PERIOD => Some(EngineCommand::InsertChar('.')),
        VK_OEM_COMMA => Some(EngineCommand::InsertChar(',')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineCommand;

    // === アルファベットキー ===

    #[test]
    fn alphabet_key_lowercase() {
        let cmd = map_key(VK_A, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::InsertChar('a')));
    }

    #[test]
    fn alphabet_key_all_letters() {
        for vk in VK_A..=VK_Z {
            let cmd = map_key(vk, &Modifiers::none(), true);
            let expected_char = (b'a' + (vk - VK_A) as u8) as char;
            assert_eq!(cmd, Some(EngineCommand::InsertChar(expected_char)));
        }
    }

    // === 特殊キー ===

    #[test]
    fn space_key_converts() {
        let cmd = map_key(VK_SPACE, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::Convert));
    }

    #[test]
    fn enter_key_commits() {
        let cmd = map_key(VK_RETURN, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::Commit));
    }

    #[test]
    fn escape_key_cancels() {
        let cmd = map_key(VK_ESCAPE, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::Cancel));
    }

    #[test]
    fn backspace_key() {
        let cmd = map_key(VK_BACK, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::Backspace));
    }

    #[test]
    fn down_arrow_next_candidate() {
        let cmd = map_key(VK_DOWN, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::NextCandidate));
    }

    #[test]
    fn up_arrow_prev_candidate() {
        let cmd = map_key(VK_UP, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::PrevCandidate));
    }

    // === IME オフ ===

    #[test]
    fn ime_off_returns_none() {
        let cmd = map_key(VK_A, &Modifiers::none(), false);
        assert_eq!(cmd, None);
    }

    #[test]
    fn ime_off_space_returns_none() {
        let cmd = map_key(VK_SPACE, &Modifiers::none(), false);
        assert_eq!(cmd, None);
    }

    // === 修飾キー ===

    #[test]
    fn ctrl_key_returns_none() {
        let cmd = map_key(VK_A, &Modifiers::ctrl(), true);
        assert_eq!(cmd, None);
    }

    #[test]
    fn alt_key_returns_none() {
        let cmd = map_key(VK_A, &Modifiers::alt(), true);
        assert_eq!(cmd, None);
    }

    #[test]
    fn shift_alphabet_uppercase() {
        let cmd = map_key(VK_A, &Modifiers::shift(), true);
        assert_eq!(cmd, Some(EngineCommand::InsertChar('A')));
    }

    // === 処理しないキー ===

    #[test]
    fn function_keys_return_none() {
        let cmd = map_key(VK_F1, &Modifiers::none(), true);
        assert_eq!(cmd, None);
    }

    // === 数字キー ===

    #[test]
    fn number_key_0() {
        let cmd = map_key(VK_0, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::InsertChar('0')));
    }

    #[test]
    fn number_key_9() {
        let cmd = map_key(VK_9, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::InsertChar('9')));
    }

    #[test]
    fn number_keys_all() {
        for vk in VK_0..=VK_9 {
            let cmd = map_key(vk, &Modifiers::none(), true);
            let expected_char = (b'0' + (vk - VK_0) as u8) as char;
            assert_eq!(cmd, Some(EngineCommand::InsertChar(expected_char)));
        }
    }

    #[test]
    fn number_key_with_shift_returns_none() {
        // Shift+数字はシステムに処理を委ねる（! @ # 等）
        let cmd = map_key(VK_0, &Modifiers::shift(), true);
        assert_eq!(cmd, None);
    }

    // === 句読点 ===

    #[test]
    fn minus_key_inserts_minus() {
        let cmd = map_key(VK_OEM_MINUS, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::InsertChar('-')));
    }

    #[test]
    fn period_key_inserts_period() {
        let cmd = map_key(VK_OEM_PERIOD, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::InsertChar('.')));
    }

    #[test]
    fn comma_key_inserts_comma() {
        let cmd = map_key(VK_OEM_COMMA, &Modifiers::none(), true);
        assert_eq!(cmd, Some(EngineCommand::InsertChar(',')));
    }
}
