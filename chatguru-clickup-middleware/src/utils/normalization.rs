//! Funções de normalização para validação simplificada de pastas
//! 
//! Este módulo fornece utilitários para normalizar strings e verificar
//! compatibilidade entre nomes de pastas e valores Info_2 do payload.

use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

/// Remove acentos, converte para lowercase, remove espaços extras e caracteres especiais
/// usando NFKD (Normalization Form Compatibility Decomposition)
/// 
/// # Exemplos
/// ```
/// use chatguru_clickup_middleware::utils::normalization::normalize_string;
/// 
/// assert_eq!(normalize_string("João & Silva Ltda."), "joao silva ltda");
/// assert_eq!(normalize_string("  Anne   Souza  "), "anne souza");
/// assert_eq!(normalize_string("Café & Cia"), "cafe cia");
/// ```
pub fn normalize_string(input: &str) -> String {
    input
        .nfkd() // Normalização NFKD para decomposição completa
        .filter(|c| !is_combining_mark(*c)) // Remove marcas diacríticas (acentos)
        .collect::<String>()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace()) // Mantém apenas alfanuméricos e espaços
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Verifica se duas strings são compatíveis após normalização NFKD
/// 
/// Uma string é considerada compatível com outra se:
/// - Uma contém a outra após normalização
/// - São idênticas após normalização
/// - Possuem pelo menos 70% de similaridade (usando strsim)
/// 
/// # Exemplos
/// ```
/// use chatguru_clickup_middleware::utils::normalization::is_compatible;
/// 
/// assert!(is_compatible("João Silva", "João Silva - Cliente"));
/// assert!(is_compatible("anne", "Anne Souza"));
/// assert!(is_compatible("Café Ltda", "Cafe & Cia Ltda"));
/// assert!(!is_compatible("João", "Maria"));
/// ```
pub fn is_compatible(info_2: &str, folder_name: &str) -> bool {
    let normalized_info = normalize_string(info_2);
    let normalized_folder = normalize_string(folder_name);
    
    // Verificar se estão vazias após normalização
    if normalized_info.is_empty() || normalized_folder.is_empty() {
        return false;
    }
    
    // 1. Verificação de substring (uma contém a outra)
    if normalized_folder.contains(&normalized_info) || normalized_info.contains(&normalized_folder) {
        return true;
    }
    
    // 2. Verificação de similaridade usando strsim (70% de similaridade)
    let similarity = strsim::jaro_winkler(&normalized_info, &normalized_folder);
    if similarity >= 0.7 {
        return true;
    }
    
    // 3. Verificação palavra por palavra (pelo menos 50% das palavras em comum)
    let info_words: Vec<&str> = normalized_info.split_whitespace().collect();
    let folder_words: Vec<&str> = normalized_folder.split_whitespace().collect();
    
    if info_words.is_empty() || folder_words.is_empty() {
        return false;
    }
    
    let common_words = info_words.iter()
        .filter(|word| folder_words.contains(word))
        .count();
    
    let min_words = info_words.len().min(folder_words.len());
    let word_similarity = common_words as f64 / min_words as f64;
    
    word_similarity >= 0.5
}

/// Verifica se uma string representa um mês e ano válidos
/// 
/// Formatos aceitos:
/// - "NOVEMBER 2025"
/// - "November 2025" 
/// - "NOVEMBRO 2025"
/// - "Nov 2025"
/// 
/// # Exemplos
/// ```
/// use chatguru_clickup_middleware::utils::normalization::is_monthly_list;
/// 
/// assert!(is_monthly_list("NOVEMBER 2025"));
/// assert!(is_monthly_list("Novembro 2025"));
/// assert!(!is_monthly_list("Lista Geral"));
/// assert!(!is_monthly_list("Pendencias"));
/// ```
pub fn is_monthly_list(list_name: &str) -> bool {
    let normalized = normalize_string(list_name);
    
    // Lista de nomes de meses em português e inglês (todos normalizados)
    let months = [
        "january", "janeiro", "jan",
        "february", "fevereiro", "feb", "fev",
        "march", "marco", "mar",
        "april", "abril", "apr", "abr",
        "may", "maio", "mai",
        "june", "junho", "jun",
        "july", "julho", "jul",
        "august", "agosto", "aug", "ago",
        "september", "setembro", "sep", "set",
        "october", "outubro", "oct", "out",
        "november", "novembro", "nov",
        "december", "dezembro", "dec", "dez"
    ];
    
    // Verificar se contém um mês e um ano (4 dígitos)
    let contains_month = months.iter().any(|month| normalized.contains(month));
    let contains_year = normalized.chars()
        .collect::<String>()
        .split_whitespace()
        .any(|word| word.len() == 4 && word.chars().all(|c| c.is_ascii_digit()));
    
    contains_month && contains_year
}

