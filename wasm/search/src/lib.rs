use docfind_core::Index;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(msg: &str);
}

#[wasm_bindgen]
pub struct WasmIndex {
	inner: Index,
}

#[wasm_bindgen]
impl WasmIndex {
	#[wasm_bindgen(constructor)]
	pub fn new(index_bytes: &[u8]) -> Result<WasmIndex, JsValue> {
		let index = Index::from_bytes(index_bytes)
			.map_err(|e| JsValue::from_str(&format!("Failed to deserialize index: {}", e)))?;

		Ok(WasmIndex { inner: index })
	}

	pub fn search(&self, query: &str, max_results: Option<usize>) -> Result<Vec<String>, JsValue> {
		return docfind_core::search(
			&self.inner,
			query,
			max_results.unwrap_or(10),
		)
		.map_err(|e| JsValue::from_str(&format!("Search failed: {}", e)));
	}
}
