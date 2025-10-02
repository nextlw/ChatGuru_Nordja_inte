use std::collections::HashMap;
use std::path::Path;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tracing::info;

/// Padrão aprendido serializable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub is_activity: bool,
    pub confidence: f32,
    pub times_seen: u32,
}

/// Classificador inteligente usando TF-IDF e padrões
#[derive(Debug, Clone)]
pub struct SmartClassifier {
    /// Dicionário de termos com pesos (TF-IDF)
    activity_terms: HashMap<String, f32>,
    non_activity_terms: HashMap<String, f32>,

    /// Padrões aprendidos dinamicamente (serializável)
    learned_patterns: HashMap<String, LearnedPattern>,
}

// Carregar termos de arquivo YAML/JSON ao invés de hardcode
static ACTIVITY_TERMS: Lazy<HashMap<String, f32>> = Lazy::new(|| {
    // Carregar de config/activity_terms.yaml
    load_terms_from_config("config/activity_terms.yaml")
        .unwrap_or_else(|_| default_activity_terms())
});

static NON_ACTIVITY_TERMS: Lazy<HashMap<String, f32>> = Lazy::new(|| {
    load_terms_from_config("config/non_activity_terms.yaml")
        .unwrap_or_else(|_| default_non_activity_terms())
});

impl SmartClassifier {
    pub fn new() -> Self {
        let mut classifier = Self {
            activity_terms: ACTIVITY_TERMS.clone(),
            non_activity_terms: NON_ACTIVITY_TERMS.clone(),
            learned_patterns: HashMap::new(),
        };

        // Tentar carregar padrões aprendidos anteriormente
        if let Err(e) = classifier.load_from_file("config/learned_patterns.json") {
            info!("No previous learned patterns found or failed to load: {}", e);
        }

        classifier
    }

    /// Classifica mensagem usando TF-IDF score
    pub fn classify(&self, message: &str) -> (bool, f32) {
        let tokens = self.tokenize(message);

        let activity_score = self.calculate_score(&tokens, &self.activity_terms);
        let non_activity_score = self.calculate_score(&tokens, &self.non_activity_terms);

        let confidence = (activity_score - non_activity_score).abs() /
                        (activity_score + non_activity_score).max(1.0);

        (activity_score > non_activity_score, confidence)
    }

    /// Tokeniza e normaliza mensagem
    fn tokenize(&self, message: &str) -> Vec<String> {
        use rust_stemmers::{Algorithm, Stemmer};

        let stemmer = Stemmer::create(Algorithm::Portuguese);

        message.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 2)  // Remover palavras muito curtas
            .map(|word| stemmer.stem(word).to_string())
            .collect()
    }

    /// Calcula score TF-IDF
    fn calculate_score(&self, tokens: &[String], terms: &HashMap<String, f32>) -> f32 {
        tokens.iter()
            .filter_map(|token| terms.get(token))
            .sum()
    }

    /// Aprende novo padrão dinamicamente
    pub fn learn(&mut self, message: &str, is_activity: bool, confidence: f32) {
        if confidence > 0.8 {  // Só aprender de classificações confiantes
            let key = self.normalize_for_learning(message);

            // Atualizar ou inserir padrão
            self.learned_patterns
                .entry(key)
                .and_modify(|pattern| {
                    pattern.times_seen += 1;
                    // Média ponderada de confiança
                    pattern.confidence = (pattern.confidence + confidence) / 2.0;
                })
                .or_insert(LearnedPattern {
                    is_activity,
                    confidence,
                    times_seen: 1,
                });
        }
    }

    fn normalize_for_learning(&self, message: &str) -> String {
        message.to_lowercase().chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    /// Salva padrões aprendidos em JSON
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        // Criar diretório se não existir
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.learned_patterns)?;
        fs::write(path, json)?;

        info!(
            "✅ Saved {} learned patterns to {}",
            self.learned_patterns.len(),
            path
        );

        Ok(())
    }

    /// Carrega padrões aprendidos de JSON
    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        if !Path::new(path).exists() {
            return Err("File does not exist".into());
        }

        let json = fs::read_to_string(path)?;
        self.learned_patterns = serde_json::from_str(&json)?;

        info!(
            "✅ Loaded {} learned patterns from {}",
            self.learned_patterns.len(),
            path
        );

        Ok(())
    }

    /// Retorna número de padrões aprendidos
    pub fn learned_patterns_count(&self) -> usize {
        self.learned_patterns.len()
    }

    /// Limpa padrões com baixa confiança ou pouco vistos
    pub fn cleanup_weak_patterns(&mut self) {
        let before = self.learned_patterns.len();

        self.learned_patterns.retain(|_, pattern| {
            pattern.confidence > 0.7 && pattern.times_seen >= 2
        });

        let after = self.learned_patterns.len();
        if before > after {
            info!("🧹 Cleaned up {} weak patterns ({} → {})", before - after, before, after);
        }
    }
}

// Funções auxiliares para carregar de config
fn load_terms_from_config(path: &str) -> Result<HashMap<String, f32>, Box<dyn std::error::Error>> {
    use std::fs;
    let content = fs::read_to_string(path)?;
    let terms: HashMap<String, f32> = serde_yaml::from_str(&content)?;
    Ok(terms)
}

fn default_activity_terms() -> HashMap<String, f32> {
    // Fallback caso arquivo não exista
    [
        ("preciso", 10.0),
        ("quero", 9.0),
        ("pedido", 8.5),
        ("comprar", 8.0),
        ("orçamento", 7.5),
        ("urgente", 7.0),
        ("solicit", 6.5),  // Stem de "solicito", "solicitação"
        // ... mais termos
    ].iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn default_non_activity_terms() -> HashMap<String, f32> {
    [
        ("oi", 10.0),
        ("olá", 10.0),
        ("bom", 8.0),  // "bom dia"
        ("obrigad", 9.0),  // Stem de "obrigado", "obrigada"
        ("ok", 7.0),
        ("sim", 6.0),
        // ... mais termos
    ].iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_classification() {
        let classifier = SmartClassifier::new();

        let (is_activity, confidence) = classifier.classify("Preciso de 10 caixas de papel A4");
        assert!(is_activity);
        assert!(confidence > 0.7);
    }

    #[test]
    fn test_non_activity_classification() {
        let classifier = SmartClassifier::new();

        let (is_activity, confidence) = classifier.classify("Oi, bom dia!");
        assert!(!is_activity);
        assert!(confidence > 0.7);
    }
}
