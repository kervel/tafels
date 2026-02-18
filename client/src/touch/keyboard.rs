use bevy::prelude::*;

pub struct SoftKeyboardPlugin;

impl Plugin for SoftKeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SoftKeyboardInput::default());

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Startup, setup_hidden_input)
            .add_systems(Update, drain_soft_keyboard);
    }
}

#[derive(Resource, Default)]
pub struct SoftKeyboardInput {
    pub buffer: Vec<char>,
}

// ---- WASM implementation ----

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::sync::{Arc, Mutex};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{HtmlInputElement, InputEvent};

    /// Shared buffer between JS callback and Bevy system.
    pub static KEYBOARD_BUFFER: std::sync::LazyLock<Arc<Mutex<Vec<char>>>> =
        std::sync::LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

    /// Track if backspace was pressed (separate from input event).
    pub static BACKSPACE_COUNT: std::sync::LazyLock<Arc<Mutex<u32>>> =
        std::sync::LazyLock::new(|| Arc::new(Mutex::new(0)));

    pub fn create_hidden_input() {
        let Some(window) = web_sys::window() else { return };
        let Some(document) = window.document() else { return };

        // Don't create duplicate
        if document.get_element_by_id("bevy-kb").is_some() {
            return;
        }

        let Ok(el) = document.create_element("input") else { return };
        el.set_id("bevy-kb");
        let _ = el.set_attribute("type", "text");
        let _ = el.set_attribute("autocomplete", "off");
        let _ = el.set_attribute("autocapitalize", "off");
        let _ = el.set_attribute("style",
            "position:fixed; top:-100px; left:-100px; opacity:0; width:1px; height:1px; font-size:16px;");

        let Ok(input_el) = el.clone().dyn_into::<HtmlInputElement>() else { return };

        // Listen for input events
        let buffer = KEYBOARD_BUFFER.clone();
        let input_closure = Closure::wrap(Box::new(move |event: InputEvent| {
            if let Some(data) = event.data() {
                if let Ok(mut buf) = buffer.lock() {
                    for c in data.chars() {
                        buf.push(c);
                    }
                }
            }
        }) as Box<dyn FnMut(InputEvent)>);

        let _ = input_el.add_event_listener_with_callback(
            "input",
            input_closure.as_ref().unchecked_ref(),
        );
        input_closure.forget();

        // Listen for keydown to catch backspace
        let bs_count = BACKSPACE_COUNT.clone();
        let keydown_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if event.key() == "Backspace" {
                if let Ok(mut count) = bs_count.lock() {
                    *count += 1;
                }
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        let _ = input_el.add_event_listener_with_callback(
            "keydown",
            keydown_closure.as_ref().unchecked_ref(),
        );
        keydown_closure.forget();

        if let Some(body) = document.body() {
            let _ = body.append_child(&el);
        }
    }

    pub fn focus_input() {
        let Some(window) = web_sys::window() else { return };
        let Some(document) = window.document() else { return };
        if let Some(el) = document.get_element_by_id("bevy-kb") {
            if let Ok(input) = el.dyn_into::<HtmlInputElement>() {
                // Clear value so "input" event fires for repeated chars
                input.set_value("");
                let _ = input.focus();
            }
        }
    }

    pub fn blur_input() {
        let Some(window) = web_sys::window() else { return };
        let Some(document) = window.document() else { return };
        if let Some(el) = document.get_element_by_id("bevy-kb") {
            if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                let _ = html_el.blur();
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn setup_hidden_input() {
    wasm::create_hidden_input();
}

#[cfg(target_arch = "wasm32")]
fn drain_soft_keyboard(mut keyboard: ResMut<SoftKeyboardInput>) {
    // Drain characters
    if let Ok(mut buf) = wasm::KEYBOARD_BUFFER.lock() {
        keyboard.buffer.append(&mut *buf);
    }
    // Drain backspaces as a special '\x08' marker
    if let Ok(mut count) = wasm::BACKSPACE_COUNT.lock() {
        for _ in 0..*count {
            keyboard.buffer.push('\x08'); // backspace marker
        }
        *count = 0;
    }
}

/// Call from Bevy systems to show the soft keyboard.
#[allow(unused)]
pub fn show_soft_keyboard() {
    #[cfg(target_arch = "wasm32")]
    wasm::focus_input();
}

/// Call from Bevy systems to hide the soft keyboard.
#[allow(unused)]
pub fn hide_soft_keyboard() {
    #[cfg(target_arch = "wasm32")]
    wasm::blur_input();
}
