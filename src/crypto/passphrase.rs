//! Passphrase strength validation and entropy estimation


/// Passphrase strength assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassphraseStrength {
    /// Too weak - < 40 bits entropy
    TooWeak,
    /// Weak - 40-54 bits entropy
    Weak,
    /// Moderate - 55-69 bits entropy
    Moderate,
    /// Strong - 70-94 bits entropy (5-7 word diceware)
    Strong,
    /// Very Strong - >= 95 bits entropy
    VeryStrong,
}

impl PassphraseStrength {
    /// Convert to human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            PassphraseStrength::TooWeak => "Too Weak (not recommended)",
            PassphraseStrength::Weak => "Weak (minimum acceptable)",
            PassphraseStrength::Moderate => "Moderate (good)",
            PassphraseStrength::Strong => "Strong (excellent)",
            PassphraseStrength::VeryStrong => "Very Strong (outstanding)",
        }
    }

    /// Is this passphrase acceptable for production use?
    pub fn is_acceptable(&self) -> bool {
        !matches!(self, PassphraseStrength::TooWeak)
    }
}

/// Passphrase validation result
#[derive(Debug, Clone)]
pub struct PassphraseValidation {
    /// Estimated entropy in bits
    pub entropy_bits: f64,
    /// Strength assessment
    pub strength: PassphraseStrength,
    /// Number of words (if word-based)
    pub word_count: usize,
    /// Character length
    pub char_length: usize,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

/// Validate a passphrase and assess its strength
///
/// This uses a simplified entropy estimation based on:
/// - Character diversity (lowercase, uppercase, digits, symbols)
/// - Length
/// - Word count (if appears to be word-based)
///
/// # Example
///
/// ```rust,ignore
/// let validation = validate_passphrase("correct horse battery staple mountain river");
/// println!("Strength: {}", validation.strength.description());
/// println!("Entropy: {:.1} bits", validation.entropy_bits);
/// ```
pub fn validate_passphrase(passphrase: &str) -> PassphraseValidation {
    let char_length = passphrase.len();
    let words: Vec<&str> = passphrase.split_whitespace().collect();
    let word_count = words.len();

    // Estimate entropy
    let entropy_bits = estimate_entropy(passphrase, word_count);

    // Determine strength
    // Adjusted thresholds: word-based passphrases with 5-6 words should be "Strong"
    let strength = match entropy_bits {
        e if e < 40.0 => PassphraseStrength::TooWeak,
        e if e < 55.0 => PassphraseStrength::Weak,
        e if e < 70.0 => PassphraseStrength::Moderate,
        e if e < 95.0 => PassphraseStrength::Strong,
        _ => PassphraseStrength::VeryStrong,
    };

    // Generate suggestions
    let mut suggestions = Vec::new();

    if entropy_bits < 55.0 {
        suggestions.push("Consider using more words or characters".to_string());
    }

    if word_count < 4 && char_length < 20 {
        suggestions.push("Use at least 4 random words or 20+ characters".to_string());
    }

    if passphrase.chars().all(|c| c.is_alphanumeric())
        && word_count == 0 {
            suggestions.push("Consider adding symbols for extra security".to_string());
        }

    // Check for common patterns
    if passphrase.to_lowercase().contains("password")
        || passphrase.to_lowercase().contains("123")
        || passphrase.to_lowercase().contains("abc")
    {
        suggestions.push("Avoid common words and patterns".to_string());
    }

    PassphraseValidation {
        entropy_bits,
        strength,
        word_count,
        char_length,
        suggestions,
    }
}

/// Estimate passphrase entropy in bits
///
/// This is a simplified estimation that considers:
/// - Word-based passphrases: ~12.9 bits per word (from EFF wordlist)
/// - Character-based: log2(charset_size) * length
fn estimate_entropy(passphrase: &str, word_count: usize) -> f64 {
    // If it looks like a word-based passphrase (3+ words)
    if word_count >= 3 {
        // EFF long wordlist has 7,776 words (log2(7776) ≈ 12.9 bits per word)
        // We use a slightly conservative estimate
        word_count as f64 * 12.5
    } else {
        // Character-based entropy estimation
        let charset_size = estimate_charset_size(passphrase);
        let length = passphrase.len() as f64;

        (charset_size as f64).log2() * length
    }
}

/// Estimate the character set size used in passphrase
fn estimate_charset_size(passphrase: &str) -> usize {
    let mut charset_size = 0;

    let has_lowercase = passphrase.chars().any(|c| c.is_lowercase());
    let has_uppercase = passphrase.chars().any(|c| c.is_uppercase());
    let has_digit = passphrase.chars().any(|c| c.is_ascii_digit());
    let has_symbol = passphrase.chars().any(|c| !c.is_alphanumeric() && !c.is_whitespace());
    let has_space = passphrase.chars().any(|c| c.is_whitespace());

    if has_lowercase { charset_size += 26; }
    if has_uppercase { charset_size += 26; }
    if has_digit { charset_size += 10; }
    if has_symbol { charset_size += 32; }  // Common symbols
    if has_space { charset_size += 1; }

    charset_size.max(1)  // At least 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_passphrase() {
        let validation = validate_passphrase("password");
        assert!(matches!(validation.strength, PassphraseStrength::TooWeak | PassphraseStrength::Weak));
        assert!(!validation.suggestions.is_empty());
    }

    #[test]
    fn test_strong_word_based_passphrase() {
        let validation = validate_passphrase("correct horse battery staple mountain river");
        assert_eq!(validation.word_count, 6);
        assert!(matches!(
            validation.strength,
            PassphraseStrength::Strong | PassphraseStrength::VeryStrong
        ));
        assert!(validation.entropy_bits > 70.0);
    }

    #[test]
    fn test_moderate_passphrase() {
        let validation = validate_passphrase("MyS3cure!Pass");
        assert!(validation.entropy_bits > 40.0);
    }

    #[test]
    fn test_diceware_style() {
        // 6 words from diceware = ~77.5 bits
        let validation = validate_passphrase("cleft cam synod lacy yr wok");
        assert_eq!(validation.word_count, 6);
        assert!(validation.entropy_bits >= 70.0);
        assert!(validation.strength.is_acceptable());
    }
}
