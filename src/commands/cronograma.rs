// src-tauri/src/commands/cronograma.rs

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct CronogramaAula {
    pub id: String,
    #[serde(rename = "professorId")]   pub professor_id: String,
    #[serde(rename = "professorNome")] pub professor_nome: String,
    #[serde(rename = "turmaId")]       pub turma_id: String,
    #[serde(rename = "turmaNome")]     pub turma_nome: String,
    #[serde(rename = "diaSemana")]     pub dia_semana: String,
    #[serde(rename = "horarioInicio")] pub horario_inicio: String,
    #[serde(rename = "horarioFim")]    pub horario_fim: String,
    pub tipo: String,
    #[serde(rename = "dataInicio")]    pub data_inicio: Option<String>,
    #[serde(rename = "dataFim")]       pub data_fim: Option<String>,
    #[serde(rename = "criadoPor")]     pub criado_por: String,
}

#[tauri::command]
pub async fn get_cronograma(
    professor_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<CronogramaAula>, String> {
    let pool = &state.db;

    if let Some(pid) = professor_id {
        let rows = sqlx::query!(
            r#"SELECT ca.id::text, ca.professor_id::text, pf.nome AS professor_nome,
                      ca.turma_id::text, t.nome AS turma_nome, ca.dia_semana,
                      TO_CHAR(ca.horario_inicio,'HH24:MI') AS horario_inicio,
                      TO_CHAR(ca.horario_fim,'HH24:MI') AS horario_fim,
                      ca.tipo, ca.criado_por,
                      ca.data_inicio::text AS data_inicio,
                      ca.data_fim::text AS data_fim
               FROM cronograma_aulas ca
               JOIN turmas t ON t.id=ca.turma_id
               JOIN perfis pf ON pf.id=ca.professor_id
               WHERE ca.professor_id=$1::uuid
               ORDER BY CASE ca.dia_semana
                 WHEN 'SEGUNDA' THEN 1 WHEN 'TERÇA' THEN 2 WHEN 'QUARTA' THEN 3
                 WHEN 'QUINTA' THEN 4 WHEN 'SEXTA' THEN 5 WHEN 'SÁBADO' THEN 6 END,
               ca.horario_inicio"#,
            pid.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| CronogramaAula {
            id: r.id.unwrap_or_default(),
            professor_id: r.professor_id.unwrap_or_default(),
            professor_nome: r.professor_nome,
            turma_id: r.turma_id.unwrap_or_default(),
            turma_nome: r.turma_nome,
            dia_semana: r.dia_semana,
            horario_inicio: r.horario_inicio.unwrap_or_default(),
            horario_fim: r.horario_fim.unwrap_or_default(),
            tipo: r.tipo,
            data_inicio: r.data_inicio,
            data_fim: r.data_fim,
            criado_por: r.criado_por,
        }).collect())
    } else {
        let rows = sqlx::query!(
            r#"SELECT ca.id::text, ca.professor_id::text, pf.nome AS professor_nome,
                      ca.turma_id::text, t.nome AS turma_nome, ca.dia_semana,
                      TO_CHAR(ca.horario_inicio,'HH24:MI') AS horario_inicio,
                      TO_CHAR(ca.horario_fim,'HH24:MI') AS horario_fim,
                      ca.tipo, ca.criado_por,
                      ca.data_inicio::text AS data_inicio,
                      ca.data_fim::text AS data_fim
               FROM cronograma_aulas ca
               JOIN turmas t ON t.id=ca.turma_id
               JOIN perfis pf ON pf.id=ca.professor_id
               ORDER BY pf.nome, CASE ca.dia_semana
                 WHEN 'SEGUNDA' THEN 1 WHEN 'TERÇA' THEN 2 WHEN 'QUARTA' THEN 3
                 WHEN 'QUINTA' THEN 4 WHEN 'SEXTA' THEN 5 WHEN 'SÁBADO' THEN 6 END,
               ca.horario_inicio"#
        )
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

        Ok(rows.into_iter().map(|r| CronogramaAula {
            id: r.id.unwrap_or_default(),
            professor_id: r.professor_id.unwrap_or_default(),
            professor_nome: r.professor_nome,
            turma_id: r.turma_id.unwrap_or_default(),
            turma_nome: r.turma_nome,
            dia_semana: r.dia_semana,
            horario_inicio: r.horario_inicio.unwrap_or_default(),
            horario_fim: r.horario_fim.unwrap_or_default(),
            tipo: r.tipo,
            data_inicio: r.data_inicio,
            data_fim: r.data_fim,
            criado_por: r.criado_por,
        }).collect())
    }
}

#[tauri::command]
pub async fn criar_cronograma(
    professor_id: String,
    turma_id: String,
    dia_semana: String,
    horario_inicio: String,
    horario_fim: String,
    tipo: String,
    data_inicio: Option<String>,
    data_fim: Option<String>,
    criado_por: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let parsed_inicio = chrono::NaiveTime::parse_from_str(&horario_inicio, "%H:%M").map_err(|e| e.to_string())?;
    let parsed_fim = chrono::NaiveTime::parse_from_str(&horario_fim, "%H:%M").map_err(|e| e.to_string())?;
    
    let parsed_d_inicio = match data_inicio {
        Some(ref d) => Some(chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|e| e.to_string())?),
        None => None,
    };
    let parsed_d_fim = match data_fim {
        Some(ref d) => Some(chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|e| e.to_string())?),
        None => None,
    };

    sqlx::query!(
        "INSERT INTO cronograma_aulas
         (professor_id, turma_id, dia_semana, horario_inicio, horario_fim, tipo, data_inicio, data_fim, criado_por)
         VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7, $8, $9)",
        professor_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?,
        turma_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?,
        dia_semana, parsed_inicio, parsed_fim, tipo,
        parsed_d_inicio, parsed_d_fim, criado_por
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn excluir_cronograma(id: String, state: State<'_, AppState>) -> Result<(), String> {
    sqlx::query!(
        "DELETE FROM cronograma_aulas WHERE id=$1::uuid",
        id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}