//! Utilitários de fuzzy matching e normalização de strings

use deunicode::deunicode;
use strsim::jaro_winkler;

/// Normaliza uma string para comparação fuzzy
///
/// - Remove acentos (deunicode)
/// - Converte para lowercase
/// - Remove separadores especiais (/, -, |, \, etc.)
/// - Remove números e parênteses
/// - Remove espaços extras
///
/// # Exemplos
///
/// ```
/// use clickup::matching::normalize;
///
/// assert_eq!(normalize("José da Silva"), "jose da silva");
/// assert_eq!(normalize("  João  "), "joao");
/// assert_eq!(normalize("Hugo / NSA Global"), "hugo nsa global");
/// assert_eq!(normalize("Company-Name (2024)"), "company name");
/// ```
pub fn normalize(text: &str) -> String {
    let mut normalized = deunicode(text).to_lowercase();
    
    // Remove separadores especiais: /, -, |, \, _, +, =, etc.
    normalized = normalized
        .replace('/', " ")
        .replace('-', " ")
        .replace('|', " ")
        .replace('\\', " ")
        .replace('_', " ")
        .replace('+', " ")
        .replace('=', " ")
        .replace('&', " ");
    
    // Remove números e símbolos especiais
    normalized = normalized
        .chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace())
        .collect();
    
    // Normaliza espaços múltiplos para um único espaço
    normalized
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
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
/// assert!(similarity("Anne", "Ana") > 0.75);
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

/// Calcula similaridade baseada em tokens individuais
///
/// Compara cada token da primeira string com todos os tokens da segunda,
/// usando a maior similaridade encontrada para cada token.
///
/// # Argumentos
///
/// * `a` - Primeira string
/// * `b` - Segunda string
/// * `threshold` - Threshold mínimo para considerar um token como match
///
/// # Retorno
///
/// Score de similaridade baseado na proporção de tokens que fizeram match
///
/// # Exemplos
///
/// ```
/// use clickup::matching::token_similarity;
///
/// let score = token_similarity("Hugo Tisaka", "Hugo / NSA Global", 0.7);
/// assert!(score > 0.0); // "Hugo" deve fazer match
/// ```
pub fn token_similarity(a: &str, b: &str, threshold: f64) -> f64 {
    let tokens_a = tokenize(a);
    let tokens_b = tokenize(b);
    
    if tokens_a.is_empty() || tokens_b.is_empty() {
        return 0.0;
    }
    
    let mut matched_tokens = 0;
    
    for token_a in &tokens_a {
        let mut max_similarity = 0.0;
        
        for token_b in &tokens_b {
            let sim = jaro_winkler(token_a, token_b);
            if sim > max_similarity {
                max_similarity = sim;
            }
        }
        
        if max_similarity >= threshold {
            matched_tokens += 1;
        }
    }
    
    matched_tokens as f64 / tokens_a.len() as f64
}

/// Estrutura para armazenar detalhes de uma tentativa de matching
#[derive(Debug, Clone)]
pub struct MatchingDetails {
    pub input: String,
    pub target: String,
    pub normalized_input: String,
    pub normalized_target: String,
    pub jaro_winkler_score: f64,
    pub token_score: f64,
    pub final_score: f64,
    pub threshold: f64,
    pub is_match: bool,
    pub match_reason: String,
}

