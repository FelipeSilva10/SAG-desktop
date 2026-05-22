// src-tauri/src/config.rs
// Lê credenciais de ~/.config/sag/config.toml
// Nunca commite o arquivo de config — ele fica só na máquina do professor/admin

use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SupabaseConfig {
    pub url: String,
    pub service_role_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DbConfig,
    pub supabase: SupabaseConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        // Tenta ~/.config/sag/config.toml
        let config_dir = dirs::config_dir()
            .ok_or("Não foi possível encontrar diretório de configuração.")?;
        let config_path = config_dir.join("sag").join("config.toml");

        if !config_path.exists() {
            // Cria exemplo se não existir
            let example_dir = config_path.parent().unwrap();
            fs::create_dir_all(example_dir).map_err(|e| e.to_string())?;
            fs::write(
                &config_path,
                EXAMPLE_CONFIG,
            )
            .map_err(|e| e.to_string())?;
            return Err(format!(
                "Arquivo de configuração criado em:\n{}\n\nEdite-o com as credenciais do banco antes de iniciar o SAG.",
                config_path.display()
            ));
        }

        let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        toml::from_str::<AppConfig>(&content).map_err(|e| format!("Erro no config.toml: {e}"))
    }

    /// Connection string para o sqlx
    pub fn pg_url(&self) -> String {
        let db = &self.database;
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode=require",
            db.user, db.password, db.host, db.port, db.name
        )
    }
}

const EXAMPLE_CONFIG: &str = r#"# SAG Desktop — Configuração de Conexão
# Arquivo: ~/.config/sag/config.toml
# NÃO commite este arquivo no git.

[database]
host     = "db.SUAREF.supabase.co"
port     = 5432
user     = "postgres"
password = "SUA_SENHA_AQUI"
name     = "postgres"

[supabase]
url              = "https://SUAREF.supabase.co"
service_role_key = "eyJhbGci..."
"#;
