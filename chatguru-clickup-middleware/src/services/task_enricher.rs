//! Serviço para enriquecer tarefas no ClickUp
//!
//! Atualiza uma tarefa com os campos categoria_nova, subcategoria_nova e estrelas.

use std::sync::Arc;
use tracing::{info, error};

use super::prompts::AiPromptConfig;
use super::field_validator::ValidatedFieldValues;
use clickup_v2::client::api::CustomFieldValue;

/// Enriquece uma tarefa no ClickUp com os campos validados
pub async fn enrich_task(
    client: &Arc<clickup_v2::client::ClickUpClient>,
    task_id: &str,
    prompt_config: &AiPromptConfig,
    field_values: &ValidatedFieldValues,
) -> Result<(), String> {
    info!("📝 Enriquecendo tarefa {} com campos validados", task_id);

    // Obter IDs dos campos do YAML
    let field_ids = prompt_config.get_field_ids()
        .ok_or("field_ids não encontrados no YAML")?;

    info!(
        "📤 Atualizando tarefa com: categoria_id={}, subcategoria_id={}, stars={}",
        field_values.categoria_id,
        field_values.subcategoria_id,
        field_values.stars
    );

    // Atualizar cada campo individualmente (ClickUp requer isso)

    // 1. Atualizar categoria_nova (dropdown)
    match client.update_custom_field(
        task_id,
        &field_ids.category_field_id,
        CustomFieldValue::DropdownOption(field_values.categoria_id.clone()),
    ).await {
        Ok(_) => {
            info!("✅ Campo categoria_nova atualizado");
        }
        Err(e) => {
            error!("❌ Erro ao atualizar categoria_nova: {}", e);
            return Err(format!("Erro ao atualizar categoria_nova: {}", e));
        }
    }

    // 2. Atualizar subcategoria_nova (dropdown)
    match client.update_custom_field(
        task_id,
        &field_ids.subcategory_field_id,
        CustomFieldValue::DropdownOption(field_values.subcategoria_id.clone()),
    ).await {
        Ok(_) => {
            info!("✅ Campo subcategoria_nova atualizado");
        }
        Err(e) => {
            error!("❌ Erro ao atualizar subcategoria_nova: {}", e);
            return Err(format!("Erro ao atualizar subcategoria_nova: {}", e));
        }
    }

    // 3. Atualizar estrelas (rating)
    match client.update_custom_field(
        task_id,
        &field_ids.stars_field_id,
        CustomFieldValue::Rating(field_values.stars as i32),
    ).await {
        Ok(_) => {
            info!("✅ Campo estrelas atualizado");
        }
        Err(e) => {
            error!("❌ Erro ao atualizar estrelas: {}", e);
            return Err(format!("Erro ao atualizar estrelas: {}", e));
        }
    }

    info!("🎉 Tarefa {} enriquecida com sucesso!", task_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_field_values() {
        // Teste de valores de campos
        let categoria_value = CustomFieldValue::DropdownOption("cat-123".to_string());
        let subcategoria_value = CustomFieldValue::DropdownOption("subcat-456".to_string());
        let stars_value = CustomFieldValue::Rating(3);

        match categoria_value {
            CustomFieldValue::DropdownOption(id) => assert_eq!(id, "cat-123"),
            _ => panic!("Tipo incorreto"),
        }

        match stars_value {
            CustomFieldValue::Rating(rating) => assert_eq!(rating, 3),
            _ => panic!("Tipo incorreto"),
        }
    }
}

