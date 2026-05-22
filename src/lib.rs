use tauri::Manager;
// src-tauri/src/lib.rs
// Núcleo do app Tauri — AppState, plugins e registro de commands

mod config;
mod supabase;

// FIX #1: removida a linha `mod commands;` que conflitava com o bloco inline abaixo.
// Apenas uma das formas deve existir. Usamos o bloco inline pois não há commands/mod.rs.
pub mod commands {
    pub mod auth;
    pub mod escolas;
    pub mod turmas;
    pub mod pessoas;
    pub mod chamada;
    pub mod cronograma;
    pub mod diario;
}

use commands::{auth, escolas, turmas, pessoas, chamada, cronograma, diario};
use sqlx::postgres::PgPoolOptions;
use supabase::SupabaseAdmin;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub supabase: SupabaseAdmin,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        // FIX #7: tauri-plugin-store removido do Cargo.toml pois não é usado;
        // se precisar no futuro: .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Carrega configuração
            let cfg = match config::AppConfig::load() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("ERRO DE CONFIGURAÇÃO:\n{e}");
                    #[cfg(not(debug_assertions))]
                    {
                        use tauri_plugin_dialog::DialogExt;
                        app.dialog()
                            .message(format!("Configuração necessária:\n\n{e}"))
                            .title("SAG — Configuração")
                            .blocking_show();
                    }
                    std::process::exit(1);
                }
            };

            // Pool de conexões PostgreSQL
            let pool = tauri::async_runtime::block_on(
                PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&cfg.pg_url())
            )
            .expect("Falha ao conectar ao banco de dados. Verifique o config.toml.");

            let supabase = SupabaseAdmin::new(
                cfg.supabase.url.clone(),
                cfg.supabase.service_role_key.clone(),
            );

            app.manage(AppState { db: pool, supabase });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            auth::fazer_login,

            // Escolas
            escolas::get_escolas,
            escolas::criar_escola,
            escolas::atualizar_escola,
            escolas::excluir_escola,

            // Turmas
            turmas::get_turmas,
            turmas::criar_turma,
            turmas::atualizar_turma,
            turmas::excluir_turma,

            // Professores
            pessoas::get_professores,
            pessoas::criar_professor,
            pessoas::atualizar_professor,
            pessoas::excluir_professor,

            // Alunos
            pessoas::get_alunos,
            pessoas::criar_aluno,
            pessoas::atualizar_aluno,
            pessoas::excluir_aluno,

            // Chamada
            chamada::get_chamadas,
            chamada::get_resumo_chamada,
            chamada::chamada_existe,
            chamada::salvar_chamada,
            chamada::get_presencas_chamada,
            chamada::atualizar_presencas,
            chamada::excluir_chamada,

            // Cronograma
            cronograma::get_cronograma,
            cronograma::criar_cronograma,
            cronograma::excluir_cronograma,

            // Diário
            diario::get_diario,
            diario::criar_diario,
            diario::atualizar_diario,
            diario::excluir_diario,

            // Horas
            diario::get_horas,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o SAG Desktop");
}
