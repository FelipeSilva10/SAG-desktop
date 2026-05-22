// src-tauri/src/commands/chamada.rs

use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct Chamada {
    pub id: String,
    #[serde(rename = "professorId")] pub professor_id: String,
    #[serde(rename = "turmaId")]     pub turma_id: String,
    #[serde(rename = "turmaNome")]   pub turma_nome: String,
    #[serde(rename = "cronogramaId")] pub cronograma_id: Option<String>,
    #[serde(rename = "dataAula")]    pub data_aula: String,
    #[serde(rename = "horarioInicio")] pub horario_inicio: String,
    #[serde(rename = "horarioFim")]  pub horario_fim: String,
    #[serde(rename = "totalAlunos")] pub total_alunos: i64,
    #[serde(rename = "totalPresentes")] pub total_presentes: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChamadaPresenca {
    pub id: Option<String>,
    #[serde(rename = "chamadaId")] pub chamada_id: Option<String>,
    #[serde(rename = "alunoId")]   pub aluno_id: String,
    #[serde(rename = "alunoNome")] pub aluno_nome: String,
    pub presente: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ResumoTurma {
    #[serde(rename = "turmaId")]       pub turma_id: String,
    #[serde(rename = "turmaNome")]     pub turma_nome: String,
    #[serde(rename = "escolaNome")]    pub escola_nome: String,
    #[serde(rename = "totalChamadas")] pub total_chamadas: i64,
    #[serde(rename = "ultimaChamada")] pub ultima_chamada: Option<String>,
    #[serde(rename = "mediaPresenca")] pub media_presenca: f64,
}

#[tauri::command]
pub async fn get_chamadas(
    professor_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Chamada>, String> {
    let pid = professor_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let rows = sqlx::query!(
        r#"SELECT c.id::text, c.professor_id::text, c.turma_id::text,
                  t.nome AS turma_nome,
                  c.cronograma_id::text as "cronograma_id?",
                  c.data_aula::text AS data_aula,
                  TO_CHAR(c.horario_inicio,'HH24:MI') AS horario_inicio,
                  TO_CHAR(c.horario_fim,'HH24:MI') AS horario_fim,
                  COUNT(cp.id) AS total_alunos,
                  COUNT(cp.id) FILTER (WHERE cp.presente) AS total_presentes
           FROM chamadas c
           JOIN turmas t ON t.id=c.turma_id
           LEFT JOIN chamada_presencas cp ON cp.chamada_id=c.id
           WHERE c.professor_id=$1
           GROUP BY c.id, t.nome
           ORDER BY c.data_aula DESC, c.horario_inicio"#,
        pid
    )
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| Chamada {
        id: r.id.unwrap_or_default(),
        professor_id: r.professor_id.unwrap_or_default(),
        turma_id: r.turma_id.unwrap_or_default(),
        turma_nome: r.turma_nome,
        cronograma_id: r.cronograma_id,
        data_aula: r.data_aula.unwrap_or_default(),
        horario_inicio: r.horario_inicio.unwrap_or_default(),
        horario_fim: r.horario_fim.unwrap_or_default(),
        total_alunos: r.total_alunos.unwrap_or(0),
        total_presentes: r.total_presentes.unwrap_or(0),
    }).collect())
}

#[tauri::command]
pub async fn get_resumo_chamada(
    professor_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ResumoTurma>, String> {
    let pid = professor_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let rows = sqlx::query!(
        r#"SELECT t.id::text AS turma_id, t.nome AS turma_nome, e.nome AS escola_nome,
                  COUNT(DISTINCT c.id) AS total_chamadas,
                  MAX(c.data_aula::text) AS ultima_chamada,
                  ROUND(
                      100.0 * COUNT(cp.id) FILTER (WHERE cp.presente)::numeric
                      / NULLIF(COUNT(cp.id),0), 1
                  )::float8 AS media_presenca
           FROM turmas t
           JOIN escolas e ON e.id=t.escola_id
           LEFT JOIN chamadas c ON c.turma_id=t.id AND c.professor_id=$1
           LEFT JOIN chamada_presencas cp ON cp.chamada_id=c.id
           WHERE t.professor_id=$1
           GROUP BY t.id, t.nome, e.nome ORDER BY t.nome"#,
        pid
    )
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| ResumoTurma {
        turma_id: r.turma_id.unwrap_or_default(),
        turma_nome: r.turma_nome,
        escola_nome: r.escola_nome,
        total_chamadas: r.total_chamadas.unwrap_or(0),
        ultima_chamada: r.ultima_chamada,
        media_presenca: r.media_presenca.unwrap_or(0.0),
    }).collect())
}