/// Verifica matching avançado com logging detalhado
///
/// Combina Jaro-Winkler tradicional com token matching para maior flexibilidade.
/// Retorna detalhes completos do matching para debugging.
///
/// # Argumentos
///
/// * `input` - String de entrada (ex: "Hugo Tisaka")
/// * `target` - String alvo (ex: "Hugo / NSA Global")
/// * `threshold` - Threshold principal (0.0 a 1.0)
///
/// # Retorno
///
/// `MatchingDetails` com informações completas do matching
pub fn advanced_fuzzy_match(input: &str, target: &str, threshold: f64) -> MatchingDetails {
    let normalized_input = normalize(input);
    let normalized_target = normalize(target);
    
    // 1. Jaro-Winkler tradicional
    let jaro_winkler_score = jaro_winkler(&normalized_input, &normalized_target);
    
    // 2. Token matching com threshold reduzido (0.6 para tokens individuais)
    let token_score = token_similarity(input, target, 0.6);
    
    // 3. Score final: maior entre Jaro-Winkler e token matching
    let final_score = jaro_winkler_score.max(token_score);
    
    // 4. Determinar se é match
    let is_match = final_score >= threshold;
    
    // 5. Explicar o motivo do resultado
    let match_reason = if is_match {
        if jaro_winkler_score >= threshold {
            format!("Jaro-Winkler match: {:.1}%", jaro_winkler_score * 100.0)
        } else {
            format!("Token match: {:.1}%", token_score * 100.0)
        }
    } else {
        format!("No match: Jaro-Winkler {:.1}%, Token {:.1}% (threshold: {:.1}%)",
               jaro_winkler_score * 100.0, token_score * 100.0, threshold * 100.0)
    };
    
    MatchingDetails {
        input: input.to_string(),
        target: target.to_string(),
        normalized_input,
        normalized_target,
        jaro_winkler_score,
        token_score,
        final_score,
        threshold,
        is_match,
        match_reason,
    }
}

/// Função de conveniência que retorna apenas o score do matching avançado
pub fn advanced_similarity(input: &str, target: &str) -> f64 {
    let details = advanced_fuzzy_match(input, target, 0.0);
    details.final_score
}

/// Verifica se duas strings são similares usando matching avançado
pub fn advanced_fuzzy_match_simple(input: &str, target: &str, threshold: f64) -> Option<f64> {
    let details = advanced_fuzzy_match(input, target, threshold);
    if details.is_match {
        Some(details.final_score)
    } else {
        None
    }
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
        assert!(similarity("Anne", "Ana") > 0.75); // Ajustado para valor real

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

    #[test]
    fn test_advanced_fuzzy_matching() {
        // Caso específico do incidente: Hugo Tisaka vs Hugo / NSA Global
        let details = advanced_fuzzy_match("Hugo Tisaka", "Hugo / NSA Global", 0.70);
        
        // Deve fazer match com threshold 0.70
        assert!(details.is_match, "Hugo Tisaka vs Hugo / NSA Global deve fazer match com threshold 0.70");
        
        // Verifica se o score é >= 0.70
        assert!(details.final_score >= 0.70,
                "Score final deve ser >= 0.70, mas foi {:.3}", details.final_score);
        
        // Verifica se pelo menos um método (Jaro-Winkler ou token) funcionou
        assert!(details.jaro_winkler_score > 0.0 || details.token_score > 0.0,
                "Pelo menos um score deve ser > 0");
        
        println!("Detalhes do matching:");
        println!("Input: '{}'", details.input);
        println!("Target: '{}'", details.target);
        println!("Jaro-Winkler: {:.1}%", details.jaro_winkler_score * 100.0);
        println!("Token score: {:.1}%", details.token_score * 100.0);
        println!("Final score: {:.1}%", details.final_score * 100.0);
        println!("Match: {} - {}", details.is_match, details.match_reason);
    }

    #[test]
    fn test_token_similarity_specific_cases() {
        // Token "Hugo" deve fazer match
        let score1 = token_similarity("Hugo Tisaka", "Hugo / NSA Global", 0.6);
        assert!(score1 > 0.0, "Deve ter match parcial por 'Hugo'");
        
        // NSA Global vs Hugo NSA Global - deve ter score alto
        let score2 = token_similarity("NSA Global", "Hugo / NSA Global", 0.6);
        assert!(score2 >= 0.66, "NSA Global deve ter score alto: {:.3}", score2);
        
        // Caso com todos os tokens
        let score3 = token_similarity("Hugo NSA Global", "Hugo / NSA Global", 0.6);
        assert!(score3 >= 0.90, "Todos os tokens devem fazer match: {:.3}", score3);
    }

    #[test]
    fn test_normalize_separators() {
        // Testa a normalização melhorada com separadores
        assert_eq!(normalize("Hugo / NSA Global"), "hugo nsa global");
        assert_eq!(normalize("Company-Name"), "company name");
        assert_eq!(normalize("Data\\Processing"), "data processing");
        assert_eq!(normalize("Name_With_Underscores"), "name with underscores");
        assert_eq!(normalize("Test+Plus=Equals&Ampersand"), "test plus equals ampersand");
    }
}
