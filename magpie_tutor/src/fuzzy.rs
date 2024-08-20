//! Simple implementation for simple fuzzy sorting.

use std::{
    cmp::{max, min},
    fmt::Debug,
};

/// Result for fuzzy
#[derive(Debug)]
pub struct FuzzyRes<'a, T> {
    /// The rank of this result
    pub rank: f32,
    /// The item of this result
    pub data: &'a T,
}

/// Fuzzy match and return the best item.
pub fn fuzzy_best<'a, T, F>(
    value: &str,
    vec: Vec<&'a T>,
    threshold: f32,
    mut f: F,
) -> Option<FuzzyRes<'a, T>>
where
    F: FnMut(&T) -> &str,
    T: Debug,
{
    let mut best = None;

    for v in vec {
        let r = lev(
            f(v).to_lowercase().as_str(),
            value.to_lowercase().as_str(),
            threshold,
        );

        best = match best {
            // if the current rank is better change it
            Some(FuzzyRes { rank, .. }) if r >= rank => Some(FuzzyRes { rank: r, data: v }),
            // This case only trigger once and set the starting value
            None if r > 0. => Some(FuzzyRes { rank: r, data: v }),

            // Anythign else keep it as is
            same => same,
        };
    }

    best
}

/// Normalize levenshtein distance.
///
/// <https://github.com/TheAlgorithms/Rust/blob/master/src/string/levenshtein_distance.rs>
pub fn lev(string1: &str, string2: &str, threshold: f32) -> f32 {
    if string1.is_empty() {
        return 0.;
    }

    let l1 = string1.len();
    let mut prev_dist: Vec<usize> = (0..=l1).collect();

    for (row, c2) in string2.chars().enumerate() {
        // we'll keep a reference to matrix[i-1][j-1] (top-left cell)
        let mut prev_substitution_cost = prev_dist[0];
        // diff with empty string, since `row` starts at 0, it's `row + 1`
        prev_dist[0] = row + 1;

        for (col, c1) in string1.chars().enumerate() {
            // "on the left" in the matrix (i.e. the value we just computed)
            let deletion_cost = prev_dist[col] + 1;
            // "on the top" in the matrix (means previous)
            let insertion_cost = prev_dist[col + 1] + 1;
            let substitution_cost = if c1 == c2 {
                // last char is the same on both ends, so the min_distance is left unchanged from matrix[i-1][i+1]
                prev_substitution_cost
            } else {
                // substitute the last character
                prev_substitution_cost + 1
            };
            // save the old value at (i-1, j-1)
            prev_substitution_cost = prev_dist[col + 1];
            prev_dist[col + 1] = min(substitution_cost, min(deletion_cost, insertion_cost));
        }
    }

    let max = max(string1.len(), string2.len());

    // Normalize the distance
    let t = (max - prev_dist[l1]) as f32 / max as f32;

    if t >= threshold {
        t
    } else {
        0.
    }
}
