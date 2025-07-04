//! Parser for parsing an input string as a Vietnamese syllable
use crate::{
    maps::{
        ACCUTE_MAP, BREVE_MAP, CIRCUMFLEX_MAP, DOT_MAP, DYET_MAP, GRAVE_MAP, HOOK_ABOVE_MAP,
        HORN_MAP, TILDE_MAP,
    },
    processor::{LetterModification, ToneMark},
    util::is_vowel,
};
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_till, take_while},
    IResult, Parser,
};

/// Components of a Vietnamese syllable parsed from input text.
pub struct SyllableComponents<'a> {
    /// The initial consonant of the syllable (may be empty).
    pub initial_consonant: &'a str,
    /// The vowel part of the syllable.
    pub vowel: &'a str,
    /// The final consonant of the syllable (may be empty).
    pub final_consonant: &'a str,
}

fn initial_consonant(input: &str) -> IResult<&str, &str> {
    if input.to_lowercase().starts_with("gi") && !input.chars().nth(2).is_some_and(is_vowel) {
        return tag_no_case("g").parse(input);
    }
    alt((tag_no_case("gi"), tag_no_case("qu"), take_till(is_vowel))).parse(input)
}

fn vowel(input: &str) -> IResult<&str, &str> {
    take_while(is_vowel).parse(input)
}

/// Parses a vowel from the input string.
///
/// # Errors
/// Returns a parsing error if the input doesn't contain a valid vowel.
pub fn parse_vowel(input: &str) -> IResult<&str, &str> {
    let (rest, _) = initial_consonant(input)?;
    let (rest, vowel) = vowel(rest)?;
    Ok((rest, vowel))
}

/// Parses a complete Vietnamese syllable into its components.
///
/// # Errors
/// Returns a parsing error if the input doesn't contain a valid syllable structure.
pub fn parse_syllable(input: &str) -> IResult<&str, SyllableComponents<'_>> {
    let (rest, initial_consonant) = initial_consonant(input)?;
    let (rest, vowel) = vowel(rest)?;
    Ok((
        rest,
        SyllableComponents {
            initial_consonant,
            vowel,
            final_consonant: rest,
        },
    ))
}

/// Extract letter modifications from an input string.
///
/// Note: In some cases, there might be more than 1 modification. E.g đươc has 3 modifications.
#[must_use]
pub fn extract_letter_modifications(input: &str) -> Vec<(usize, LetterModification)> {
    input
        .chars()
        .enumerate()
        .filter_map(|(index, ch)| {
            if HORN_MAP.values().any(|c| *c == ch) {
                return Some((index, LetterModification::Horn));
            }
            if BREVE_MAP.values().any(|c| *c == ch) {
                return Some((index, LetterModification::Breve));
            }
            if CIRCUMFLEX_MAP.values().any(|c| *c == ch) {
                return Some((index, LetterModification::Circumflex));
            }
            if DYET_MAP.values().any(|c| *c == ch) {
                return Some((index, LetterModification::Dyet));
            }
            None
        })
        .collect()
}

/// Extract a tone mark from an input string. There can only be one tone mark.
#[must_use]
pub fn extract_tone(input: &str) -> Option<ToneMark> {
    for ch in input.chars() {
        let Some(tone_mark) = extract_tone_char(ch) else {
            continue;
        };
        return Some(tone_mark);
    }
    None
}

/// Extract a tone mark from an input char.
#[must_use]
pub fn extract_tone_char(ch: char) -> Option<ToneMark> {
    if ACCUTE_MAP.values().any(|c| *c == ch) {
        return Some(ToneMark::Acute);
    }
    if GRAVE_MAP.values().any(|c| *c == ch) {
        return Some(ToneMark::Grave);
    }
    if HOOK_ABOVE_MAP.values().any(|c| *c == ch) {
        return Some(ToneMark::HookAbove);
    }
    if TILDE_MAP.values().any(|c| *c == ch) {
        return Some(ToneMark::Tilde);
    }
    if DOT_MAP.values().any(|c| *c == ch) {
        return Some(ToneMark::Underdot);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_vowel_normal() {
        let result = parse_vowel("viet");
        let expected = Ok(("t", "ie"));
        assert_eq!(result, expected);
    }

    #[test]
    fn get_vowel_empty() {
        let result = parse_vowel("vt");
        let expected = Ok(("", ""));
        assert_eq!(result, expected);
    }

    #[test]
    fn get_vowel_double_start_tone() {
        let result = parse_vowel("quai");
        let expected = Ok(("", "ai"));
        assert_eq!(result, expected);
    }

    #[test]
    fn get_vowel_double_start_tone_2() {
        let result = parse_vowel("gia");
        let expected = Ok(("", "a"));
        assert_eq!(result, expected);
    }
}
