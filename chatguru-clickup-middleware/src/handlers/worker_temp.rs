// Estruturas necessárias para a classificação IA
#[derive(Debug, Clone)]
pub struct OrganizationalContext {
    pub folder_id: String,
    pub folder_name: String,
    pub list_id: String,
    pub list_name: String,
}

/// 🏗️ FUNÇÃO AUXILIAR: Busca contexto organizacional para enriquecer classificação IA
///
/// OBJETIVO: Fornecer informações de folder_id e list_id à IA para melhorar a classificação
/// BENEFÍCIO: IA pode considerar a estrutura organizacional ao determinar se é uma task
///
/// PARÂMETROS:
/// - info_2: Cliente identificado via campos personalizados
///
/// RETORNO:
/// - Ok(Some(context)): Contexto organizacional encontrado
/// - Ok(None): Cliente não mapeado, mas sem erro
/// - Err: Erro na busca (não deve interromper o processamento principal)
///
/// IMPLEMENTAÇÃO: Utiliza WorkspaceHierarchyService para busca rápida de estrutura organizacional
pub async fn get_organizational_context_for_ai(info_2: &str) -> Result<Option<OrganizationalContext>, String> {
    use crate::services;
    use crate::utils::logging::{log_info, log_warning};

    // 📋 LOG DE INÍCIO DA BUSCA DE CONTEXTO ORGANIZACIONAL
    log_info(&format!(
        "🔍 INICIANDO BUSCA DE CONTEXTO ORGANIZACIONAL - Cliente: '{}'",
        info_2
    ));

    // Validação de entrada
    if info_2.is_empty() {
        log_warning("⚠️ CONTEXTO ORGANIZACIONAL: Info_2 vazio, retornando contexto nulo");
        return Ok(None);
    }

    // 🔑 OBTENÇÃO DE CREDENCIAIS E CONFIGURAÇÃO
    let secrets_service = match services::SecretManagerService::new().await {
        Ok(service) => service,
        Err(e) => {
            log_warning(&format!(
                "⚠️ CONTEXTO ORGANIZACIONAL: Falha ao inicializar SecretsService: {}",
                e
            ));
            return Ok(None); // Não é erro crítico, retorna contexto nulo
        }
    };

    let api_token = match secrets_service.get_clickup_api_token().await {
        Ok(token) => token,
        Err(e) => {
            log_warning(&format!(
                "⚠️ CONTEXTO ORGANIZACIONAL: Falha ao obter token ClickUp: {}",
                e
            ));
            return Ok(None); // Não é erro crítico, retorna contexto nulo
        }
    };

    let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")
        .or_else(|_| std::env::var("CLICKUP_TEAM_ID"))
        .unwrap_or_else(|_| "9013037641".to_string()); // Default workspace da Nordja

    // 🏗️ INICIALIZAÇÃO DO WORKSPACE HIERARCHY SERVICE
    let clickup_client = match clickup::ClickUpClient::new(api_token.clone()) {
        Ok(client) => client,
        Err(e) => {
            log_warning(&format!(
                "⚠️ CONTEXTO ORGANIZACIONAL: Falha ao criar ClickUpClient: {}",
                e
            ));
            return Ok(None); // Não é erro crítico
        }
    };

    let mut hierarchy_service = services::WorkspaceHierarchyService::new(
        clickup_client,
        workspace_id.clone()
    );

    // 🎯 VALIDAÇÃO E BUSCA DE ESTRUTURA ORGANIZACIONAL
    log_info(&format!(
        "🎯 EXECUTANDO VALIDAÇÃO DE ESTRUTURA para cliente '{}'",
        info_2
    ));

    match hierarchy_service.validate_and_find_target(info_2).await {
        Ok(validation_result) => {
            if validation_result.is_valid
                && validation_result.folder_id.is_some()
                && validation_result.list_id.is_some() {
                
                let context = OrganizationalContext {
                    folder_id: validation_result.folder_id.clone().unwrap(),
                    folder_name: validation_result.folder_name.clone().unwrap_or_else(|| "Pasta Desconhecida".to_string()),
                    list_id: validation_result.list_id.clone().unwrap(),
                    list_name: validation_result.list_name.clone().unwrap_or_else(|| "Lista Desconhecida".to_string()),
                };

                log_info(&format!(
                    "✅ CONTEXTO ORGANIZACIONAL ENCONTRADO - Pasta: '{}' ({}), Lista: '{}' ({})",
                    context.folder_name,
                    context.folder_id,
                    context.list_name,
                    context.list_id
                ));

                Ok(Some(context))
            } else {
                log_info(&format!(
                    "ℹ️ CLIENTE NÃO MAPEADO '{}': {} | Validation: folder={}, list={}",
                    info_2,
                    validation_result.reason,
                    validation_result.folder_id.is_some(),
                    validation_result.list_id.is_some()
                ));
                Ok(None) // Cliente não mapeado, mas não é erro
            }
        },
        Err(e) => {
            log_warning(&format!(
                "⚠️ ERRO NA BUSCA DE CONTEXTO ORGANIZACIONAL para '{}': {}",
                info_2,
                e
            ));
            Ok(None) // Não é erro crítico, retorna contexto nulo para não interromper classificação IA
        }
    }
}