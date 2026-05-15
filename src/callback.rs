use once_cell::sync::Lazy;
use std::ffi::{c_char, CStr};
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;
use TaiLerDownloader::core::downloader::Event;

/// Raw event from the TLD C callback.
#[derive(Debug, Clone)]
pub struct CallbackEvent {
    pub event: Event,
    pub data: serde_json::Value,
}

static EVENT_TX: Lazy<Mutex<Option<UnboundedSender<CallbackEvent>>>> =
    Lazy::new(|| Mutex::new(None));

/// Register the sender that the C callback will write to.
/// Returns the old sender if one was already set.
pub fn register_sender(tx: UnboundedSender<CallbackEvent>) -> Option<UnboundedSender<CallbackEvent>> {
    EVENT_TX.lock().unwrap().replace(tx)
}

/// Clear the registered sender (called when download completes).
pub fn clear_sender() {
    *EVENT_TX.lock().unwrap() = None;
}

/// C callback for TLD progress events.
/// This is called from a C thread, so we parse strings on the stack
/// and push through the channel without blocking.
extern "C" fn progress_callback(event_ptr: *const c_char, msg_ptr: *const c_char) {
    let event_str = unsafe { c_str_to_string(event_ptr) };
    let msg_str = unsafe { c_str_to_string(msg_ptr) };

    let event: Event = match serde_json::from_str(&event_str) {
        Ok(e) => e,
        Err(_) => return,
    };

    let data: serde_json::Value = match serde_json::from_str(&msg_str) {
        Ok(d) => d,
        Err(_) => serde_json::Value::Object(Default::default()),
    };

    if let Ok(guard) = EVENT_TX.lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(CallbackEvent { event, data });
        }
    }
}

unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

/// Return the extern "C" function pointer suitable for TLD's ProgressCallback type.
pub fn get_callback() -> TaiLerDownloader::core::downloader::ProgressCallback {
    progress_callback as TaiLerDownloader::core::downloader::ProgressCallback
}
