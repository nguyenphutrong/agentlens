use std::collections::HashMap;

use super::store::{Chunk, SearchResult};

/// Reciprocal Rank Fusion algorithm
/// Combines multiple result lists with different scoring
/// k is typically 60 (default constant from original RRF paper)
pub fn reciprocal_rank_fusion(
    k: f32,
    limit: usize,
    result_lists: Vec<Vec<SearchResult>>,
) -> Vec<SearchResult> {
    let mut scores: HashMap<String, f32> = HashMap::new();
    let mut chunk_map: HashMap<String, Chunk> = HashMap::new();

    for list in result_lists {
        for (rank, result) in list.iter().enumerate() {
            let id = &result.chunk.id;
            *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k + rank as f32 + 1.0);
            chunk_map.entry(id.clone()).or_insert(result.chunk.clone());
        }
    }

    let mut results: Vec<SearchResult> = scores
        .into_iter()
        .map(|(id, score)| {
            SearchResult::new(
                chunk_map.remove(&id).expect("Chunk must exist in map"),
                score,
            )
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    results
}

/// Simple text search for hybrid mode
/// Scores chunks based on word match ratio
pub fn text_search(chunks: &[Chunk], query: &str, limit: usize) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let words: Vec<String> = query_lower
        .split_whitespace()
        .filter(|w| w.len() >= 2)
        .map(|s| s.to_string())
        .collect();

    if words.is_empty() {
        return Vec::new();
    }

    let mut results: Vec<SearchResult> = chunks
        .iter()
        .filter_map(|chunk| {
            let content_lower = chunk.content.to_lowercase();

            // Exact phrase match bonus
            let phrase_bonus = if content_lower.contains(&query_lower) {
                0.5
            } else {
                0.0
            };

            // Word match score
            let match_count = words
                .iter()
                .filter(|w| content_lower.contains(w.as_str()))
                .count();

            if match_count > 0 {
                let base_score = match_count as f32 / words.len() as f32;
                Some(SearchResult::new(chunk.clone(), base_score + phrase_bonus))
            } else {
                None
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_chunk(id: &str, content: &str) -> Chunk {
        Chunk {
            id: id.to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 10,
            content: content.to_string(),
            vector: vec![0.1, 0.2, 0.3],
            hash: "abc123".to_string(),
            updated_at: Utc::now(),
            chunk_type: super::super::store::ChunkType::Function,
        }
    }

    #[test]
    fn test_rrf_single_list() {
        let list = vec![
            SearchResult::new(make_chunk("a", "content a"), 0.9),
            SearchResult::new(make_chunk("b", "content b"), 0.8),
        ];

        let results = reciprocal_rank_fusion(60.0, 10, vec![list]);
        assert_eq!(results.len(), 2);
        // First item should have higher score
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_rrf_multiple_lists() {
        let list1 = vec![
            SearchResult::new(make_chunk("a", "a"), 0.9),
            SearchResult::new(make_chunk("b", "b"), 0.8),
        ];
        let list2 = vec![
            SearchResult::new(make_chunk("b", "b"), 0.9),
            SearchResult::new(make_chunk("c", "c"), 0.8),
        ];

        let results = reciprocal_rank_fusion(60.0, 10, vec![list1, list2]);

        // "b" appears in both lists, should have highest combined score
        assert_eq!(results[0].chunk.id, "b");
    }

    #[test]
    fn test_text_search_basic() {
        let chunks = vec![
            make_chunk("1", "This is authentication code"),
            make_chunk("2", "Database connection handler"),
            make_chunk("3", "User login authentication flow"),
        ];

        let results = text_search(&chunks, "authentication", 10);
        assert_eq!(results.len(), 2);
        // Both chunks with "authentication" should be returned
        assert!(results.iter().any(|r| r.chunk.id == "1"));
        assert!(results.iter().any(|r| r.chunk.id == "3"));
    }

    #[test]
    fn test_text_search_phrase_bonus() {
        let chunks = vec![
            make_chunk("1", "user authentication"),
            make_chunk("2", "authentication for user accounts"),
        ];

        let results = text_search(&chunks, "user authentication", 10);
        assert_eq!(results.len(), 2);
        // Exact phrase match should have higher score
        assert!(results[0].score > results[1].score);
        assert_eq!(results[0].chunk.id, "1");
    }

    #[test]
    fn test_text_search_no_matches() {
        let chunks = vec![make_chunk("1", "hello world")];
        let results = text_search(&chunks, "foobar", 10);
        assert!(results.is_empty());
    }
}
