//! Utilitários de fuzzy matching e normalização de strings

use deunicode::deunicode;
use strsim::jaro_winkler;

/// Normaliza uma string para comparação fuzzy
///
/// - Remove acentos (deunicode)
/// - Converte para lowercase
/// - Remove espaços extras
///
/// # Exemplos
///
/// ```
/// use clickup::matching::normalize;
///
/// assert_eq!(normalize("José da Silva"), "jose da silva");
/// assert_eq!(normalize("  João  "), "joao");
/// assert_eq!(normalize("ANDRÉ"), "andre");
/// ```
pub fn normalize(text: &str) -> String {
    deunicode(text).to_lowercase().trim().to_string()
}

/// Calcula similaridade fuzzy entre duas strings usando Jaro-Winkler
///
/// Retorna um valor entre 0.0 (totalmente diferente) e 1.0 (idêntico)
///
/// # Argumentos
///
/// * `a` - Primeira string
/// * `b` - Segunda string
///
/// # Retorno
///
/// Score de similaridade (0.0 a 1.0)
///
/// # Exemplos
///
/// ```
/// use clickup::matching::similarity;
///
/// assert!(similarity("Gabriel", "Gabriel") > 0.99);
/// assert!(similarity("William", "Willian") > 0.90);
/// assert!(similarity("Anne", "Ana") > 0.80);
/// ```
pub fn similarity(a: &str, b: &str) -> f64 {
    jaro_winkler(&normalize(a), &normalize(b))
}

/// Verifica se duas strings são similares acima de um threshold
///
/// # Argumentos
///
/// * `a` - Primeira string
/// * `b` - Segunda string
/// * `threshold` - Threshold mínimo (0.0 a 1.0)
///
/// # Retorno
///
/// `Some(score)` se similaridade >= threshold, `None` caso contrário
///
/// # Exemplos
///
/// ```
/// use clickup::matching::fuzzy_match;
///
/// assert!(fuzzy_match("Gabriel", "Gabriel", 0.90).is_some());
/// assert!(fuzzy_match("William", "Willian", 0.90).is_some());
/// assert!(fuzzy_match("Anne", "Pedro", 0.90).is_none());
/// ```
pub fn fuzzy_match(a: &str, b: &str, threshold: f64) -> Option<f64> {
    let score = similarity(a, b);
    if score >= threshold {
        Some(score)
    } else {
        None
    }
}

/// Verifica se uma string contém outra (case-insensitive, sem acentos)
///
/// # Exemplos
///
/// ```
/// use clickup::matching::contains;
///
/// assert!(contains("José da Silva", "jose"));
/// assert!(contains("ANDRÉ LUIZ", "andre"));
/// assert!(!contains("Maria", "joão"));
/// ```
pub fn contains(haystack: &str, needle: &str) -> bool {
    normalize(haystack).contains(&normalize(needle))
}

/// Extrai tokens (palavras) de uma string normalizada
///
/// # Exemplos
///
/// ```
/// use clickup::matching::tokenize;
///
/// assert_eq!(tokenize("José da Silva"), vec!["jose", "da", "silva"]);
/// assert_eq!(tokenize("  ANDRÉ  LUIZ  "), vec!["andre", "luiz"]);
/// ```
pub fn tokenize(text: &str) -> Vec<String> {
    normalize(text)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("José da Silva"), "jose da silva");
        assert_eq!(normalize("ANDRÉ"), "andre");
        assert_eq!(normalize("  João  "), "joao");
        assert_eq!(normalize("Ação"), "acao");
    }

    #[test]
    fn test_similarity() {
        // Idênticos
        assert!(similarity("Gabriel", "Gabriel") > 0.99);

        // Muito similares
        assert!(similarity("William", "Willian") > 0.90);
        assert!(similarity("Anne", "Ana") > 0.80);

        // Diferentes
        assert!(similarity("Gabriel", "Pedro") < 0.70);
    }

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("Gabriel", "Gabriel", 0.90).is_some());
        assert!(fuzzy_match("William", "Willian", 0.90).is_some());
        assert!(fuzzy_match("Anne", "Pedro", 0.90).is_none());
    }

    #[test]
    fn test_contains() {
        assert!(contains("José da Silva", "jose"));
        assert!(contains("ANDRÉ LUIZ", "andre"));
        assert!(contains("Maria José", "maria"));
        assert!(!contains("Maria", "joão"));
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize("José da Silva"), vec!["jose", "da", "silva"]);
        assert_eq!(tokenize("  ANDRÉ  LUIZ  "), vec!["andre", "luiz"]);
        assert_eq!(tokenize("Anne"), vec!["anne"]);
    }
}
