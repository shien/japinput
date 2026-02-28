//! TSF TextService。IME のメインオブジェクト。
//!
//! `ITfTextInputProcessorEx` と `ITfKeyEventSink` を実装し、
//! Windows の TSF フレームワークと ConversionEngine を接続する。

use std::sync::Mutex;

use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::TextServices::*;
use windows::core::*;

use crate::dictionary::Dictionary;
use crate::engine::{ConversionEngine, EngineOutput};
use crate::key_mapping::{self, Modifiers};

#[implement(ITfTextInputProcessorEx, ITfTextInputProcessor, ITfKeyEventSink)]
pub struct TextService {
    thread_mgr: Mutex<Option<ITfThreadMgr>>,
    client_id: Mutex<u32>,
    engine: Mutex<ConversionEngine>,
    ime_on: Mutex<bool>,
    composition: Mutex<Option<ITfComposition>>,
}

impl TextService {
    pub fn new() -> Self {
        let dict = Self::load_default_dict();
        Self {
            thread_mgr: Mutex::new(None),
            client_id: Mutex::new(0),
            engine: Mutex::new(ConversionEngine::new(dict)),
            ime_on: Mutex::new(false),
            composition: Mutex::new(None),
        }
    }

    fn load_default_dict() -> Option<Dictionary> {
        let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
        let dict_path = exe_dir.join("dict").join("SKK-JISYO.L");
        Dictionary::load_from_file(&dict_path).ok()
    }

    /// EngineOutput に基づいて Composition を更新する。
    fn update_composition(&self, context: &ITfContext, output: &EngineOutput) -> Result<()> {
        if !output.committed.is_empty() {
            self.commit_text(context, &output.committed)?;
            return Ok(());
        }

        if !output.display.is_empty() {
            self.set_composition_text(context, &output.display)?;
        } else {
            self.end_composition()?;
        }

        Ok(())
    }

    /// Composition を開始する（まだ開始していない場合）。
    fn start_composition(&self, context: &ITfContext) -> Result<()> {
        let mut comp = self.composition.lock().unwrap();
        if comp.is_some() {
            return Ok(());
        }

        let context_composition: ITfContextComposition = context.cast()?;
        let tid = *self.client_id.lock().unwrap();

        unsafe {
            let edit_cookie = context.RequestEditSession(
                tid,
                &EditSessionStartComposition,
                TF_ES_READWRITE | TF_ES_SYNC,
            )?;
            let _ = edit_cookie;
        }

        // Composition 開始は EditSession 内で行う必要がある。
        // 簡易実装: ITfInsertAtSelection → StartComposition
        let _ = context_composition;

        // Composition オブジェクトをキャッシュ
        // 注意: 実際の実装では EditSession コールバック内で開始する
        *comp = None; // placeholder

        Ok(())
    }

    /// Composition テキストを設定する。
    fn set_composition_text(&self, context: &ITfContext, text: &str) -> Result<()> {
        self.start_composition(context)?;

        let comp = self.composition.lock().unwrap();
        if let Some(ref composition) = *comp {
            unsafe {
                let range = composition.GetRange()?;
                let wide: Vec<u16> = text.encode_utf16().collect();
                range.SetText(0, &wide)?;
            }
        }

        Ok(())
    }

    /// テキストを確定し、Composition を終了する。
    fn commit_text(&self, context: &ITfContext, text: &str) -> Result<()> {
        self.set_composition_text(context, text)?;
        self.end_composition()?;
        Ok(())
    }

    /// Composition を終了する。
    fn end_composition(&self) -> Result<()> {
        let mut comp = self.composition.lock().unwrap();
        if let Some(ref composition) = *comp {
            unsafe {
                composition.EndComposition()?;
            }
        }
        *comp = None;
        Ok(())
    }
}

// --- ITfTextInputProcessorEx ---

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Option<&ITfThreadMgr>, tid: u32, _flags: u32) -> Result<()> {
        let thread_mgr = ptim.ok_or(E_INVALIDARG)?.clone();

        let keystroke_mgr: ITfKeystrokeMgr = thread_mgr.cast()?;
        let self_sink: ITfKeyEventSink = self.cast()?;
        unsafe {
            keystroke_mgr.AdviseKeyEventSink(tid, &self_sink, TRUE)?;
        }

        *self.thread_mgr.lock().unwrap() = Some(thread_mgr);
        *self.client_id.lock().unwrap() = tid;
        *self.ime_on.lock().unwrap() = true;

        Ok(())
    }

    fn Deactivate(&self) -> Result<()> {
        let thread_mgr = self.thread_mgr.lock().unwrap().take();
        let tid = *self.client_id.lock().unwrap();

        if let Some(thread_mgr) = thread_mgr {
            if let Ok(keystroke_mgr) = thread_mgr.cast::<ITfKeystrokeMgr>() {
                unsafe {
                    let _ = keystroke_mgr.UnadviseKeyEventSink(tid);
                }
            }
        }

        *self.ime_on.lock().unwrap() = false;
        self.end_composition().ok();
        Ok(())
    }
}

// --- ITfTextInputProcessor ---

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        ITfTextInputProcessorEx_Impl::Deactivate(self)
    }
}

// --- ITfKeyEventSink ---

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _fforeground: BOOL) -> Result<()> {
        Ok(())
    }

    fn OnTestKeyDown(
        &self,
        _pic: Option<&ITfContext>,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        let ime_on = *self.ime_on.lock().unwrap();
        let modifiers = modifiers_from_keyboard_state();
        let vk = wparam.0 as u16;

        match key_mapping::map_key(vk, &modifiers, ime_on) {
            Some(_) => Ok(TRUE),
            None => Ok(FALSE),
        }
    }

    fn OnKeyDown(&self, pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let ime_on = *self.ime_on.lock().unwrap();
        let modifiers = modifiers_from_keyboard_state();
        let vk = wparam.0 as u16;

        let Some(command) = key_mapping::map_key(vk, &modifiers, ime_on) else {
            return Ok(FALSE);
        };

        let mut engine = self.engine.lock().unwrap();
        let output = engine.process(command);
        drop(engine);

        if let Some(context) = pic {
            self.update_composition(context, &output)?;
        }

        Ok(TRUE)
    }

    fn OnTestKeyUp(
        &self,
        _pic: Option<&ITfContext>,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Result<BOOL> {
        Ok(FALSE)
    }

    fn OnKeyUp(&self, _pic: Option<&ITfContext>, _wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(FALSE)
    }
}

/// キーボードの現在の修飾キー状態を取得する。
fn modifiers_from_keyboard_state() -> Modifiers {
    unsafe {
        Modifiers {
            shift: GetKeyState(key_mapping::VK_SHIFT as i32) < 0,
            ctrl: GetKeyState(key_mapping::VK_CONTROL as i32) < 0,
            alt: GetKeyState(key_mapping::VK_MENU as i32) < 0,
        }
    }
}

// EditSession (簡易版: Composition 開始用)
struct EditSessionStartComposition;
