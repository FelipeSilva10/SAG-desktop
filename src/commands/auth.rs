// src-tauri/src/commands/auth.rs

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
pub struct UsuarioSessao {
    pub id: String,
    pub nome: String,
    pub role: String, // "ADMIN" | "TEACHER"
}

#[tauri::command]
pub async fn fazer_login(
    login: String,
    senha: String,
    state: State<'_, AppState>,
) -> Result<UsuarioSessao, String> {
    let pool = &state.db;

    // 1. Tenta admin
    let admin = sqlx::query!(
        "SELECT id, nome FROM backoffice_admins WHERE login = $1 AND senha = $2 LIMIT 1",
        login,
        senha
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Erro DB: {e}"))?;

    if let Some(row) = admin {
        return Ok(UsuarioSessao {
            id: row.id.to_string(),
            nome: row.nome,
            role: "ADMIN".to_string(),
        });
    }

    // 2. Tenta professor
    let prof = sqlx::query!(
        "SELECT id, nome FROM perfis WHERE role='teacher' AND email=$1 AND senha=$2 LIMIT 1",
        login,
        senha
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Erro DB: {e}"))?;

    if let Some(row) = prof {
        return Ok(UsuarioSessao {
            id: row.id.to_string(),
            nome: row.nome,
            role: "TEACHER".to_string(),
        });
    }

    Err("Credenciais inválidas.".to_string())
}
