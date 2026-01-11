use std::boxed::Box;

use ruffle_core::FontQuery;

use ruffle_core::backend::ui::{
    DialogLoaderError, DialogResultFuture, FileDialogResult, FileFilter,
};
use ruffle_core::backend::ui::{
    FontDefinition, FullscreenError, LanguageIdentifier, MouseCursor, US_ENGLISH, UiBackend,
};

use chrono::{DateTime, Utc};
use sdl2::video::FullscreenType;
use sdl2::video::Window;
use url::Url;

/// UiBackend that does nothing.
pub struct SdlUiBackend {
    window: Box<Window>,
}

impl SdlUiBackend {
    pub fn new(window: Box<Window>) -> Self {
        Self { window }
    }
}

impl UiBackend for SdlUiBackend {
    fn mouse_visible(&self) -> bool {
        true
    }

    fn set_mouse_visible(&mut self, _visible: bool) {}

    fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}

    fn clipboard_content(&mut self) -> String {
        "".into()
    }

    fn set_clipboard_content(&mut self, _content: String) {}

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), FullscreenError> {
        //if is_full {
        let _ = self.window.set_fullscreen(FullscreenType::Desktop);
        let _ = self.window.set_bordered(false);
        //} else {
        //    let _ = self.window.set_fullscreen(FullscreenType::Off);
        //    let _ = self.window.set_bordered(true);
        //}
        Ok(())
    }

    fn display_root_movie_download_failed_message(&self, _invalid_swf: bool, _fetch_error: String) {
    }

    fn message(&self, _message: &str) {}

    fn display_unsupported_video(&self, _url: Url) {}

    fn load_device_font(&self, _query: &FontQuery, _register: &mut dyn FnMut(FontDefinition)) {}

    fn sort_device_fonts(
        &self,
        _query: &FontQuery,
        _register: &mut dyn FnMut(FontDefinition),
    ) -> Vec<FontQuery> {
        Vec::new()
    }

    fn open_virtual_keyboard(&self) {
        self.window.subsystem().text_input().start();
    }

    fn close_virtual_keyboard(&self) {
        self.window.subsystem().text_input().stop();
    }

    fn language(&self) -> LanguageIdentifier {
        US_ENGLISH.clone()
    }

    fn display_file_open_dialog(
        &mut self,
        _filters: Vec<FileFilter>,
    ) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            let result: Result<Box<dyn FileDialogResult>, DialogLoaderError> =
                Ok(Box::new(NullFileDialogResult::new()));
            result
        }))
    }

    fn close_file_dialog(&mut self) {}

    fn display_file_save_dialog(
        &mut self,
        _file_name: String,
        _domain: String,
    ) -> Option<DialogResultFuture> {
        None
    }
}

pub struct NullFileDialogResult {}

impl NullFileDialogResult {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for NullFileDialogResult {
    fn default() -> Self {
        NullFileDialogResult::new()
    }
}

impl FileDialogResult for NullFileDialogResult {
    fn is_cancelled(&self) -> bool {
        true
    }

    fn creation_time(&self) -> Option<DateTime<Utc>> {
        None
    }
    fn modification_time(&self) -> Option<DateTime<Utc>> {
        None
    }
    fn file_name(&self) -> Option<String> {
        None
    }
    fn size(&self) -> Option<u64> {
        None
    }
    fn file_type(&self) -> Option<String> {
        None
    }
    fn contents(&self) -> &[u8] {
        &[]
    }

    fn write_and_refresh(&mut self, _data: &[u8]) {}
}
