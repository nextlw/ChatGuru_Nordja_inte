use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, FromRow};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SpaceInfo {
    pub space_id: String,
    pub space_name: String,
    pub team_id: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FolderInfo {
    pub folder_id: String,
    pub folder_name: String,
    pub space_id: String,
    pub is_active: bool,
    pub task_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ListInfo {
    pub list_id: String,
    pub list_name: String,
    pub folder_id: Option<String>,
    pub space_id: String,
    pub is_folderless: bool,
    pub is_active: bool,
    pub task_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseSummary {
    pub total_spaces: i64,
    pub active_spaces: i64,
    pub total_folders: i64,
    pub active_folders: i64,
    pub total_lists: i64,
    pub active_lists: i64,
}

/// GET /admin/db-check - Verifica o estado do banco de dados
pub async fn check_database(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Query spaces
    let spaces = sqlx::query_as::<_, SpaceInfo>(
        r#"
        SELECT space_id, space_name, team_id, is_active
        FROM spaces
        ORDER BY space_name
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query spaces: {}", e),
        )
    })?;

    // Query folders (top 100)
    let folders = sqlx::query_as::<_, FolderInfo>(
        r#"
        SELECT folder_id, folder_name, space_id, is_active, task_count
        FROM folders
        WHERE is_active = true
        ORDER BY folder_name
        LIMIT 100
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query folders: {}", e),
        )
    })?;

    // Query lists (top 100)
    let lists = sqlx::query_as::<_, ListInfo>(
        r#"
        SELECT list_id, list_name, folder_id, space_id, is_folderless, is_active, task_count
        FROM lists
        WHERE is_active = true
        ORDER BY list_name
        LIMIT 100
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query lists: {}", e),
        )
    })?;

    // Summary counts
    let (total_spaces, active_spaces): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(CASE WHEN is_active = true THEN 1 END) as active
        FROM spaces
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to count spaces: {}", e),
        )
    })?;

    let (total_folders, active_folders): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(CASE WHEN is_active = true THEN 1 END) as active
        FROM folders
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to count folders: {}", e),
        )
    })?;

    let (total_lists, active_lists): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(CASE WHEN is_active = true THEN 1 END) as active
        FROM lists
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to count lists: {}", e),
        )
    })?;

    let summary = DatabaseSummary {
        total_spaces,
        active_spaces,
        total_folders,
        active_folders,
        total_lists,
        active_lists,
    };

    // Expected spaces from ClickUp API (team Nordja - 9013037641)
    let expected_spaces = vec![
        "Clientes Inativos",
        "Georgia",
        "Intl Affairs",
        "Base de Conhecimento",
        "Velma Fortes",
        "Anne Souza",
        "Bruna Senhora",
        "Mariana Medeiros",
        "Mariana Cruz",
        "Nordja",
        "Clientes Esporádicos",
        "Thaís Cotts",
        "Renata Schnoor",
        "Natália Branco",
    ];

    let found_spaces: Vec<String> = spaces.iter().map(|s| s.space_name.clone()).collect();
    let missing_spaces: Vec<String> = expected_spaces
        .iter()
        .filter(|s| !found_spaces.contains(&s.to_string()))
        .map(|s| s.to_string())
        .collect();

    Ok(Json(json!({
        "summary": summary,
        "spaces": {
            "count": spaces.len(),
            "data": spaces,
            "expected": expected_spaces,
            "missing": missing_spaces,
        },
        "folders": {
            "count": folders.len(),
            "total": total_folders,
            "data": folders,
        },
        "lists": {
            "count": lists.len(),
            "total": total_lists,
            "data": lists,
        },
        "status": if missing_spaces.is_empty() && active_spaces > 0 && active_folders > 0 && active_lists > 0 {
            "✅ Database is fully populated with all spaces, folders, and lists"
        } else if missing_spaces.is_empty() && active_spaces > 0 {
            "⚠️ Database has all spaces but may be missing folders or lists"
        } else {
            "❌ Database is missing expected spaces"
        }
    })))
}
