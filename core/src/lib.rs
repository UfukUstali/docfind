use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

/// A minimal FSST-compressed vector of UTF-8 strings with random access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsstStrVec {
	dict_syms: Vec<[u8; 8]>,
	dict_lens: Vec<u8>,
	offsets: Vec<u32>,
	data: Vec<u8>,
}

impl FsstStrVec {
	fn from_strings(strings: &[impl AsRef<str>]) -> Self {
		let sample: Vec<&[u8]> = strings.iter().map(|s| s.as_ref().as_bytes()).collect();
		let compressor = fsst::Compressor::train(&sample);

		let syms: Vec<fsst::Symbol> = compressor.symbol_table().to_vec();
		let lens: Vec<u8> = compressor.symbol_lengths().to_vec();

		let mut offsets = Vec::with_capacity(strings.len());
		let mut data = Vec::new();
		for s in strings {
			offsets.push(data.len() as u32);
			let c = compressor.compress(s.as_ref().as_bytes());
			data.extend_from_slice(&c);
		}

		let dict_syms: Vec<[u8; 8]> = syms
			.into_iter()
			.map(|sym| u64::to_le_bytes(sym.to_u64()))
			.collect();

		Self {
			dict_syms,
			dict_lens: lens,
			offsets,
			data,
		}
	}

	pub fn len(&self) -> usize {
		self.offsets.len()
	}

	pub fn get(&self, i: usize) -> Option<String> {
		if i >= self.len() {
			return None;
		}
		let start = self.offsets[i] as usize;
		let end = if i + 1 < self.len() {
			self.offsets[i + 1] as usize
		} else {
			self.data.len()
		};
		let codes = &self.data[start..end];

		let syms: Vec<fsst::Symbol> = self
			.dict_syms
			.iter()
			.map(fsst::Symbol::from_slice)
			.collect();
		let decomp = fsst::Decompressor::new(&syms, &self.dict_lens);

		let bytes = decomp.decompress(codes);
		Some(String::from_utf8(bytes).expect("FSST preserves UTF-8 for UTF-8 input"))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputItem {
	pub id: String,
	#[serde(deserialize_with = "parse_search_terms")]
	pub search_terms: Vec<(SearchTokens, u8)>,
}

fn parse_search_terms<'de, D>(deserializer: D) -> Result<Vec<(SearchTokens, u8)>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	#[derive(Deserialize)]
	struct TermEntry {
		r#type: String,
		value: serde_json::Value,
		weight: u8,
	}

	let entries: Vec<TermEntry> = serde::Deserialize::deserialize(deserializer)?;
	let mut result = Vec::with_capacity(entries.len());
	for e in entries {
		let tokens = match (e.r#type.as_str(), e.value) {
			("raw", serde_json::Value::String(s)) => SearchTokens::Raw(s),
			("tokens", serde_json::Value::Array(arr)) => {
				let strings: Vec<String> = arr
					.into_iter()
					.filter_map(|v| v.as_str().map(String::from))
					.collect();
				SearchTokens::Tokens(strings)
			}
			_ => return Err(serde::de::Error::custom(format!(
				"invalid search term type: expected 'raw' or 'tokens', got '{}'",
				e.r#type
			))),
		};
		result.push((tokens, e.weight));
	}
	Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum SearchTokens {
	Raw(String),
	Tokens(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
	fst: Vec<u8>,
	ids: FsstStrVec,
	keyword_to_items: Vec<Vec<(usize, u8)>>,
}

impl Index {
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
		let index: Index = postcard::from_bytes(bytes)?;
		Ok(index)
	}

	pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
		Ok(postcard::to_allocvec(self)?)
	}
}

pub fn build_index(items: Vec<InputItem>) -> Result<Index, Box<dyn std::error::Error>> {
	let mut ids: Vec<String> = Vec::new();
	let mut keywords_to_items: HashMap<String, Vec<(usize, u8)>> = HashMap::new();

	for (item_index, item) in items.iter().enumerate() {
		ids.push(item.id.clone());

		let mut seen_keywords: HashSet<String> = HashSet::new();

		for (tokens, weight) in &item.search_terms {
			match tokens {
				SearchTokens::Raw(raw) => {
					for keyword in raw.split_whitespace() {
						let keyword = keyword.to_lowercase();
						if keyword.is_empty() || seen_keywords.contains(&keyword) {
							continue;
						}
						seen_keywords.insert(keyword.clone());

						keywords_to_items
							.entry(keyword)
							.or_default()
							.push((item_index, *weight));
					}
				}
				SearchTokens::Tokens(tokens) => {
					for keyword in tokens {
						let keyword = keyword.to_lowercase();
						if keyword.is_empty() || seen_keywords.contains(&keyword) {
							continue;
						}
						seen_keywords.insert(keyword.clone());

						keywords_to_items
							.entry(keyword)
							.or_default()
							.push((item_index, *weight));
					}
				}
			}
		}
	}

	let mut fst_builder = fst::MapBuilder::memory();
	let mut keyword_to_items: Vec<Vec<(usize, u8)>> = Vec::new();
	let mut sorted_keywords: Vec<String> = keywords_to_items.keys().cloned().collect();
	sorted_keywords.sort();

	for (index, keyword) in sorted_keywords.iter().enumerate() {
		fst_builder.insert(keyword, index as u64)?;

		let mut item_scores = keywords_to_items.get(keyword).unwrap().clone();
		item_scores.sort_by(|a, b| b.1.cmp(&a.1));

		keyword_to_items.push(item_scores);
	}

	let fst = fst_builder.into_inner().unwrap();
	let ids_fsst = FsstStrVec::from_strings(&ids);

	Ok(Index {
		fst,
		ids: ids_fsst,
		keyword_to_items,
	})
}

pub fn search(
	index: &Index,
	query: &str,
	max_results: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
	use fst::automaton::Levenshtein;
	use fst::map::OpBuilder;
	use fst::{Automaton, Streamer};
	use std::collections::HashSet;

	let map = fst::Map::new(&index.fst)?;

	let mut query_words: HashSet<String> = query
		.split_whitespace()
		.map(|w| w.to_lowercase())
		.filter(|w| !w.is_empty())
		.collect();

	query_words.insert(query.to_lowercase());

	let mut keyword_indices: Vec<u64> = Vec::new();

	for query_word in query_words {
		use fst::automaton::Str;

		let lev = Levenshtein::new(query_word.as_str(), 1)?;
		let prefix = Str::new(query_word.as_str()).starts_with();

		let mut op = OpBuilder::new()
			.add(map.search(lev))
			.add(map.search(prefix))
			.union();

		while let Some((_keyword, indexed_value)) = op.next() {
			let keyword_index = indexed_value.to_vec().get(0).unwrap().value;
			keyword_indices.push(keyword_index);
		}
	}

	let mut items: HashMap<usize, u8> = HashMap::new();

	for keyword_index in keyword_indices {
		let matching_items = &index.keyword_to_items[keyword_index as usize];

		for (item_index, score) in matching_items {
			let entry = items.entry(*item_index).or_insert(0);
			*entry = entry.saturating_add(*score);
		}
	}

	let mut items: Vec<(usize, u8)> = items.into_iter().collect();
	items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
	items.truncate(max_results);

	let mut result: Vec<String> = Vec::new();

	for (item_index, _) in items {
		let id = index
			.ids
			.get(item_index)
			.ok_or_else(|| "Failed to get item id")?;

		result.push(id);
	}

	Ok(result)
}

#[cfg(test)]
mod tests;
