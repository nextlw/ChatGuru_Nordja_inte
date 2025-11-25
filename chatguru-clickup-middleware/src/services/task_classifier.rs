//! Serviço para classificar tarefas usando IA
//!
//! Usa o IA Service para analisar o título e descrição de uma tarefa
//! e determinar categoria, subcategoria e estrelas.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};

use super::prompts::AiPromptConfig;
use super::task_fetcher::TaskInfo;

/// Resultado da classificação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub categoria: String,
    pub subcategoria: String,
    pub is_activity: bool,
}

/// Classifica uma tarefa usando IA Service
pub async fn classify_task(
    ia_service: &Arc<ia_service::IaService>,
    prompt_config: &AiPromptConfig,
    task: &TaskInfo,
) -> Result<Classification, String> {
    info!("🤖 Classificando tarefa: {}", task.name);

    // Preparar contexto para a IA
    let context = format!(
        "TÍTULO DA TAREFA: {}\n\nDESCRIÇÃO: {}",
        task.name,
        task.description.as_deref().unwrap_or("Sem descrição")
    );

    // Gerar prompt completo com categorias e subcategorias
    let full_prompt = prompt_config.generate_prompt(&context);

    // Chamar IA Service para classificar
    let result = ia_service
        .classify_activity(&context, &[], &full_prompt)
        .await
        .map_err(|e| format!("Erro na classificação: {}", e))?;

    // Verificar se é uma atividade válida
    if !result.is_activity {
        error!("❌ Tarefa não é uma atividade válida: {}", result.reason);
        // Mesmo assim, tentar usar a categoria se disponível
    }

    // Extrair categoria e subcategoria
    let categoria = result.category
        .clone()
        .ok_or("Categoria não determinada pela IA")?;

    let subcategoria = result.sub_categoria
        .clone()
        .ok_or("Subcategoria não determinada pela IA")?;

    info!(
        "✅ Classificação: categoria='{}', subcategoria='{}', is_activity={}",
        categoria, subcategoria, result.is_activity
    );

    Ok(Classification {
        categoria,
        subcategoria,
        is_activity: result.is_activity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Testes requerem mock do IA Service
    // Por enquanto apenas verificamos a estrutura

    #[test]
    fn test_classification_structure() {
        let classification = Classification {
            categoria: "Plano de Saúde".to_string(),
            subcategoria: "Reembolso Médico".to_string(),
            is_activity: true,
        };

        assert_eq!(classification.categoria, "Plano de Saúde");
        assert_eq!(classification.subcategoria, "Reembolso Médico");
        assert!(classification.is_activity);
    }
}

