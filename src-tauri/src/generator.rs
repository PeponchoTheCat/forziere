use rand::Rng;
use rand::seq::SliceRandom;
use serde::Deserialize;

const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()-_=+[]{};:,.<>?/~";

// Codici di errore: il frontend li traduce (vedi `src/i18n/errors.js`).
pub const ERR_NO_CLASS: &str = "no_char_class";

pub const MIN_LENGTH: usize = 4;
pub const MAX_LENGTH: usize = 128;

#[derive(Debug, Deserialize)]
pub struct GenOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
}

/// Genera una password con almeno un carattere per ogni classe selezionata.
pub fn generate(opts: &GenOptions) -> Result<String, String> {
    let classes: Vec<&str> = [
        (opts.uppercase, UPPER),
        (opts.lowercase, LOWER),
        (opts.digits, DIGITS),
        (opts.symbols, SYMBOLS),
    ]
    .into_iter()
    .filter_map(|(on, set)| on.then_some(set))
    .collect();

    if classes.is_empty() {
        return Err(ERR_NO_CLASS.into());
    }
    if !(MIN_LENGTH..=MAX_LENGTH).contains(&opts.length) {
        return Err(format!("invalid_length:{MIN_LENGTH}:{MAX_LENGTH}"));
    }

    // `rand::rng()` è un CSPRNG inizializzato dal sistema operativo.
    let mut rng = rand::rng();
    let pool: Vec<char> = classes.iter().flat_map(|c| c.chars()).collect();

    let mut chars: Vec<char> = classes
        .iter()
        .map(|c| {
            let set: Vec<char> = c.chars().collect();
            set[rng.random_range(0..set.len())]
        })
        .collect();
    while chars.len() < opts.length {
        chars.push(pool[rng.random_range(0..pool.len())]);
    }
    chars.shuffle(&mut rng);
    Ok(chars.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(length: usize, u: bool, l: bool, d: bool, s: bool) -> GenOptions {
        GenOptions { length, uppercase: u, lowercase: l, digits: d, symbols: s }
    }

    #[test]
    fn generate_returns_requested_length_at_bounds_and_middle() {
        for len in [MIN_LENGTH, 16, MAX_LENGTH] {
            // Arrange
            let options = opts(len, true, true, true, true);

            // Act
            let password = generate(&options).unwrap();

            // Assert
            assert_eq!(password.chars().count(), len);
        }
    }

    #[test]
    fn generate_includes_every_selected_class_even_at_minimum_length() {
        // Arrange
        let options = opts(MIN_LENGTH, true, true, true, true);

        for _ in 0..200 {
            // Act
            let password = generate(&options).unwrap();

            // Assert
            assert!(password.chars().any(|c| UPPER.contains(c)));
            assert!(password.chars().any(|c| LOWER.contains(c)));
            assert!(password.chars().any(|c| DIGITS.contains(c)));
            assert!(password.chars().any(|c| SYMBOLS.contains(c)));
        }
    }

    #[test]
    fn generate_uses_only_lowercase_and_digits_when_selected() {
        // Arrange
        let options = opts(32, false, true, true, false);

        for _ in 0..100 {
            // Act
            let password = generate(&options).unwrap();

            // Assert
            assert!(password.chars().all(|c| LOWER.contains(c) || DIGITS.contains(c)));
        }
    }

    #[test]
    fn generate_uses_only_uppercase_when_only_uppercase_selected() {
        // Arrange
        let options = opts(20, true, false, false, false);

        // Act
        let password = generate(&options).unwrap();

        // Assert
        assert!(password.chars().all(|c| UPPER.contains(c)));
    }

    #[test]
    fn generate_uses_only_symbols_when_only_symbols_selected() {
        // Arrange
        let options = opts(20, false, false, false, true);

        // Act
        let password = generate(&options).unwrap();

        // Assert
        assert!(password.chars().all(|c| SYMBOLS.contains(c)));
    }

    #[test]
    fn generate_produces_different_passwords_on_consecutive_calls() {
        // Arrange
        let options = opts(32, true, true, true, true);

        // Act
        let first = generate(&options).unwrap();
        let second = generate(&options).unwrap();

        // Assert
        assert_ne!(first, second);
    }

    #[test]
    fn generate_fails_when_no_class_is_selected() {
        // Arrange
        let options = opts(16, false, false, false, false);

        // Act
        let result = generate(&options);

        // Assert
        assert_eq!(result.unwrap_err(), ERR_NO_CLASS);
    }

    #[test]
    fn generate_fails_when_length_is_below_minimum() {
        // Arrange
        let options = opts(MIN_LENGTH - 1, true, true, true, true);

        // Act
        let result = generate(&options);

        // Assert
        assert_eq!(result.unwrap_err(), "invalid_length:4:128");
    }

    #[test]
    fn generate_fails_when_length_is_above_maximum() {
        // Arrange
        let options = opts(MAX_LENGTH + 1, true, true, true, true);

        // Act
        let result = generate(&options);

        // Assert
        assert_eq!(result.unwrap_err(), "invalid_length:4:128");
    }

    #[test]
    fn gen_options_deserializes_from_frontend_json() {
        // Arrange
        let json = r#"{"length":12,"uppercase":true,"lowercase":false,"digits":true,"symbols":false}"#;

        // Act
        let options: GenOptions = serde_json::from_str(json).unwrap();

        // Assert
        assert_eq!(options.length, 12);
        assert!(options.uppercase && !options.lowercase && options.digits && !options.symbols);
    }
}
