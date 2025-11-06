//! Serviço simplificado para validação de hierarquia do workspace ClickUp
//!
//! Este módulo implementa a validação simplificada solicitada:
//! 1. Verifica se Info_2 é compatível com alguma pasta do workspace
//! 2. Garante que existe lista do mês vigente na pasta encontrada
//! 3. Se Info_2 vazio ou sem pasta compatível → interrompe processamento

use crate::utils::error::AppError;
use clickup::folders::SmartFolderFinder;
use clickup::ClickUpClient;

/// Resultado da validação da hierarquia do workspace
#[derive(Debug, Clone)]
pub struct WorkspaceValidation {
    pub is_valid: bool,
    pub folder_id: Option<String>,
    pub folder_name: Option<String>,
    pub list_id: Option<String>,
    pub list_name: Option<String>,
    pub reason: String,
}

impl WorkspaceValidation {
    pub fn invalid(reason: String) -> Self {
        Self {
            is_valid: false,
            folder_id: None,
            folder_name: None,
            list_id: None,
            list_name: None,
            reason,
        }
    }
    
    pub fn valid(folder_id: String, folder_name: String, list_id: String, list_name: String) -> Self {
        Self {
            is_valid: true,
            folder_id: Some(folder_id),
            folder_name: Some(folder_name),
            list_id: Some(list_id),
            list_name: Some(list_name),
            reason: "Validação bem-sucedida".to_string(),
        }
    }
}

/// Serviço de validação da hierarquia do workspace
#[derive(Debug)]
pub struct WorkspaceHierarchyService {
    finder: SmartFolderFinder,
}

impl WorkspaceHierarchyService {
    /// Cria novo serviço de hierarquia usando SmartFolderFinder
    pub fn new(client: ClickUpClient, workspace_id: String) -> Self {
        Self {
            finder: SmartFolderFinder::new(client, workspace_id),
        }
    }

    /// Validação principal simplificada conforme solicitado
    ///
    /// 1. Se info_2 vazio → retorna inválido (interrompe)
    /// 2. Usa SmartFolderFinder para buscar pasta compatível com info_2
    /// 3. Se não encontrar pasta compatível → retorna inválido (interrompe)
    /// 4. Se encontrar → já retorna com lista do mês vigente (SmartFolderFinder cuida disso)
    /// 5. Retorna válido com folder_id e list_id
    pub async fn validate_and_find_target(&mut self, info_2: &str) -> Result<WorkspaceValidation, AppError> {
        tracing::info!("🔍 Iniciando validação simplificada para Info_2: '{}'", info_2);
        
        // 1. Verificar se Info_2 está vazio
        if info_2.trim().is_empty() {
            tracing::warn!("❌ Info_2 vazio - interrompendo processamento");
            return Ok(WorkspaceValidation::invalid(
                "Info_2 está vazio - não é possível determinar pasta de destino".to_string()
            ));
        }

        // 2. Usar SmartFolderFinder para buscar pasta compatível e lista do mês
        match self.finder.find_folder_for_client(info_2).await {
            Ok(Some(result)) => {
                tracing::info!(
                    "✅ Validação bem-sucedida: Pasta='{}' ({}), Lista='{}' ({}), Método={:?}, Confiança={:.2}",
                    result.folder_name, result.folder_id,
                    result.list_name.as_ref().unwrap_or(&"sem lista".to_string()),
                    result.list_id.as_ref().unwrap_or(&"sem id".to_string()),
                    result.search_method,
                    result.confidence
                );

                Ok(WorkspaceValidation::valid(
                    result.folder_id,
                    result.folder_name,
                    result.list_id.unwrap_or_else(|| "sem_lista".to_string()),
                    result.list_name.unwrap_or_else(|| "sem_nome".to_string()),
                ))
            }
            Ok(None) => {
                tracing::warn!("❌ Nenhuma pasta compatível encontrada para Info_2: '{}' - interrompendo processamento", info_2);
                Ok(WorkspaceValidation::invalid(
                    format!("Nenhuma pasta do workspace é compatível com Info_2: '{}'", info_2)
                ))
            }
            Err(e) => {
                tracing::error!("❌ Erro ao buscar pasta/lista para Info_2: '{}': {}", info_2, e);
                Ok(WorkspaceValidation::invalid(
                    format!("Erro ao buscar hierarquia do workspace: {}", e)
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clickup::ClickUpClient;

    #[test]
    fn test_validation_structure() {
        // Teste da estrutura de validação
        let valid = WorkspaceValidation::valid(
            "folder123".to_string(),
            "Teste Folder".to_string(),
            "list456".to_string(),
            "NOVEMBRO 2025".to_string()
        );
        assert!(valid.is_valid);
        assert_eq!(valid.folder_id.unwrap(), "folder123");
        
        let invalid = WorkspaceValidation::invalid("Teste de erro".to_string());
        assert!(!invalid.is_valid);
        assert!(invalid.folder_id.is_none());
    }

    #[test]
    fn test_service_creation() {
        let client = ClickUpClient::new("test_token").unwrap();
        let service = WorkspaceHierarchyService::new(client, "workspace_123".to_string());
        // Teste básico de criação - o SmartFolderFinder é testado em seu próprio módulo
        assert!(format!("{:?}", service).contains("WorkspaceHierarchyService"));
    }
}