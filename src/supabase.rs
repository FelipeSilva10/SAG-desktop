// src-tauri/src/supabase.rs
// Chama a Admin Auth API do Supabase via HTTP (reqwest)

use reqwest::Client;
use serde_json::{json, Value};

pub struct SupabaseAdmin {
    pub client: Client,
    pub base_url: String,
    pub service_role_key: String,
}

impl SupabaseAdmin {
    pub fn new(base_url: String, service_role_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            service_role_key,
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.service_role_key)
    }

    /// Cria usuário no Supabase Auth e retorna o UUID gerado.
    pub async fn criar_usuario(&self, email: &str, password: &str) -> Result<String, String> {
        let url = format!("{}/auth/v1/admin/users", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("apikey", &self.service_role_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&json!({
                "email": email,
                "password": password,
                "email_confirm": true
            }))
            .send()
            .await
            .map_err(|e| format!("Erro de rede: {e}"))?;

        if resp.status().is_success() {
            let body: Value = resp.json().await.map_err(|e| e.to_string())?;
            let id = body["id"]
                .as_str()
                .ok_or("Resposta do Supabase sem campo 'id'")?
                .to_string();
            Ok(id)
        } else {
            let body: Value = resp.json().await.unwrap_or(json!({}));
            let msg = body["msg"]
                .as_str()
                .or(body["message"].as_str())
                .unwrap_or("Erro desconhecido do Supabase Auth");
            Err(msg.to_string())
        }
    }

    // Fix #3: método ausente — atualiza email e/ou senha no Supabase Auth.
    // Aceita Option para cada campo; apenas os Some(_) são enviados.
    pub async fn atualizar_usuario(
        &self,
        user_id: &str,
        email: Option<&str>,
        password: Option<&str>,
    ) -> Result<(), String> {
        let url = format!("{}/auth/v1/admin/users/{}", self.base_url, user_id);
        let mut body = serde_json::Map::new();
        if let Some(e) = email {
            body.insert("email".into(), json!(e));
        }
        if let Some(p) = password {
            body.insert("password".into(), json!(p));
        }
        // Nada a atualizar no Auth → retorna Ok imediatamente
        if body.is_empty() {
            return Ok(());
        }

        let resp = self
            .client
            .put(&url)
            .header("apikey", &self.service_role_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&Value::Object(body))
            .send()
            .await
            .map_err(|e| format!("Erro de rede: {e}"))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "Supabase Auth retornou status {} ao atualizar usuário",
                resp.status()
            ))
        }
    }

    /// Remove usuário do Supabase Auth pelo UUID.
    pub async fn excluir_usuario(&self, user_id: &str) -> Result<(), String> {
        let url = format!("{}/auth/v1/admin/users/{}", self.base_url, user_id);
        let resp = self
            .client
            .delete(&url)
            .header("apikey", &self.service_role_key)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Erro de rede: {e}"))?;

        if resp.status().is_success() || resp.status().as_u16() == 404 {
            Ok(())
        } else {
            Err(format!("Supabase Auth retornou status {}", resp.status()))
        }
    }
}
