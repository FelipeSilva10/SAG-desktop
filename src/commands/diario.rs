// src-tauri/src/commands/diario.rs

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct DiarioAula {
    pub id: String,
    #[serde(rename = "professorId")] pub professor_id: String,
    #[serde(rename = "turmaId")]     pub turma_id: String,
    #[serde(rename = "turmaNome")]   pub turma_nome: String,
    #[serde(rename = "escolaNome")]  pub escola_nome: String,
    #[serde(rename = "dataAula")]    pub data_aula: String,
    pub titulo: String,
    pub conteudo: String,
    pub observacoes: String,
}

#[tauri::command]
pub async fn get_diario(
    professor_id: String,
    turma_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<DiarioAula>, String> {
    let pid = professor_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let pool = &state.db;

    if let Some(tid) = turma_id {
        let rows = sqlx::query!(
            r#"SELECT d.id::text, d.professor_id::text, d.turma_id::text,
                      t.nome AS turma_nome, e.nome AS escola_nome,
                      d.data_aula::text AS data_aula, d.titulo, d.conteudo, d.observacoes
               FROM diario_aulas d
               JOIN turmas t ON t.id=d.turma_id JOIN escolas e ON e.id=t.escola_id
               WHERE d.professor_id=$1 AND d.turma_id=$2::uuid
               ORDER BY d.data_aula DESC, d.created_at DESC"#,
            pid, tid.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| DiarioAula {
            id: r.id.unwrap_or_default(),
            professor_id: r.professor_id.unwrap_or_default(),
            turma_id: r.turma_id.unwrap_or_default(),
            turma_nome: r.turma_nome,
            escola_nome: r.escola_nome,
            data_aula: r.data_aula.unwrap_or_default(),
            titulo: r.titulo,
            conteudo: r.conteudo,
            observacoes: r.observacoes,
        }).collect())
    } else {
        let rows = sqlx::query!(
            r#"SELECT d.id::text, d.professor_id::text, d.turma_id::text,
                      t.nome AS turma_nome, e.nome AS escola_nome,
                      d.data_aula::text AS data_aula, d.titulo, d.conteudo, d.observacoes
               FROM diario_aulas d
               JOIN turmas t ON t.id=d.turma_id JOIN escolas e ON e.id=t.escola_id
               WHERE d.professor_id=$1
               ORDER BY d.data_aula DESC, d.created_at DESC"#,
            pid
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| DiarioAula {
            id: r.id.unwrap_or_default(),
            professor_id: r.professor_id.unwrap_or_default(),
            turma_id: r.turma_id.unwrap_or_default(),
            turma_nome: r.turma_nome,
            escola_nome: r.escola_nome,
            data_aula: r.data_aula.unwrap_or_default(),
            titulo: r.titulo,
            conteudo: r.conteudo,
            observacoes: r.observacoes,
        }).collect())
    }
}

#[tauri::command]
pub async fn criar_diario(
    professor_id: String,
    turma_id: String,
    data_aula: String,
    titulo: String,
    conteudo: String,
    observacoes: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let parsed_data = chrono::NaiveDate::parse_from_str(&data_aula, "%Y-%m-%d").map_err(|e| e.to_string())?;

    let row = sqlx::query!(
        "INSERT INTO diario_aulas (professor_id, turma_id, data_aula, titulo, conteudo, observacoes)
         VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6) RETURNING id::text",
        professor_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?,
        turma_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?,
        parsed_data, titulo, conteudo, observacoes
    )
    .fetch_one(&state.db).await.map_err(|e| e.to_string())?;
    Ok(row.id.unwrap_or_default())
}

#[tauri::command]
pub async fn atualizar_diario(
    id: String,
    data_aula: String,
    titulo: String,
    conteudo: String,
    observacoes: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let parsed_data = chrono::NaiveDate::parse_from_str(&data_aula, "%Y-%m-%d").map_err(|e| e.to_string())?;

    sqlx::query!(
        "UPDATE diario_aulas SET data_aula=$1, titulo=$2, conteudo=$3, observacoes=$4 WHERE id=$5::uuid",
        parsed_data, titulo, conteudo, observacoes,
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn excluir_diario(id: String, state: State<'_, AppState>) -> Result<(), String> {
    sqlx::query!(
        "DELETE FROM diario_aulas WHERE id=$1::uuid",
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

// ── Horas ─────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct RegistroHoras {
    #[serde(rename = "chamadaId")]     pub chamada_id: String,
    #[serde(rename = "professorId")]   pub professor_id: String,
    #[serde(rename = "professorNome")] pub professor_nome: String,
    #[serde(rename = "turmaId")]       pub turma_id: String,
    #[serde(rename = "turmaNome")]     pub turma_nome: String,
    #[serde(rename = "escolaNome")]    pub escola_nome: String,
    #[serde(rename = "escolaTipo")]    pub escola_tipo: String,
    #[serde(rename = "dataAula")]      pub data_aula: String,
    #[serde(rename = "horarioInicio")] pub horario_inicio: String,
    #[serde(rename = "horarioFim")]    pub horario_fim: String,
    #[serde(rename = "tipoAula")]      pub tipo_aula: String,
    #[serde(rename = "horasMinistradas")] pub horas_ministradas: f64,
    #[serde(rename = "totalAlunos")]   pub total_alunos: i64,
    #[serde(rename = "totalPresentes")] pub total_presentes: i64,
    #[serde(rename = "totalAusentes")] pub total_ausentes: i64,
}

#[tauri::command]
pub async fn get_horas(
    professor_id: Option<String>,
    mes: Option<i32>,
    ano: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<RegistroHoras>, String> {
    // Usa a view v_registro_horas
    let rows = sqlx::query!(
        r#"SELECT chamada_id::text, professor_id::text, professor_nome,
                  turma_id::text, turma_nome, escola_nome,
                  COALESCE(escola_tipo,'PUBLICA') AS escola_tipo,
                  data_aula::text, horario_inicio, horario_fim, tipo_aula,
                  horas_ministradas::float8 AS horas_ministradas,
                  total_alunos, total_presentes, total_ausentes
           FROM v_registro_horas
           WHERE ($1::uuid IS NULL OR professor_id = $1::uuid)
             AND ($2::int IS NULL OR mes = $2::int)
             AND ($3::int IS NULL OR ano = $3::int)
           ORDER BY data_aula DESC, professor_nome, horario_inicio"#,
        professor_id.as_deref().and_then(|s| s.parse::<uuid::Uuid>().ok()),
        mes,
        ano
    )
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| RegistroHoras {
        chamada_id: r.chamada_id.unwrap_or_default(),
        professor_id: r.professor_id.unwrap_or_default(),
        professor_nome: r.professor_nome.unwrap_or_default(),
        turma_id: r.turma_id.unwrap_or_default(),
        turma_nome: r.turma_nome.unwrap_or_default(),
        escola_nome: r.escola_nome.unwrap_or_default(),
        escola_tipo: r.escola_tipo.unwrap_or_else(|| "PUBLICA".into()),
        data_aula: r.data_aula.unwrap_or_default(),
        horario_inicio: r.horario_inicio.unwrap_or_default(),
        horario_fim: r.horario_fim.unwrap_or_default(),
        tipo_aula: r.tipo_aula.unwrap_or_default(),
        horas_ministradas: r.horas_ministradas.unwrap_or(0.0),
        total_alunos: r.total_alunos.unwrap_or(0),
        total_presentes: r.total_presentes.unwrap_or(0),
        total_ausentes: r.total_ausentes.unwrap_or(0),
    }).collect())
}