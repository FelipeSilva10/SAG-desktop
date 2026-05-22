// src-tauri/src/config.rs
// Lê credenciais de ~/.config/sag/config.toml
// Nunca commite o arquivo de config — ele fica só na máquina do professor/admin

use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
    pub url: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
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
    pub fn config_path() -> Result<PathBuf, String> {
        let config_dir =
            dirs::config_dir().ok_or("Não foi possível encontrar diretório de configuração.")?;
        Ok(config_dir.join("sag").join("config.toml"))
    }

    pub fn load() -> Result<Self, String> {
        // Tenta ~/.config/sag/config.toml
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Cria exemplo se não existir
            let example_dir = config_path.parent().unwrap();
            fs::create_dir_all(example_dir).map_err(|e| e.to_string())?;
            fs::write(&config_path, EXAMPLE_CONFIG).map_err(|e| e.to_string())?;
            return Err(format!(
                "Arquivo de configuração criado em:\n{}\n\nEdite-o com as credenciais do banco antes de iniciar o SAG.",
                config_path.display()
            ));
        }

        let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        toml::from_str::<AppConfig>(&content).map_err(|e| format!("Erro no config.toml: {e}"))
    }

    /// Connection string para o sqlx
    pub fn pg_url(&self) -> Result<String, String> {
        let db = &self.database;
        if let Some(url) = db.url.as_deref().filter(|url| !url.trim().is_empty()) {
            return Ok(url.to_string());
        }

        let host = db
            .host
            .as_deref()
            .ok_or("database.host ausente no config.toml")?;
        let port = db.port.ok_or("database.port ausente no config.toml")?;
        let user = db
            .user
            .as_deref()
            .ok_or("database.user ausente no config.toml")?;
        let password = db
            .password
            .as_deref()
            .ok_or("database.password ausente no config.toml")?;
        let name = db
            .name
            .as_deref()
            .ok_or("database.name ausente no config.toml")?;

        Ok(format!(
            "postgres://{user}:{password}@{host}:{port}/{name}?sslmode=require"
        ))
    }
}

const EXAMPLE_CONFIG: &str = r#"# SAG Desktop — Configuração de Conexão
# Arquivo: ~/.config/sag/config.toml
# NÃO commite este arquivo no git.

[database]
# Em redes sem IPv6, prefira colar aqui a string do Supabase:
# Dashboard > Connect > Transaction pooler ou Session pooler.
# url = "postgresql://postgres.SUAREF:SUA_SENHA@aws-0-REGIAO.pooler.supabase.com:6543/postgres?sslmode=require"

# Conexão direta. No Supabase, costuma exigir IPv6.
host     = "db.SUAREF.supabase.co"
port     = 5432
user     = "postgres"
password = "SUA_SENHA_AQUI"
name     = "postgres"

[supabase]
url              = "https://SUAREF.supabase.co"
service_role_key = "SUA_SERVICE_ROLE_KEY"
"#;
