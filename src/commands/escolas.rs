// src-tauri/src/commands/escolas.rs

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct Escola {
    pub id: String,
    pub nome: String,
    pub status: String,
    pub tipo: String,
    #[serde(rename = "tipoLabel")]
    pub tipo_label: String,
}

fn tipo_label(tipo: &str) -> String {
    if tipo == "PRIVADA" { "Privada" } else { "Pública" }.to_string()
}

#[tauri::command]
pub async fn get_escolas(
    professor_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Escola>, String> {
    let pool = &state.db;

    if let Some(pid) = professor_id {
        let rows = sqlx::query!(
            "SELECT DISTINCT e.id::text, e.nome, e.status, COALESCE(e.tipo, 'PUBLICA') AS tipo
             FROM escolas e JOIN turmas t ON t.escola_id = e.id
             WHERE t.professor_id = $1::uuid ORDER BY e.nome",
            pid.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| {
            let tipo = r.tipo.clone().unwrap_or_default();
            Escola {
                id: r.id.unwrap_or_default(),
                nome: r.nome,
                status: r.status,
                tipo_label: tipo_label(&tipo),
                tipo,
            }
        }).collect())
    } else {
        let rows = sqlx::query!(
            "SELECT id::text, nome, status, COALESCE(tipo,'PUBLICA') AS tipo
             FROM escolas ORDER BY nome"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| {
            let tipo = r.tipo.clone().unwrap_or_default();
            Escola {
                id: r.id.unwrap_or_default(),
                nome: r.nome,
                status: r.status,
                tipo_label: tipo_label(&tipo),
                tipo,
            }
        }).collect())
    }
}

#[tauri::command]
pub async fn criar_escola(
    nome: String,
    tipo: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query!(
        "INSERT INTO escolas (nome, status, tipo) VALUES ($1, 'ativo', $2)",
        nome.trim(),
        tipo
    )
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn atualizar_escola(
    id: String,
    nome: String,
    tipo: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query!(
        "UPDATE escolas SET nome=$1, tipo=$2 WHERE id=$3::uuid",
        nome,
        tipo,
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn excluir_escola(id: String, state: State<'_, AppState>) -> Result<(), String> {
    sqlx::query!(
        "DELETE FROM escolas WHERE id=$1::uuid",
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}