/// Gera o nome padrão para lista do mês vigente
/// 
/// Formato: "NOVEMBER 2025"
/// 
/// # Exemplos
/// ```
/// use chrono::{TimeZone, Utc};
/// use chatguru_clickup_middleware::utils::normalization::generate_current_month_name;
/// 
/// // Para novembro de 2025
/// let expected = "NOVEMBER 2025";
/// // assert_eq!(generate_current_month_name(), expected); // Depende da data atual
/// ```
pub fn generate_current_month_name() -> String {
    chrono::Utc::now()
        .format("%B %Y")
        .to_string()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_string_nfkd() {
        // Testes com NFKD para caracteres complexos
        assert_eq!(normalize_string("João & Silva Ltda."), "joao silva ltda");
        assert_eq!(normalize_string("  Anne   Souza  "), "anne souza");
        assert_eq!(normalize_string("Café & Cia"), "cafe cia");
        assert_eq!(normalize_string(""), "");
        assert_eq!(normalize_string("123 ABC"), "123 abc");
        
        // Testes específicos para NFKD
        assert_eq!(normalize_string("Açaí"), "acai");
        assert_eq!(normalize_string("José & Cia"), "jose cia");
        assert_eq!(normalize_string("François"), "francois");
        assert_eq!(normalize_string("Müller & Co"), "muller co");
    }

    #[test]
    fn test_is_compatible() {
        // Casos positivos - substring
        assert!(is_compatible("João Silva", "João Silva - Cliente"));
        assert!(is_compatible("anne", "Anne Souza"));
        assert!(is_compatible("Café Ltda", "Cafe Ltda"));
        
        // Casos positivos - similaridade
        assert!(is_compatible("João", "Joao"));
        assert!(is_compatible("Anne", "Ana"));
        
        // Casos negativos
        assert!(!is_compatible("João", "Maria"));
        assert!(!is_compatible("", "Anne"));
        assert!(!is_compatible("João", ""));
        
        // Casos de palavras em comum
        assert!(is_compatible("Silva Ltda", "João Silva Ltda ME"));
        assert!(is_compatible("Anne Souza", "Anne Cristina Souza"));
        
        // Testes com acentos
        assert!(is_compatible("José", "Jose"));
        assert!(is_compatible("Açaí", "Acai"));
    }

    #[test]
    fn test_is_monthly_list() {
        // Casos positivos
        assert!(is_monthly_list("NOVEMBER 2025"));
        assert!(is_monthly_list("November 2025"));
        assert!(is_monthly_list("NOVEMBRO 2025"));
        assert!(is_monthly_list("Nov 2025"));
        assert!(is_monthly_list("Janeiro 2024"));
        
        // Casos negativos
        assert!(!is_monthly_list("Lista Geral"));
        assert!(!is_monthly_list("Pendencias"));
        assert!(!is_monthly_list("November"));
        assert!(!is_monthly_list("2025"));
        assert!(!is_monthly_list(""));
    }

    #[test]
    fn test_generate_current_month_name() {
        let result = generate_current_month_name();
        // Verificar que tem formato "MONTH YEAR"
        assert!(result.len() > 0);
        assert!(result.contains(" "));
        assert!(result.chars().all(|c| c.is_uppercase() || c.is_ascii_digit() || c.is_whitespace()));
    }
}