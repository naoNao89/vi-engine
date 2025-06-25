//! Useful utilities functions that might be helpful for developing a Vietnamese IME.

/// Strip off tone mark & modifications from an input char.
///
/// This function removes Vietnamese diacritics and tone marks from a character,
/// returning the base character while preserving case.
///
/// # Examples
///
/// ```
/// use vi::util::clean_char;
///
/// assert_eq!(clean_char('á'), 'a');
/// assert_eq!(clean_char('Ế'), 'E');
/// assert_eq!(clean_char('ự'), 'u');
/// ```
#[inline]
#[must_use]
pub const fn clean_char(ch: char) -> char {
    match ch {
        // Lowercase a family
        'a' | 'à' | 'ả' | 'ã' | 'á' | 'ạ' | 'ă' | 'ằ' | 'ẳ' | 'ẵ' | 'ắ' | 'ặ' | 'â' | 'ầ' | 'ẩ'
        | 'ẫ' | 'ấ' | 'ậ' => 'a',
        // Uppercase A family
        'A' | 'À' | 'Ả' | 'Ã' | 'Á' | 'Ạ' | 'Ă' | 'Ằ' | 'Ẳ' | 'Ẵ' | 'Ắ' | 'Ặ' | 'Â' | 'Ầ' | 'Ẩ'
        | 'Ẫ' | 'Ấ' | 'Ậ' => 'A',
        // Lowercase d family
        'd' | 'đ' => 'd',
        // Uppercase D family
        'D' | 'Đ' => 'D',
        // Lowercase e family
        'e' | 'è' | 'ẻ' | 'ẽ' | 'é' | 'ẹ' | 'ê' | 'ề' | 'ể' | 'ễ' | 'ế' | 'ệ' => {
            'e'
        }
        // Uppercase E family
        'E' | 'È' | 'Ẻ' | 'Ẽ' | 'É' | 'Ẹ' | 'Ê' | 'Ề' | 'Ể' | 'Ễ' | 'Ế' | 'Ệ' => {
            'E'
        }
        // Lowercase i family
        'i' | 'ì' | 'ỉ' | 'ĩ' | 'í' | 'ị' => 'i',
        // Uppercase I family
        'I' | 'Ì' | 'Ỉ' | 'Ĩ' | 'Í' | 'Ị' => 'I',
        // Lowercase o family
        'o' | 'ò' | 'ỏ' | 'õ' | 'ó' | 'ọ' | 'ô' | 'ồ' | 'ổ' | 'ỗ' | 'ố' | 'ộ' | 'ơ' | 'ờ' | 'ở'
        | 'ỡ' | 'ớ' | 'ợ' => 'o',
        // Uppercase O family
        'O' | 'Ò' | 'Ỏ' | 'Õ' | 'Ó' | 'Ọ' | 'Ô' | 'Ồ' | 'Ổ' | 'Ỗ' | 'Ố' | 'Ộ' | 'Ơ' | 'Ờ' | 'Ở'
        | 'Ỡ' | 'Ớ' | 'Ợ' => 'O',
        // Lowercase u family
        'u' | 'ù' | 'ủ' | 'ũ' | 'ú' | 'ụ' | 'ư' | 'ừ' | 'ử' | 'ữ' | 'ứ' | 'ự' => {
            'u'
        }
        // Uppercase U family
        'U' | 'Ù' | 'Ủ' | 'Ũ' | 'Ú' | 'Ụ' | 'Ư' | 'Ừ' | 'Ử' | 'Ữ' | 'Ứ' | 'Ự' => {
            'U'
        }
        // Lowercase y family
        'y' | 'ỳ' | 'ỷ' | 'ỹ' | 'ý' | 'ỵ' => 'y',
        // Uppercase Y family
        'Y' | 'Ỳ' | 'Ỷ' | 'Ỹ' | 'Ý' | 'Ỵ' => 'Y',
        // Any other character remains unchanged
        _ => ch,
    }
}

