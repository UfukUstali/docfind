use docfind_core::build_index;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(msg: &str);
}

#[wasm_bindgen]
pub fn build(documents_json: &str) -> Result<Vec<u8>, JsValue> {
	let items: Vec<docfind_core::InputItem> = serde_json::from_str(documents_json)
		.map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;

	let index = build_index(items)
		.map_err(|e| JsValue::from_str(&format!("Failed to build index: {}", e)))?;

	index
		.to_bytes()
		.map_err(|e| JsValue::from_str(&format!("Failed to serialize index: {}", e)))
}
