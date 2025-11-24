/// Utilitários para manipulação segura de strings UTF-8

/// Trunca uma string de forma segura, garantindo que o índice não corte no meio de um caractere UTF-8
///
/// # Argumentos
/// * `s` - String a ser truncada
/// * `max_bytes` - Número máximo de bytes a retornar
///
/// # Retorna
/// Uma string truncada que garante terminar em um limite de caractere válido
///
/// # Exemplo
/// ```
/// use chatguru_clickup_middleware::utils::string_utils::truncate_safe;
///
/// let text = "Olá, mundo! 🌍";
/// let truncated = truncate_safe(text, 10);
/// // Retorna "Olá, mundo" (sem cortar o emoji no meio)
/// ```
pub fn truncate_safe(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    // Encontrar o último byte válido que não corta um caractere UTF-8
    let mut end = max_bytes;

    // Se o byte no índice max_bytes não é o início de um caractere UTF-8,
    // retroceder até encontrar um byte válido
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }

    // Se não encontramos um limite válido, retornar string vazia
    if end == 0 {
        return "";
    }

    &s[..end]
}

/// Trunca uma string e adiciona um sufixo (como "...") de forma segura
///
/// # Argumentos
/// * `s` - String a ser truncada
/// * `max_bytes` - Número máximo de bytes (antes do sufixo)
/// * `suffix` - Sufixo a adicionar (ex: "...")
///
/// # Retorna
/// Uma string truncada com o sufixo, garantindo que não corte no meio de um caractere UTF-8
pub fn truncate_with_suffix(s: &str, max_bytes: usize, suffix: &str) -> String {
    let truncated = truncate_safe(s, max_bytes);
    if truncated.len() < s.len() {
        format!("{}{}", truncated, suffix)
    } else {
        truncated.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_safe_ascii() {
        let text = "Hello, World!";
        assert_eq!(truncate_safe(text, 5), "Hello");
        assert_eq!(truncate_safe(text, 100), text);
    }

    #[test]
    fn test_truncate_safe_utf8() {
        let text = "Olá, mundo!";
        // "Olá" = 3 bytes (O=1, l=1, á=2)
        assert_eq!(truncate_safe(text, 3), "Ol");
        assert_eq!(truncate_safe(text, 4), "Olá");
    }

    #[test]
    fn test_truncate_safe_emoji() {
        let text = "Hello 🌍 World";
        // "Hello 🌍" = 11 bytes (Hello=5, space=1, 🌍=4, space=1)
        let result = truncate_safe(text, 10);
        assert!(result.ends_with("Hello"));
        assert!(!result.contains("🌍")); // Emoji não deve ser cortado no meio
    }

    #[test]
    fn test_truncate_with_suffix() {
        let text = "This is a very long text";
        let result = truncate_with_suffix(text, 10, "...");
        assert_eq!(result, "This is a...");
    }
}

