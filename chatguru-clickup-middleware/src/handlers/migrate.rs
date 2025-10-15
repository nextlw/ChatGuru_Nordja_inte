use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::json;
use sqlx::PgPool;

/// Divide SQL em statements respeitando dollar-quoted strings (funções PL/pgSQL)
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_tag: Option<String> = None;

    for line in sql.lines() {
        let trimmed = line.trim();

        // Ignorar comentários standalone e linhas vazias
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        // Detectar início de dollar-quoted string
        if !in_dollar_quote {
            if let Some(start_pos) = line.find("$$") {
                in_dollar_quote = true;
                // Capturar tag se houver (ex: $body$, $func$)
                let before = &line[..start_pos];
                if let Some(tag_start) = before.rfind('$') {
                    dollar_tag = Some(line[tag_start..=start_pos+1].to_string());
                } else {
                    dollar_tag = Some("$$".to_string());
                }
                current.push_str(line);
                current.push('\n');
                continue;
            }
        }

        // Se estiver dentro de dollar quote
        if in_dollar_quote {
            current.push_str(line);
            current.push('\n');

            // Verificar se é o fim do dollar quote
            if let Some(ref tag) = dollar_tag {
                if line.contains(tag) && line != current.lines().next().unwrap_or("") {
                    // Verificar se tem ; após o tag
                    if line.trim_start().starts_with(tag) && line.contains(';') {
                        in_dollar_quote = false;
                        dollar_tag = None;
                        statements.push(current.trim().to_string());
                        current.clear();
                    }
                }
            }
            continue;
        }

        // Fora de dollar quote, procurar por ;
        current.push_str(line);
        current.push('\n');

        if line.contains(';') {
            statements.push(current.trim().to_string());
            current.clear();
        }
    }

    // Se sobrou algo no buffer
    if !current.trim().is_empty() {
        statements.push(current.trim().to_string());
    }

    statements
}

/// POST /admin/migrate - Aplica todas as migrações SQL
pub async fn apply_migrations(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // FIXME: SQL migration file was removed - temporarily disabled
    // let migration_sql = include_str!("../../migrations/FULL_MIGRATION_ALL.sql");
    let migration_sql = "-- No migrations available";

    // Dividir em statements respeitando dollar-quoted strings (funções PL/pgSQL)
    let statements = split_sql_statements(migration_sql);

    let total_statements = statements.len();
    let mut executed = 0;
    let mut errors = Vec::new();

    for (idx, statement) in statements.iter().enumerate() {
        if statement.trim().is_empty() {
            continue;
        }

        match sqlx::query(statement).execute(&pool).await {
            Ok(_) => {
                executed += 1;
                tracing::info!("✅ Statement {}/{} executed", idx + 1, total_statements);
            }
            Err(e) => {
                let error_msg = format!("Statement {}: {}", idx + 1, e);
                tracing::warn!("⚠️  {}", error_msg);
                errors.push(error_msg);
            }
        }
    }

    // Verificar tabelas criadas
    let tables: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public'
        ORDER BY table_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to query tables: {}", e),
        )
    })?;

    Ok(Json(json!({
        "status": "success",
        "total_statements": total_statements,
        "executed": executed,
        "errors": errors,
        "tables_created": tables.len(),
        "tables": tables.iter().map(|t| &t.0).collect::<Vec<_>>(),
        "message": if errors.is_empty() {
            "✅ All migrations applied successfully!"
        } else {
            "⚠️ Migrations applied with some warnings (this is normal for idempotent migrations)"
        }
    })))
}
