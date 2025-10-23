use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use sqlx::PgPool;

/// Deserialize task_count que pode ser string ou integer
fn deserialize_task_count<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => Ok(n.as_i64().map(|v| v as i32)),
        serde_json::Value::String(s) => Ok(s.parse::<i32>().ok()),
        serde_json::Value::Null => Ok(None),
        _ => Err(D::Error::custom("task_count must be number or string")),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ClickUpSpace {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClickUpFolder {
    id: String,
    name: String,
    hidden: Option<bool>,
    archived: Option<bool>,
    #[serde(deserialize_with = "deserialize_task_count")]
    task_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClickUpList {
    id: String,
    name: String,
    folder: Option<serde_json::Value>,
    archived: Option<bool>,
    #[serde(deserialize_with = "deserialize_task_count")]
    task_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpacesResponse {
    spaces: Vec<ClickUpSpace>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FoldersResponse {
    folders: Vec<ClickUpFolder>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListsResponse {
    lists: Vec<ClickUpList>,
}

/// POST /admin/sync-clickup - Sincroniza spaces, folders e lists do ClickUp para o banco
pub async fn sync_clickup_data(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    tracing::info!("🔄 Iniciando sincronização do ClickUp...");

    // Obter token do ClickUp via Secret Manager
    let clickup_token = std::env::var("CLICKUP_API_TOKEN")
        .or_else(|_| std::env::var("clickup_api_token"))
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ClickUp API token not configured".to_string(),
            )
        })?;

    let client = reqwest::Client::new();

    // 1. Buscar todos os spaces do team Nordja (ID: 9013037641)
    let team_id = "9013037641";
    let spaces_url = format!("https://api.clickup.com/api/v2/team/{}/space?archived=false", team_id);

    let spaces_response = client
        .get(&spaces_url)
        .header("Authorization", &format!("Bearer {}", clickup_token))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to fetch spaces: {}", e)))?;

    let spaces_data: SpacesResponse = spaces_response
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse spaces: {}", e)))?;

    let mut spaces_synced = 0;
    let mut folders_synced = 0;
    let mut lists_synced = 0;

    // 2. Para cada space, inserir/atualizar no banco e buscar folders
    for space in &spaces_data.spaces {
        tracing::info!("📦 Sincronizando space: {} (ID: {})", space.name, space.id);

        // Inserir/atualizar space
        sqlx::query(
            r#"
            INSERT INTO spaces (space_id, space_name, team_id, raw_data, synced_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (space_id)
            DO UPDATE SET
                space_name = EXCLUDED.space_name,
                synced_at = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(&space.id)
        .bind(&space.name)
        .bind(team_id)
        .bind(json!(space))
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to insert space: {}", e),
            )
        })?;

        spaces_synced += 1;

        // 3. Buscar folders do space
        let folders_url = format!("https://api.clickup.com/api/v2/space/{}/folder?archived=false", space.id);

        let folders_response = client
            .get(&folders_url)
            .header("Authorization", &format!("Bearer {}", clickup_token))
            .send()
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to fetch folders: {}", e)))?;

        let folders_data: FoldersResponse = folders_response
            .json()
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse folders: {}", e)))?;

        // 4. Para cada folder, inserir/atualizar e buscar lists
        for folder in &folders_data.folders {
            tracing::info!("📁 Sincronizando folder: {} (ID: {})", folder.name, folder.id);

            let task_count: i32 = folder.task_count.unwrap_or(0);

            sqlx::query(
                r#"
                INSERT INTO folders (folder_id, folder_name, space_id, is_hidden, is_archived, task_count, raw_data, synced_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                ON CONFLICT (folder_id)
                DO UPDATE SET
                    folder_name = EXCLUDED.folder_name,
                    is_hidden = EXCLUDED.is_hidden,
                    is_archived = EXCLUDED.is_archived,
                    task_count = EXCLUDED.task_count,
                    synced_at = NOW(),
                    updated_at = NOW()
                "#,
            )
            .bind(&folder.id)
            .bind(&folder.name)
            .bind(&space.id)
            .bind(folder.hidden.unwrap_or(false))
            .bind(folder.archived.unwrap_or(false))
            .bind(task_count)
            .bind(json!(folder))
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to insert folder: {}", e),
                )
            })?;

            folders_synced += 1;

            // 5. Buscar lists do folder
            let lists_url = format!("https://api.clickup.com/api/v2/folder/{}/list?archived=false", folder.id);

            let lists_response = client
                .get(&lists_url)
                .header("Authorization", &format!("Bearer {}", clickup_token))
                .send()
                .await
                .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to fetch lists: {}", e)))?;

            let lists_data: ListsResponse = lists_response
                .json()
                .await
                .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse lists: {}", e)))?;

            for list in &lists_data.lists {
                tracing::info!("📋 Sincronizando list: {} (ID: {})", list.name, list.id);

                let task_count: i32 = list.task_count.unwrap_or(0);

                sqlx::query(
                    r#"
                    INSERT INTO lists (list_id, list_name, folder_id, space_id, is_archived, task_count, raw_data, synced_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                    ON CONFLICT (list_id)
                    DO UPDATE SET
                        list_name = EXCLUDED.list_name,
                        is_archived = EXCLUDED.is_archived,
                        task_count = EXCLUDED.task_count,
                        synced_at = NOW(),
                        updated_at = NOW()
                    "#,
                )
                .bind(&list.id)
                .bind(&list.name)
                .bind(&folder.id)
                .bind(&space.id)
                .bind(list.archived.unwrap_or(false))
                .bind(task_count)
                .bind(json!(list))
                .execute(&pool)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to insert list: {}", e),
                    )
                })?;

                lists_synced += 1;
            }
        }

        // 6. Também buscar lists "folderless" (sem pasta) do space
        let folderless_url = format!("https://api.clickup.com/api/v2/space/{}/list?archived=false", space.id);

        let folderless_response = client
            .get(&folderless_url)
            .header("Authorization", &format!("Bearer {}", clickup_token))
            .send()
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to fetch folderless lists: {}", e)))?;

        let folderless_data: ListsResponse = folderless_response
            .json()
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to parse folderless lists: {}", e)))?;

        for list in &folderless_data.lists {
            tracing::info!("📋 Sincronizando folderless list: {} (ID: {})", list.name, list.id);

            let task_count: i32 = list.task_count.unwrap_or(0);

            sqlx::query(
                r#"
                INSERT INTO lists (list_id, list_name, folder_id, space_id, is_folderless, is_archived, task_count, raw_data, synced_at)
                VALUES ($1, $2, NULL, $3, true, $4, $5, $6, NOW())
                ON CONFLICT (list_id)
                DO UPDATE SET
                    list_name = EXCLUDED.list_name,
                    is_archived = EXCLUDED.is_archived,
                    task_count = EXCLUDED.task_count,
                    synced_at = NOW(),
                    updated_at = NOW()
                "#,
            )
            .bind(&list.id)
            .bind(&list.name)
            .bind(&space.id)
            .bind(list.archived.unwrap_or(false))
            .bind(task_count)
            .bind(json!(list))
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to insert folderless list: {}", e),
                )
            })?;

            lists_synced += 1;
        }
    }

    tracing::info!(
        "✅ Sincronização concluída: {} spaces, {} folders, {} lists",
        spaces_synced,
        folders_synced,
        lists_synced
    );

    Ok(Json(json!({
        "status": "success",
        "message": "✅ Sincronização do ClickUp concluída com sucesso!",
        "spaces_synced": spaces_synced,
        "folders_synced": folders_synced,
        "lists_synced": lists_synced,
        "total": spaces_synced + folders_synced + lists_synced
    })))
}
