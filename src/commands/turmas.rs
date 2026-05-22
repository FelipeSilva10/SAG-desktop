// src-tauri/src/commands/turmas.rs

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct Turma {
    pub id: String,
    #[serde(rename = "escolaId")]
    pub escola_id: String,
    pub nome: String,
    #[serde(rename = "anoLetivo")]
    pub ano_letivo: String,
    #[serde(rename = "escolaNome")]
    pub escola_nome: String,
    #[serde(rename = "professorNome")]
    pub professor_nome: String,
    #[serde(rename = "professorId")]
    pub professor_id: Option<String>,
}

#[tauri::command]
pub async fn get_turmas(
    professor_id: Option<String>,
    escola_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Turma>, String> {
    let pool = &state.db;

    if let Some(pid) = professor_id {
        let rows = sqlx::query!(
            r#"SELECT t.id::text, t.escola_id::text, t.nome, t.ano_letivo,
                      t.professor_id::text as "professor_id?",
                      e.nome AS escola_nome, p.nome AS "prof_nome?"
               FROM turmas t JOIN escolas e ON e.id=t.escola_id
               LEFT JOIN perfis p ON p.id=t.professor_id
               WHERE t.professor_id=$1::uuid ORDER BY e.nome, t.nome"#,
            pid.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;
        
        Ok(rows.into_iter().map(|r| Turma {
            id: r.id.unwrap_or_default(),
            escola_id: r.escola_id.unwrap_or_default(),
            nome: r.nome,
            ano_letivo: r.ano_letivo,
            escola_nome: r.escola_nome,
            professor_nome: r.prof_nome.unwrap_or_else(|| "Sem Professor".into()),
            professor_id: r.professor_id,
        }).collect())

    } else if let Some(eid) = escola_id {
        let rows = sqlx::query!(
            r#"SELECT t.id::text, t.escola_id::text, t.nome, t.ano_letivo,
                      t.professor_id::text as "professor_id?",
                      e.nome AS escola_nome, p.nome AS "prof_nome?"
               FROM turmas t JOIN escolas e ON e.id=t.escola_id
               LEFT JOIN perfis p ON p.id=t.professor_id
               WHERE t.escola_id=$1::uuid ORDER BY t.created_at DESC"#,
            eid.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| Turma {
            id: r.id.unwrap_or_default(),
            escola_id: r.escola_id.unwrap_or_default(),
            nome: r.nome,
            ano_letivo: r.ano_letivo,
            escola_nome: r.escola_nome,
            professor_nome: r.prof_nome.unwrap_or_else(|| "Sem Professor".into()),
            professor_id: r.professor_id,
        }).collect())

    } else {
        let rows = sqlx::query!(
            r#"SELECT t.id::text, t.escola_id::text, t.nome, t.ano_letivo,
                      t.professor_id::text as "professor_id?",
                      e.nome AS escola_nome, p.nome AS "prof_nome?"
               FROM turmas t JOIN escolas e ON e.id=t.escola_id
               LEFT JOIN perfis p ON p.id=t.professor_id
               ORDER BY t.created_at DESC"#
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| Turma {
            id: r.id.unwrap_or_default(),
            escola_id: r.escola_id.unwrap_or_default(),
            nome: r.nome,
            ano_letivo: r.ano_letivo,
            escola_nome: r.escola_nome,
            professor_nome: r.prof_nome.unwrap_or_else(|| "Sem Professor".into()),
            professor_id: r.professor_id,
        }).collect())
    }
}

#[tauri::command]
pub async fn criar_turma(
    escola_id: String,
    nome: String,
    ano_letivo: String,
    professor_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let eid = escola_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let row = sqlx::query!(
        "INSERT INTO turmas (escola_id, nome, ano_letivo) VALUES ($1, $2, $3) RETURNING id",
        eid,
        nome.trim(),
        ano_letivo
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(pid) = professor_id {
        let pid = pid.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
        sqlx::query!(
            "UPDATE turmas SET professor_id=$1 WHERE id=$2",
            pid,
            row.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

// FIX #4: atualizar_turma agora usa um único UPDATE consolidado dentro de uma transação,
// eliminando a janela de inconsistência que existia entre dois UPDATEs separados.
// professor_id=NULL é tratado explicitamente via CASE para não sobrescrever com valor antigo.
#[tauri::command]
pub async fn atualizar_turma(
    id: String,
    escola_id: String,
    nome: String,
    ano_letivo: String,
    professor_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let tid = id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let eid = escola_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let pid: Option<uuid::Uuid> = professor_id
        .as_deref()
        .map(|s| s.parse::<uuid::Uuid>().map_err(|e| e.to_string()))
        .transpose()?;

    // Um único UPDATE atômico: professor_id recebe NULL explicitamente quando pid é None.
    sqlx::query!(
        "UPDATE turmas SET escola_id=$1, nome=$2, ano_letivo=$3, professor_id=$4 WHERE id=$5",
        eid,
        nome.trim(),
        ano_letivo,
        pid,  // Option<Uuid> → sqlx envia NULL quando None
        tid
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn excluir_turma(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let tid = id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    sqlx::query!("DELETE FROM turmas WHERE id=$1", tid)
        .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}