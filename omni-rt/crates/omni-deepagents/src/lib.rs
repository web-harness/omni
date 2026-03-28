use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "deepagents")]
extern "C" {
    pub type DeepAgent;

    #[wasm_bindgen(js_name = "createDeepAgent")]
    pub fn create_deep_agent(options: JsValue) -> DeepAgent;

    #[wasm_bindgen(method)]
    pub fn invoke(this: &DeepAgent, input: JsValue) -> js_sys::Promise;
}

#[wasm_bindgen(module = "deepagents")]
extern "C" {
    pub type Agent;

    #[wasm_bindgen(method)]
    pub fn name(this: &Agent) -> String;

    #[wasm_bindgen(method)]
    pub fn description(this: &Agent) -> String;
}

pub fn create_agent_with_model(model: &str, system_prompt: &str) -> DeepAgent {
    let options = serde_wasm_bindgen::to_value(&serde_json::json!({
        "model": model,
        "systemPrompt": system_prompt
    }))
    .unwrap_or(JsValue::NULL);
    create_deep_agent(options)
}
