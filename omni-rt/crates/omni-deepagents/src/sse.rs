use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SseEvent {
    Message(Value),
    MessageComplete(Value),
    Values(Value),
    Done,
    Error(String),
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::SseEvent;
    use js_sys::{Object, Reflect, Uint8Array};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    #[wasm_bindgen]
    extern "C" {
        type ReadableStreamDefaultReader;

        #[wasm_bindgen(method, catch)]
        fn read(this: &ReadableStreamDefaultReader) -> Result<js_sys::Promise, JsValue>;

        #[wasm_bindgen(js_namespace = globalThis, catch)]
        fn fetch(resource: &str, init: &JsValue) -> Result<js_sys::Promise, JsValue>;
    }

    pub struct SseStream {
        buffer: String,
        done: bool,
        reader: Option<ReadableStreamDefaultReader>,
    }

    impl SseStream {
        pub async fn connect(url: &str, body: &str) -> Result<Self, std::io::Error> {
            let init = Object::new();
            Reflect::set(&init, &"method".into(), &"POST".into()).ok();
            Reflect::set(&init, &"body".into(), &body.into()).ok();
            let headers = Object::new();
            Reflect::set(&headers, &"Content-Type".into(), &"application/json".into()).ok();
            Reflect::set(&init, &"headers".into(), &headers.into()).ok();

            let promise =
                fetch(url, &init.into()).map_err(|e| std::io::Error::other(format!("{e:?}")))?;
            let response_val = JsFuture::from(promise)
                .await
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;

            let body_val = Reflect::get(&response_val, &"body".into())
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
            let reader_val = js_sys::Reflect::get(&body_val, &"getReader".into())
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
            let reader_fn: js_sys::Function = reader_val
                .dyn_into()
                .map_err(|_| std::io::Error::other("getReader not a function"))?;
            let reader = reader_fn
                .call0(&body_val)
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
            let reader: ReadableStreamDefaultReader = reader
                .dyn_into()
                .map_err(|_| std::io::Error::other("reader cast failed"))?;

            Ok(Self {
                buffer: String::new(),
                done: false,
                reader: Some(reader),
            })
        }

        pub async fn next_event(&mut self) -> Result<Option<SseEvent>, std::io::Error> {
            loop {
                if let Some(event) = super::parse_buffered_event(&mut self.buffer)? {
                    return Ok(Some(event));
                }

                if self.done {
                    return Ok(None);
                }

                let reader = match &self.reader {
                    Some(reader) => reader,
                    None => return Ok(None),
                };

                let read_promise = reader
                    .read()
                    .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
                let result = JsFuture::from(read_promise)
                    .await
                    .map_err(|e| std::io::Error::other(format!("{e:?}")))?;

                let done_val = Reflect::get(&result, &"done".into())
                    .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
                if done_val.as_bool() == Some(true) {
                    self.done = true;
                    return Ok(None);
                }

                let value = Reflect::get(&result, &"value".into())
                    .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
                let arr = Uint8Array::new(&value);
                self.buffer
                    .push_str(&String::from_utf8_lossy(&arr.to_vec()));
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::SseEvent;
    use futures_util::StreamExt;

    pub struct SseStream {
        buffer: String,
        done: bool,
        stream: futures_util::stream::BoxStream<'static, Result<bytes::Bytes, reqwest::Error>>,
    }

    impl SseStream {
        pub async fn connect(url: &str, body: &str) -> Result<Self, std::io::Error> {
            let response = reqwest::Client::new()
                .post(url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.to_string())
                .send()
                .await
                .map_err(|error| std::io::Error::other(error.to_string()))?;

            if !response.status().is_success() {
                return Err(std::io::Error::other(format!(
                    "stream request failed: {}",
                    response.status()
                )));
            }

            Ok(Self {
                buffer: String::new(),
                done: false,
                stream: response.bytes_stream().boxed(),
            })
        }

        pub async fn next_event(&mut self) -> Result<Option<SseEvent>, std::io::Error> {
            loop {
                if let Some(event) = super::parse_buffered_event(&mut self.buffer)? {
                    return Ok(Some(event));
                }

                if self.done {
                    return Ok(None);
                }

                match self.stream.next().await {
                    Some(Ok(chunk)) => self.buffer.push_str(&String::from_utf8_lossy(&chunk)),
                    Some(Err(error)) => {
                        return Err(std::io::Error::other(error.to_string()));
                    }
                    None => {
                        self.done = true;
                        return Ok(None);
                    }
                }
            }
        }
    }
}

pub use imp::SseStream;

fn parse_buffered_event(buffer: &mut String) -> Result<Option<SseEvent>, std::io::Error> {
    let Some(pos) = buffer.find("\n\n") else {
        return Ok(None);
    };

    let chunk = buffer[..pos].to_string();
    *buffer = buffer[pos + 2..].to_string();

    let mut event_name = None;
    let mut data_lines = Vec::new();
    for line in chunk.lines() {
        if let Some(name) = line.strip_prefix("event: ") {
            event_name = Some(name.trim().to_string());
            continue;
        }
        if let Some(data) = line.strip_prefix("data: ") {
            data_lines.push(data.to_string());
        }
    }

    let data = data_lines.join("\n");
    if let Some(name) = event_name.as_deref() {
        let event = match name {
            "message" | "messages/partial" => serde_json::from_str::<Value>(&data)
                .map(SseEvent::Message)
                .ok(),
            "messages/complete" => serde_json::from_str::<Value>(&data)
                .map(SseEvent::MessageComplete)
                .ok(),
            "values" => serde_json::from_str::<Value>(&data)
                .map(SseEvent::Values)
                .ok(),
            "end" => Some(SseEvent::Done),
            "error" => Some(SseEvent::Error(extract_error_message(&data))),
            _ => None,
        };
        if let Some(event) = event {
            return Ok(Some(event));
        }
    }

    Ok(serde_json::from_str::<SseEvent>(&data).ok())
}

fn extract_error_message(data: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(data) {
        if let Some(text) = value.get("message").and_then(Value::as_str) {
            return text.to_string();
        }
        if let Some(text) = value.as_str() {
            return text.to_string();
        }
    }
    data.to_string()
}
