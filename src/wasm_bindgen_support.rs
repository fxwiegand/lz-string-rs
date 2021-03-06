use js_sys::JsString;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compress(data: &str) -> Result<JsString, JsValue> {
    // TODO: Lossless conversion
    // let data: Vec<u32> = data.iter().map(u32::from).collect();

    let compressed = crate::compress_str(&data);

    JsString::from_code_point(&compressed)
}

#[wasm_bindgen]
pub fn decompress(data: JsString) -> JsValue {
    // Returning a String crashes?
    let data: Vec<u32> = data.iter().map(u32::from).collect();
    let decompressed = crate::decompress_str(&data)
        .map(JsString::from)
        .map(Into::into)
        .unwrap_or(JsValue::NULL);

    // TODO: Lossless conversion
    // JsString::from_code_point(&compressed)

    decompressed
}