/// Strip off tone marks & modifications from an input string.
///
/// This function removes Vietnamese diacritics and tone marks from all characters
/// in a string, returning a new string with base characters while preserving case.
///
/// # Examples
///
/// ```
/// use vi::util::clean_string;
///
/// assert_eq!(clean_string("Tiếng Việt"), "Tieng Viet");
/// assert_eq!(clean_string("Xin chào"), "Xin chao");
/// assert_eq!(clean_string("Hà Nội"), "Ha Noi");
/// ```
#[inline]
pub fn clean_string(input: &str) -> String {
    input.chars().map(clean_char).collect()
}

/// Check if a character is a vowel.
///
/// This function checks if the given character is a Vietnamese vowel,
/// including all variations with tone marks and diacritics.
///
/// # Examples
///
/// ```
/// use vi::util::is_vowel;
///
/// assert!(is_vowel('a'));
/// assert!(is_vowel('ế'));
/// assert!(is_vowel('Ư'));
/// assert!(!is_vowel('b'));
/// ```
#[inline]
#[must_use]
pub const fn is_vowel(c: char) -> bool {
    // For const fn, we need to use a simpler approach
    matches!(
        c,
        'a' | 'à'
            | 'ả'
            | 'ã'
            | 'á'
            | 'ạ'
            | 'ă'
            | 'ằ'
            | 'ẳ'
            | 'ẵ'
            | 'ắ'
            | 'ặ'
            | 'â'
            | 'ầ'
            | 'ẩ'
            | 'ẫ'
            | 'ấ'
            | 'ậ'
            | 'e'
            | 'è'
            | 'ẻ'
            | 'ẽ'
            | 'é'
            | 'ẹ'
            | 'ê'
            | 'ề'
            | 'ể'
            | 'ễ'
            | 'ế'
            | 'ệ'
            | 'i'
            | 'ì'
            | 'ỉ'
            | 'ĩ'
            | 'í'
            | 'ị'
            | 'o'
            | 'ò'
            | 'ỏ'
            | 'õ'
            | 'ó'
            | 'ọ'
            | 'ô'
            | 'ồ'
            | 'ổ'
            | 'ỗ'
            | 'ố'
            | 'ộ'
            | 'ơ'
            | 'ờ'
            | 'ở'
            | 'ỡ'
            | 'ớ'
            | 'ợ'
            | 'u'
            | 'ù'
            | 'ủ'
            | 'ũ'
            | 'ú'
            | 'ụ'
            | 'ư'
            | 'ừ'
            | 'ử'
            | 'ữ'
            | 'ứ'
            | 'ự'
            | 'y'
            | 'ỳ'
            | 'ỷ'
            | 'ỹ'
            | 'ý'
            | 'ỵ'
            | 'A'
            | 'À'
            | 'Ả'
            | 'Ã'
            | 'Á'
            | 'Ạ'
            | 'Ă'
            | 'Ằ'
            | 'Ẳ'
            | 'Ẵ'
            | 'Ắ'
            | 'Ặ'
            | 'Â'
            | 'Ầ'
            | 'Ẩ'
            | 'Ẫ'
            | 'Ấ'
            | 'Ậ'
            | 'E'
            | 'È'
            | 'Ẻ'
            | 'Ẽ'
            | 'É'
            | 'Ẹ'
            | 'Ê'
            | 'Ề'
            | 'Ể'
            | 'Ễ'
            | 'Ế'
            | 'Ệ'
            | 'I'
            | 'Ì'
            | 'Ỉ'
            | 'Ĩ'
            | 'Í'
            | 'Ị'
            | 'O'
            | 'Ò'
            | 'Ỏ'
            | 'Õ'
            | 'Ó'
            | 'Ọ'
            | 'Ô'
            | 'Ồ'
            | 'Ổ'
            | 'Ỗ'
            | 'Ố'
            | 'Ộ'
            | 'Ơ'
            | 'Ờ'
            | 'Ở'
            | 'Ỡ'
            | 'Ớ'
            | 'Ợ'
            | 'U'
            | 'Ù'
            | 'Ủ'
            | 'Ũ'
            | 'Ú'
            | 'Ụ'
            | 'Ư'
            | 'Ừ'
            | 'Ử'
            | 'Ữ'
            | 'Ứ'
            | 'Ự'
            | 'Y'
            | 'Ỳ'
            | 'Ỷ'
            | 'Ỹ'
            | 'Ý'
            | 'Ỵ'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to test character mappings with reduced cognitive complexity
    fn test_char_mappings(test_cases: &[(char, char)]) {
        for &(input, expected) in test_cases {
            assert_eq!(clean_char(input), expected, "Failed for input '{input}'");
        }
    }

    #[test]
    fn test_clean_char_a_family() {
        let test_cases = [
            // Lowercase a family
            ('a', 'a'), ('à', 'a'), ('ả', 'a'), ('ã', 'a'), ('á', 'a'), ('ạ', 'a'),
            ('ă', 'a'), ('ằ', 'a'), ('ẳ', 'a'), ('ẵ', 'a'), ('ắ', 'a'), ('ặ', 'a'),
            ('â', 'a'), ('ầ', 'a'), ('ẩ', 'a'), ('ẫ', 'a'), ('ấ', 'a'), ('ậ', 'a'),
            // Uppercase A family
            ('A', 'A'), ('À', 'A'), ('Ả', 'A'), ('Ã', 'A'), ('Á', 'A'), ('Ạ', 'A'),
            ('Ă', 'A'), ('Ằ', 'A'), ('Ẳ', 'A'), ('Ẵ', 'A'), ('Ắ', 'A'), ('Ặ', 'A'),
            ('Â', 'A'), ('Ầ', 'A'), ('Ẩ', 'A'), ('Ẫ', 'A'), ('Ấ', 'A'), ('Ậ', 'A'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_a_breve_bug_fix() {
        // Specific test for the 'Ă' character bug that was fixed
        // This test ensures the bug doesn't regress
        assert_eq!(
            clean_char('Ă'),
            'A',
            "Latin Capital Letter A with Breve should convert to A"
        );
        assert_eq!(
            clean_char('ă'),
            'a',
            "Latin Small Letter A with Breve should convert to a"
        );
    }

    #[test]
    fn test_clean_char_d_family() {
        let test_cases = [
            ('d', 'd'), ('đ', 'd'),
            ('D', 'D'), ('Đ', 'D'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_e_family() {
        let test_cases = [
            // Lowercase e family
            ('e', 'e'), ('è', 'e'), ('ẻ', 'e'), ('ẽ', 'e'), ('é', 'e'), ('ẹ', 'e'),
            ('ê', 'e'), ('ề', 'e'), ('ể', 'e'), ('ễ', 'e'), ('ế', 'e'), ('ệ', 'e'),
            // Uppercase E family
            ('E', 'E'), ('È', 'E'), ('Ẻ', 'E'), ('Ẽ', 'E'), ('É', 'E'), ('Ẹ', 'E'),
            ('Ê', 'E'), ('Ề', 'E'), ('Ể', 'E'), ('Ễ', 'E'), ('Ế', 'E'), ('Ệ', 'E'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_i_family() {
        let test_cases = [
            // Lowercase i family
            ('i', 'i'), ('ì', 'i'), ('ỉ', 'i'), ('ĩ', 'i'), ('í', 'i'), ('ị', 'i'),
            // Uppercase I family
            ('I', 'I'), ('Ì', 'I'), ('Ỉ', 'I'), ('Ĩ', 'I'), ('Í', 'I'), ('Ị', 'I'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_o_family() {
        let test_cases = [
            // Lowercase o family
            ('o', 'o'), ('ò', 'o'), ('ỏ', 'o'), ('õ', 'o'), ('ó', 'o'), ('ọ', 'o'),
            ('ô', 'o'), ('ồ', 'o'), ('ổ', 'o'), ('ỗ', 'o'), ('ố', 'o'), ('ộ', 'o'),
            ('ơ', 'o'), ('ờ', 'o'), ('ở', 'o'), ('ỡ', 'o'), ('ớ', 'o'), ('ợ', 'o'),
            // Uppercase O family
            ('O', 'O'), ('Ò', 'O'), ('Ỏ', 'O'), ('Õ', 'O'), ('Ó', 'O'), ('Ọ', 'O'),
            ('Ô', 'O'), ('Ồ', 'O'), ('Ổ', 'O'), ('Ỗ', 'O'), ('Ố', 'O'), ('Ộ', 'O'),
            ('Ơ', 'O'), ('Ờ', 'O'), ('Ở', 'O'), ('Ỡ', 'O'), ('Ớ', 'O'), ('Ợ', 'O'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_u_family() {
        let test_cases = [
            // Lowercase u family
            ('u', 'u'), ('ù', 'u'), ('ủ', 'u'), ('ũ', 'u'), ('ú', 'u'), ('ụ', 'u'),
            ('ư', 'u'), ('ừ', 'u'), ('ử', 'u'), ('ữ', 'u'), ('ứ', 'u'), ('ự', 'u'),
            // Uppercase U family
            ('U', 'U'), ('Ù', 'U'), ('Ủ', 'U'), ('Ũ', 'U'), ('Ú', 'U'), ('Ụ', 'U'),
            ('Ư', 'U'), ('Ừ', 'U'), ('Ử', 'U'), ('Ữ', 'U'), ('Ứ', 'U'), ('Ự', 'U'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_y_family() {
        let test_cases = [
            // Lowercase y family
            ('y', 'y'), ('ỳ', 'y'), ('ỷ', 'y'), ('ỹ', 'y'), ('ý', 'y'), ('ỵ', 'y'),
            // Uppercase Y family
            ('Y', 'Y'), ('Ỳ', 'Y'), ('Ỷ', 'Y'), ('Ỹ', 'Y'), ('Ý', 'Y'), ('Ỵ', 'Y'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_non_vietnamese() {
        let test_cases = [
            // Non-Vietnamese characters should remain unchanged
            ('b', 'b'), ('B', 'B'), ('c', 'c'), ('C', 'C'), ('f', 'f'), ('F', 'F'),
            ('g', 'g'), ('G', 'G'), ('h', 'h'), ('H', 'H'), ('j', 'j'), ('J', 'J'),
            ('k', 'k'), ('K', 'K'), ('l', 'l'), ('L', 'L'), ('m', 'm'), ('M', 'M'),
            ('n', 'n'), ('N', 'N'), ('p', 'p'), ('P', 'P'), ('q', 'q'), ('Q', 'Q'),
            ('r', 'r'), ('R', 'R'), ('s', 's'), ('S', 'S'), ('t', 't'), ('T', 'T'),
            ('v', 'v'), ('V', 'V'), ('w', 'w'), ('W', 'W'), ('x', 'x'), ('X', 'X'),
            ('z', 'z'), ('Z', 'Z'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_special_characters() {
        let test_cases = [
            // Special characters and numbers should remain unchanged
            ('0', '0'), ('1', '1'), ('9', '9'), (' ', ' '), ('.', '.'), (',', ','),
            ('!', '!'), ('?', '?'), ('-', '-'), ('_', '_'), ('(', '('), (')', ')'),
        ];
        test_char_mappings(&test_cases);
    }

    #[test]
    fn test_clean_char_const_fn() {
        // Test that the function can be used in const contexts
        const CLEANED_A: char = clean_char('á');
        const CLEANED_E: char = clean_char('Ế');
        const CLEANED_U: char = clean_char('ự');

        assert_eq!(CLEANED_A, 'a');
        assert_eq!(CLEANED_E, 'E');
        assert_eq!(CLEANED_U, 'u');
    }
}
