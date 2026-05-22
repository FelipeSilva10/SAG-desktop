// src-tauri/src/commands/pessoas.rs

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct Professor {
    pub id: String,
    pub nome: String,
    pub email: String,
    pub senha: String,
}

#[tauri::command]
pub async fn get_professores(state: State<'_, AppState>) -> Result<Vec<Professor>, String> {
    let rows = sqlx::query!(
        "SELECT id::text, nome, email, senha FROM perfis WHERE role='teacher' ORDER BY nome"
    )
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| Professor {
        id: r.id.unwrap_or_default(),
        nome: r.nome,
        email: r.email.unwrap_or_default(),
        senha: r.senha.unwrap_or_default(),
    }).collect())
}

#[tauri::command]
pub async fn criar_professor(
    nome: String,
    email: String,
    senha: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if senha.len() < 6 { return Err("Senha mínima de 6 caracteres.".into()); }

    let auth_id = state.supabase.criar_usuario(&email, &senha).await?;

    let result = sqlx::query!(
        "INSERT INTO perfis (id, nome, email, senha, role)
         VALUES ($1::uuid, $2, $3, $4, 'teacher')
         ON CONFLICT (id) DO UPDATE SET nome=EXCLUDED.nome, email=EXCLUDED.email, senha=EXCLUDED.senha",
        auth_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?,
        nome.trim(), email.trim(), senha
    )
    .execute(&state.db).await;

    if let Err(e) = result {
        let _ = state.supabase.excluir_usuario(&auth_id).await;
        return Err(e.to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn atualizar_professor(
    id: String,
    nome: String,
    email: String,
    senha: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.supabase
        .atualizar_usuario(&id, Some(email.trim()), Some(&senha))
        .await?;

    sqlx::query!(
        "UPDATE perfis SET nome=$1, email=$2, senha=$3 WHERE id=$4::uuid AND role='teacher'",
        nome.trim(), email.trim(), senha,
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn excluir_professor(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.supabase.excluir_usuario(&id).await?;
    sqlx::query!(
        "DELETE FROM perfis WHERE id=$1::uuid",
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Aluno {
    pub id: String,
    pub nome: String,
    pub email: String,
    pub senha: String,
    #[serde(rename = "turmaId")]
    pub turma_id: String,
    #[serde(rename = "turmaNome")]
    pub turma_nome: String,
    #[serde(rename = "escolaNome")]
    pub escola_nome: String,
}

#[tauri::command]
pub async fn get_alunos(
    professor_id: Option<String>,
    turma_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Aluno>, String> {
    let pool = &state.db;

    let rows = if let Some(tid) = turma_id {
        sqlx::query!(
            r#"SELECT p.id::text, p.nome, p.email, p.senha, p.turma_id::text as "turma_id?",
                      COALESCE(t.nome,'Sem Turma') AS turma_nome,
                      COALESCE(e.nome,'Sem Escola') AS escola_nome
               FROM perfis p
               LEFT JOIN turmas t ON p.turma_id=t.id
               LEFT JOIN escolas e ON t.escola_id=e.id
               WHERE p.role='student' AND p.turma_id=$1::uuid ORDER BY p.nome"#,
            tid.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?
        .into_iter().map(|r| Aluno {
            id: r.id.unwrap_or_default(),
            nome: r.nome,
            email: r.email.unwrap_or_default(),
            senha: r.senha.unwrap_or_default(),
            turma_id: r.turma_id.unwrap_or_default(),
            turma_nome: r.turma_nome.unwrap_or_default(),
            escola_nome: r.escola_nome.unwrap_or_default(),
        }).collect()
    } else if let Some(pid) = professor_id {
        sqlx::query!(
            r#"SELECT p.id::text, p.nome, p.email, p.senha, p.turma_id::text as "turma_id?",
                      COALESCE(t.nome,'Sem Turma') AS turma_nome,
                      COALESCE(e.nome,'Sem Escola') AS escola_nome
               FROM perfis p
               JOIN turmas t ON p.turma_id=t.id
               LEFT JOIN escolas e ON t.escola_id=e.id
               WHERE p.role='student' AND t.professor_id=$1::uuid ORDER BY p.nome"#,
            pid.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?
        .into_iter().map(|r| Aluno {
            id: r.id.unwrap_or_default(),
            nome: r.nome,
            email: r.email.unwrap_or_default(),
            senha: r.senha.unwrap_or_default(),
            turma_id: r.turma_id.unwrap_or_default(),
            turma_nome: r.turma_nome.unwrap_or_default(),
            escola_nome: r.escola_nome.unwrap_or_default(),
        }).collect()
    } else {
        sqlx::query!(
            r#"SELECT p.id::text, p.nome, p.email, p.senha, p.turma_id::text as "turma_id?",
                      COALESCE(t.nome,'Sem Turma') AS turma_nome,
                      COALESCE(e.nome,'Sem Escola') AS escola_nome
               FROM perfis p
               LEFT JOIN turmas t ON p.turma_id=t.id
               LEFT JOIN escolas e ON t.escola_id=e.id
               WHERE p.role='student' ORDER BY p.nome"#
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?
        .into_iter().map(|r| Aluno {
            id: r.id.unwrap_or_default(),
            nome: r.nome,
            email: r.email.unwrap_or_default(),
            senha: r.senha.unwrap_or_default(),
            turma_id: r.turma_id.unwrap_or_default(),
            turma_nome: r.turma_nome.unwrap_or_default(),
            escola_nome: r.escola_nome.unwrap_or_default(),
        }).collect()
    };

    Ok(rows)
}

#[tauri::command]
pub async fn criar_aluno(
    nome: String,
    email: String,
    senha: String,
    turma_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if senha.len() < 6 { return Err("Senha mínima de 6 caracteres.".into()); }

    let auth_id = state.supabase.criar_usuario(&email, &senha).await?;

    let result = sqlx::query!(
        "INSERT INTO perfis (id, nome, email, senha, role, turma_id)
         VALUES ($1::uuid, $2, $3, $4, 'student', $5::uuid)
         ON CONFLICT (id) DO UPDATE SET nome=EXCLUDED.nome, email=EXCLUDED.email,
             senha=EXCLUDED.senha, turma_id=EXCLUDED.turma_id",
        auth_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?,
        nome.trim(), email.trim(), senha,
        turma_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db).await;

    if let Err(e) = result {
        let _ = state.supabase.excluir_usuario(&auth_id).await;
        return Err(e.to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn atualizar_aluno(
    id: String,
    nome: String,
    email: String,
    senha: String,
    turma_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.supabase
        .atualizar_usuario(&id, Some(email.trim()), Some(&senha))
        .await?;

    sqlx::query!(
        "UPDATE perfis SET nome=$1, email=$2, senha=$3, turma_id=$4::uuid WHERE id=$5::uuid",
        nome.trim(), email.trim(), senha,
        turma_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?,
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn excluir_aluno(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.supabase.excluir_usuario(&id).await?;
    sqlx::query!(
        "DELETE FROM perfis WHERE id=$1::uuid",
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}