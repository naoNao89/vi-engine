//! Validation functions for verifying if a syllable is a valid vietnamese syllable.
//!
//! # The structure of a vietnamese syllable
//!
//! 1 optional consonant / consonant cluster + 1 compulsory vowel / vowel cluster + 1 optional consonant / consonant cluster
//!
//! The starting consonant are called initial consonant, while the consonant at the end is called the final consonant.
//! A cluster of consonant can contains 1 -> 3 characters.
//! See: <https://en.wikibooks.org/wiki/Vietnamese/Consonants>

use phf::{phf_set, Set};

use crate::{parsing::parse_syllable, util::clean_char};

const SINGLE_INITIAL_CONSONANTS: Set<char> =
    phf_set!['b', 'c', 'd', 'đ', 'g', 'h', 'k', 'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'x',];

const DIGRAPHS_INITIAL_CONSONANTS: Set<&'static str> =
    phf_set!["ch", "gh", "gi", "kh", "nh", "ng", "ph", "th", "tr", "qu"];

const FINAL_CONSONANTS: Set<&'static str> = phf_set!["c", "ch", "m", "n", "nh", "ng", "p", "t"];

const VOWELS: Set<&'static str> = phf_set![
    "ia", "ai", "ieu", "io", "ua", "ao", "au", "oi", "a", "i", "o", "e", "u", "oai", "uou", "uo",
    "uu", "ie", "ay", "oa", "eo", "oeo", "iu", "oao", "oay", "oe", "oo", "ui", "uy", "uya", "uyu",
    "uye", "uoi", "ye", "yeu", "y", "eu", "ue", "uay"
];

/// Verify if a syllable is a valid vietnamese syllable.
#[must_use]
pub fn is_valid_syllable(input: &str) -> bool {
    let Ok((_, components)) = parse_syllable(input) else {
        return false;
    };

    if !components.initial_consonant.is_empty()
        && !is_valid_initial_consonant(components.initial_consonant)
    {
        return false;
    }

    if components.vowel.is_empty() {
        return true;
    }

    let cleaned_vowel: String = components
        .vowel
        .chars()
        .map(|c| clean_char(c).to_ascii_lowercase())
        .collect();
    if !VOWELS.contains(cleaned_vowel.as_str()) {
        return false;
    }

    if !components.final_consonant.is_empty()
        && !is_valid_final_consonant(components.final_consonant)
    {
        return false;
    }

    true
}

/// Checks if the given string is a valid Vietnamese initial consonant.
#[must_use]
pub fn is_valid_initial_consonant(consonant: &str) -> bool {
    let consonant = consonant.to_lowercase();
    let consonant_length = consonant.chars().count();
    if consonant_length == 1 {
        if let Some(c) = consonant.chars().next() {
            return SINGLE_INITIAL_CONSONANTS.contains(&c);
        }
    }

    if consonant_length == 2 {
        return DIGRAPHS_INITIAL_CONSONANTS.contains(consonant.as_str());
    }

    if consonant_length == 3 {
        return consonant == "ngh";
    }

    false
}

/// Checks if the given string is a valid Vietnamese final consonant.
#[must_use]
pub fn is_valid_final_consonant(consonant: &str) -> bool {
    let consonant = consonant.to_lowercase();
    FINAL_CONSONANTS.contains(consonant.as_str())
}
