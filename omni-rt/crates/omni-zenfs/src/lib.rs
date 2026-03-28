use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@zenfs/core")]
extern "C" {
    pub type FileSystem;

    #[wasm_bindgen(static_method_of = FileSystem)]
    pub fn configure(options: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(static_method_of = FileSystem)]
    pub fn fs() -> JsValue;
}

#[wasm_bindgen(module = "@zenfs/core")]
extern "C" {
    pub type InMemory;

    #[wasm_bindgen(constructor)]
    pub fn new() -> InMemory;
}

#[wasm_bindgen(module = "@zenfs/dom")]
extern "C" {
    pub type IndexedDB;

    #[wasm_bindgen(constructor)]
    pub fn new() -> IndexedDB;
}

pub fn configure_with_indexeddb() -> js_sys::Promise {
    let config = serde_wasm_bindgen::to_value(&serde_json::json!({
        "mounts": {
            "/": {
                "backend": "IndexedDB"
            }
        }
    }))
    .unwrap_or(JsValue::NULL);
    FileSystem::configure(config)
}
