//! Validador de campos
//!
//! Valida se os valores de categoria e subcategoria existem nas opções
//! definidas no ai_prompt.yaml e retorna os IDs correspondentes.

use tracing::{info, warn};

use super::prompts::AiPromptConfig;

/// Valores validados dos campos com IDs resolvidos
#[derive(Debug, Clone)]
pub struct ValidatedFieldValues {
    pub categoria_id: String,
    pub subcategoria_id: String,
    pub stars: u8,
}

/// Valida categoria e subcategoria e retorna os IDs do YAML
///
/// Validações:
/// 1. Categoria deve existir em category_mappings
/// 2. Subcategoria deve existir em subcategory_mappings[categoria]
/// 3. Estrelas são obtidas do YAML (não do input)
pub fn validate_and_get_field_ids(
    prompt_config: &AiPromptConfig,
    categoria: &str,
    subcategoria: &str,
) -> Result<ValidatedFieldValues, String> {
    info!("🔍 Validando: categoria='{}', subcategoria='{}'", categoria, subcategoria);

    // 1. Validar e obter ID da categoria
    let categoria_id = prompt_config.get_category_id(categoria)
        .ok_or_else(|| {
            let available = prompt_config.get_available_categories();
            format!(
                "Categoria '{}' não existe nas opções. Disponíveis: {:?}",
                categoria, available
            )
        })?;

    info!("✅ Categoria válida: {} → {}", categoria, categoria_id);

    // 2. Validar e obter ID da subcategoria
    let subcategoria_id = prompt_config.get_subcategory_id(categoria, subcategoria)
        .ok_or_else(|| {
            let available = prompt_config.get_subcategories_for_category(categoria);
            format!(
                "Subcategoria '{}' não existe para categoria '{}'. Disponíveis: {:?}",
                subcategoria, categoria, available
            )
        })?;

    info!("✅ Subcategoria válida: {} → {}", subcategoria, subcategoria_id);

    // 3. Obter estrelas da subcategoria (do YAML, não do input)
    let stars = prompt_config.get_subcategory_stars(categoria, subcategoria)
        .unwrap_or_else(|| {
            warn!("⚠️ Estrelas não definidas para subcategoria '{}', usando 1", subcategoria);
            1
        });

    info!("✅ Estrelas: {}", stars);

    Ok(ValidatedFieldValues {
        categoria_id,
        subcategoria_id,
        stars,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Testes requerem AiPromptConfig carregado
    // Por enquanto apenas verificamos a estrutura

    #[test]
    fn test_validated_field_values_structure() {
        let values = ValidatedFieldValues {
            categoria_id: "cat-123".to_string(),
            subcategoria_id: "subcat-456".to_string(),
            stars: 3,
        };

        assert_eq!(values.categoria_id, "cat-123");
        assert_eq!(values.subcategoria_id, "subcat-456");
        assert_eq!(values.stars, 3);
    }
}