#[tauri::command]
pub async fn chamada_existe(
    professor_id: String,
    turma_id: String,
    data: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let pid = professor_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let tid = turma_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let parsed_data = chrono::NaiveDate::parse_from_str(&data, "%Y-%m-%d").map_err(|e| e.to_string())?;

    let row = sqlx::query!(
        "SELECT 1 as existe FROM chamadas WHERE professor_id=$1 AND turma_id=$2 AND data_aula=$3 LIMIT 1",
        pid, tid, parsed_data
    )
    .fetch_optional(&state.db).await.map_err(|e| e.to_string())?;
    Ok(row.is_some())
}

#[tauri::command]
pub async fn salvar_chamada(
    professor_id: String,
    turma_id: String,
    cronograma_id: Option<String>,
    data_aula: String,
    horario_inicio: String,
    horario_fim: String,
    presencas: Vec<ChamadaPresenca>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let pid = professor_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let tid = turma_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let cid: Option<uuid::Uuid> = cronograma_id
        .as_deref()
        .map(|s| s.parse::<uuid::Uuid>().map_err(|e| e.to_string()))
        .transpose()?;

    let parsed_data = chrono::NaiveDate::parse_from_str(&data_aula, "%Y-%m-%d").map_err(|e| e.to_string())?;
    let parsed_inicio = chrono::NaiveTime::parse_from_str(&horario_inicio, "%H:%M").map_err(|e| e.to_string())?;
    let parsed_fim = chrono::NaiveTime::parse_from_str(&horario_fim, "%H:%M").map_err(|e| e.to_string())?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let chamada = sqlx::query!(
        "INSERT INTO chamadas (professor_id, turma_id, cronograma_id, data_aula, horario_inicio, horario_fim)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        pid, tid, cid, parsed_data, parsed_inicio, parsed_fim
    )
    .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;

    let chamada_id = chamada.id;

    for p in &presencas {
        let aid = p.aluno_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
        sqlx::query!(
            "INSERT INTO chamada_presencas (chamada_id, aluno_id, presente)
             VALUES ($1, $2, $3)
             ON CONFLICT (chamada_id, aluno_id) DO UPDATE SET presente=EXCLUDED.presente",
            chamada_id, aid, p.presente
        )
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_presencas_chamada(
    chamada_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChamadaPresenca>, String> {
    let cid = chamada_id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let rows = sqlx::query!(
        r#"SELECT cp.id::text, cp.chamada_id::text, cp.aluno_id::text,
                  p.nome AS aluno_nome, cp.presente
           FROM chamada_presencas cp JOIN perfis p ON p.id=cp.aluno_id
           WHERE cp.chamada_id=$1 ORDER BY p.nome"#,
        cid
    )
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| ChamadaPresenca {
        id: r.id,
        chamada_id: r.chamada_id,
        aluno_id: r.aluno_id.unwrap_or_default(),
        aluno_nome: r.aluno_nome,
        presente: r.presente,
    }).collect())
}

#[tauri::command]
pub async fn atualizar_presencas(
    presencas: Vec<ChamadaPresenca>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    for p in &presencas {
        if let Some(id) = &p.id {
            let pid = id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
            sqlx::query!(
                "UPDATE chamada_presencas SET presente=$1 WHERE id=$2",
                p.presente, pid
            )
            .execute(&state.db).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn excluir_chamada(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let cid = id.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;
    sqlx::query!("DELETE FROM chamada_presencas WHERE chamada_id=$1", cid)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query!("DELETE FROM chamadas WHERE id=$1", cid)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}