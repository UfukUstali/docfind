mod tests {
	use crate::Index;
	use crate::{InputItem, SearchTokens, FsstStrVec};
	use crate::{build_index, search};

	#[test]
	fn test_fsst_str_vec_basic() {
		let strings = vec!["hello", "world", "rust", "search"];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 4);

		assert_eq!(vec.get(0), Some("hello".to_string()));
		assert_eq!(vec.get(1), Some("world".to_string()));
		assert_eq!(vec.get(2), Some("rust".to_string()));
		assert_eq!(vec.get(3), Some("search".to_string()));
	}

	#[test]
	fn test_fsst_str_vec_out_of_bounds() {
		let strings = vec!["hello", "world"];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.get(5), None);
		assert_eq!(vec.get(100), None);
	}

	#[test]
	fn test_fsst_str_vec_empty() {
		let strings: Vec<&str> = vec![];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 0);
		assert_eq!(vec.get(0), None);
	}

	#[test]
	fn test_fsst_str_vec_single_item() {
		let strings = vec!["solo"];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 1);
		assert_eq!(vec.get(0), Some("solo".to_string()));
		assert_eq!(vec.get(1), None);
	}

	#[test]
	fn test_fsst_str_vec_long_strings() {
		let strings = vec![
			"This is a much longer string that should compress well with FSST",
			"Another long string with similar patterns and repeated words",
			"The third long string continues the pattern with more text",
		];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 3);
		assert_eq!(vec.get(0), Some(strings[0].to_string()));
		assert_eq!(vec.get(1), Some(strings[1].to_string()));
		assert_eq!(vec.get(2), Some(strings[2].to_string()));
	}

	#[test]
	fn test_fsst_str_vec_unicode() {
		let strings = vec!["Hello 世界", "Rust 🦀", "Café ☕"];
		let vec = FsstStrVec::from_strings(&strings);

		assert_eq!(vec.len(), 3);
		assert_eq!(vec.get(0), Some("Hello 世界".to_string()));
		assert_eq!(vec.get(1), Some("Rust 🦀".to_string()));
		assert_eq!(vec.get(2), Some("Café ☕".to_string()));
	}

	#[test]
	fn test_input_item_creation() {
		let item = InputItem {
			id: "item-001".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("hello world".to_string()), 100),
				(SearchTokens::Tokens(vec!["tag1".to_string(), "tag2".to_string()]), 50),
			],
		};

		assert_eq!(item.id, "item-001");
		assert_eq!(item.search_terms.len(), 2);
	}

	#[test]
	fn test_search_tokens_raw() {
		let tokens = SearchTokens::Raw("Hello World TEST".to_string());
		let keywords: Vec<String> = match tokens {
			SearchTokens::Raw(s) => s
				.split_whitespace()
				.map(|w| w.to_lowercase())
				.filter(|w| !w.is_empty())
				.collect(),
			SearchTokens::Tokens(t) => t.into_iter().map(|s| s.to_lowercase()).collect(),
		};

		assert_eq!(keywords, vec!["hello", "world", "test"]);
	}

	#[test]
	fn test_search_tokens_pre_tokenized() {
		let tokens = SearchTokens::Tokens(vec!["TAG1".to_string(), "TAG2".to_string()]);
		let keywords: Vec<String> = match tokens {
			SearchTokens::Raw(s) => s
				.split_whitespace()
				.map(|w| w.to_lowercase())
				.filter(|w| !w.is_empty())
				.collect(),
			SearchTokens::Tokens(t) => t.into_iter().map(|s| s.to_lowercase()).collect(),
		};

		assert_eq!(keywords, vec!["tag1", "tag2"]);
	}

	#[test]
	fn test_build_index_simple() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("rust programming".to_string()), 90),
				],
			},
			InputItem {
				id: "item-002".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("python guide".to_string()), 90),
				],
			},
		];

		let index = build_index(items);
		assert!(index.is_ok());

		let index = index.unwrap();
		assert_eq!(index.ids.len(), 2);
	}

	#[test]
	fn test_build_index_empty() {
		let items: Vec<InputItem> = vec![];
		let index = build_index(items);
		assert!(index.is_ok());

		let index = index.unwrap();
		assert_eq!(index.ids.len(), 0);
	}

	#[test]
	fn test_build_index_single_item() {
		let items = vec![InputItem {
			id: "item-001".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("test query".to_string()), 100),
			],
		}];

		let index = build_index(items);
		assert!(index.is_ok());

		let index = index.unwrap();
		assert_eq!(index.ids.len(), 1);
	}

	#[test]
	fn test_build_index_weighted_terms() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("important".to_string()), 100),
					(SearchTokens::Raw("secondary".to_string()), 50),
				],
			},
		];

		let index = build_index(items).unwrap();
		assert_eq!(index.ids.len(), 1);
	}

	#[test]
	fn test_index_serialization() {
		let items = vec![InputItem {
			id: "item-001".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("test".to_string()), 100),
			],
		}];

		let index = build_index(items).unwrap();

		let buffer = index.to_bytes().unwrap();
		assert!(!buffer.is_empty());

		let deserialized = Index::from_bytes(&buffer);
		assert!(deserialized.is_ok());

		let deserialized_index = deserialized.unwrap();
		assert_eq!(deserialized_index.ids.len(), index.ids.len());
	}

	#[test]
	fn test_index_serialization_roundtrip() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("first item".to_string()), 100),
				],
			},
			InputItem {
				id: "item-002".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("second item".to_string()), 100),
				],
			},
		];

		let original_index = build_index(items).unwrap();

		let buffer1 = original_index.to_bytes().unwrap();
		let index1 = Index::from_bytes(&buffer1).unwrap();

		let buffer2 = index1.to_bytes().unwrap();
		let index2 = Index::from_bytes(&buffer2).unwrap();

		assert_eq!(index2.ids.len(), original_index.ids.len());
	}

	#[test]
	fn test_search_single_word() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("rust programming".to_string()), 90),
				],
			},
			InputItem {
				id: "item-002".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("python guide".to_string()), 90),
				],
			},
		];

		let index = build_index(items).unwrap();
		let results = search(&index, "rust", 10).unwrap();

		assert!(!results.is_empty());
		assert_eq!(results[0], "item-001");
	}

	#[test]
	fn test_search_case_insensitive() {
		let items = vec![InputItem {
			id: "item-001".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("JavaScript Tutorial".to_string()), 90),
			],
		}];

		let index = build_index(items).unwrap();

		let results_lower = search(&index, "javascript", 10).unwrap();
		let results_upper = search(&index, "JAVASCRIPT", 10).unwrap();
		let results_mixed = search(&index, "JavaScript", 10).unwrap();

		assert!(!results_lower.is_empty());
		assert!(!results_upper.is_empty());
		assert!(!results_mixed.is_empty());

		assert_eq!(results_lower[0], "item-001");
		assert_eq!(results_upper[0], "item-001");
		assert_eq!(results_mixed[0], "item-001");
	}

	#[test]
	fn test_search_no_results() {
		let items = vec![InputItem {
			id: "item-001".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("rust programming".to_string()), 90),
			],
		}];

		let index = build_index(items).unwrap();
		let results = search(&index, "nonexistent", 10).unwrap();

		assert!(results.is_empty());
	}

	#[test]
	fn test_search_empty_query() {
		let items = vec![InputItem {
			id: "item-001".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("test".to_string()), 100),
			],
		}];

		let index = build_index(items).unwrap();
		let results = search(&index, "", 10).unwrap();

		assert!(results.len() <= 1);
	}

	#[test]
	fn test_search_multiple_words() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("wireless audio".to_string()), 90),
				],
			},
			InputItem {
				id: "item-002".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("wireless mouse".to_string()), 90),
				],
			},
			InputItem {
				id: "item-003".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("python book".to_string()), 90),
				],
			},
		];

		let index = build_index(items).unwrap();
		let results = search(&index, "wireless", 10).unwrap();

		assert!(results.len() >= 2);
		assert!(results.contains(&"item-001".to_string()));
		assert!(results.contains(&"item-002".to_string()));
	}

	#[test]
	fn test_search_partial_word_match() {
		let items = vec![InputItem {
			id: "item-001".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("debugging tools".to_string()), 90),
			],
		}];

		let index = build_index(items).unwrap();
		let results = search(&index, "debug", 10).unwrap();

		assert!(!results.is_empty());
	}

	#[test]
	fn test_search_max_results_limit() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("product one".to_string()), 90),
				],
			},
			InputItem {
				id: "item-002".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("product two".to_string()), 90),
				],
			},
			InputItem {
				id: "item-003".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("product three".to_string()), 90),
				],
			},
			InputItem {
				id: "item-004".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("product four".to_string()), 90),
				],
			},
		];

		let index = build_index(items).unwrap();

		let results_2 = search(&index, "product", 2).unwrap();
		let results_3 = search(&index, "product", 3).unwrap();
		let results_10 = search(&index, "product", 10).unwrap();

		assert!(results_2.len() <= 2);
		assert!(results_3.len() <= 3);
		assert!(results_10.len() <= 10);
	}

	#[test]
	fn test_search_with_pre_tokenized() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Tokens(vec!["tag1".to_string(), "tag2".to_string()]), 100),
				],
			},
			InputItem {
				id: "item-002".to_string(),
				search_terms: vec![
					(SearchTokens::Tokens(vec!["tag2".to_string(), "tag3".to_string()]), 100),
				],
			},
		];

		let index = build_index(items).unwrap();
		let results = search(&index, "tag1", 10).unwrap();

		assert!(!results.is_empty());
		assert!(results.contains(&"item-001".to_string()));
	}

	#[test]
	fn test_search_with_typo() -> Result<(), Box<dyn std::error::Error>> {
		let ids = vec!["item-001", "item-002", "item-003"];
		let ids_fsst = FsstStrVec::from_strings(&ids);

		let keyword_to_items: Vec<Vec<(usize, u8)>> = vec![
			vec![(1, 1)],
			vec![(0, 10), (2, 4)],
			vec![(0, 5), (1, 3)],
		];

		let mut fst_builder = fst::MapBuilder::memory();
		fst_builder.insert("audio", 0).unwrap();
		fst_builder.insert("books", 1).unwrap();
		fst_builder.insert("electronics", 2).unwrap();
		let fst = fst_builder.into_inner()?;

		let index = Index {
			fst,
			ids: ids_fsst,
			keyword_to_items,
		};

		let results = search(&index, "audiio", 10)?;
		assert_eq!(results.len(), 1, "Expected 1 result for 'audiio'");

		Ok(())
	}

	#[test]
	fn test_search_score_accumulation() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("rust".to_string()), 50),
					(SearchTokens::Raw("programming".to_string()), 50),
				],
			},
			InputItem {
				id: "item-002".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("rust".to_string()), 100),
				],
			},
		];

		let index = build_index(items).unwrap();
		let results = search(&index, "rust programming", 10).unwrap();

		assert!(!results.is_empty());
		assert_eq!(results[0], "item-001");
	}

	#[test]
	fn test_search_weighted_terms() {
		let items = vec![
			InputItem {
				id: "item-001".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("keyword".to_string()), 100),
				],
			},
			InputItem {
				id: "item-002".to_string(),
				search_terms: vec![
					(SearchTokens::Raw("keyword".to_string()), 50),
				],
			},
		];

		let index = build_index(items).unwrap();
		let results = search(&index, "keyword", 10).unwrap();

		assert_eq!(results.len(), 2);
		assert_eq!(results[0], "item-001");
		assert_eq!(results[1], "item-002");
	}

	#[test]
	fn test_search_empty_id() {
		let items = vec![InputItem {
			id: "".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("test".to_string()), 100),
			],
		}];

		let index = build_index(items).unwrap();
		let results = search(&index, "test", 10).unwrap();

		assert!(!results.is_empty());
		assert_eq!(results[0], "");
	}

	#[test]
	fn test_deduplicate_keywords_per_item() {
		let items = vec![InputItem {
			id: "item-001".to_string(),
			search_terms: vec![
				(SearchTokens::Raw("test test duplicate".to_string()), 100),
			],
		}];

		let index = build_index(items).unwrap();
		let results = search(&index, "test", 10).unwrap();

		assert!(!results.is_empty());
	}
}
