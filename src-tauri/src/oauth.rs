use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;
use std::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const OAUTH_CLIENT_ID: &str = env!("OAUTH_CLIENT_ID", "Set OAUTH_CLIENT_ID env var before building");
const OAUTH_CLIENT_SECRET: &str = env!("OAUTH_CLIENT_SECRET", "Set OAUTH_CLIENT_SECRET env var before building");
const SCOPES: &str = "https://www.googleapis.com/auth/spreadsheets";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub expiry_time: Option<u64>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: Option<u64>,
}

pub async fn start_oauth_flow(app: &AppHandle) -> Result<OAuthTokens, Box<dyn std::error::Error + Send + Sync>> {
    // Check if we already have valid tokens
    let store = app.store("timetracker-store.json")
        .map_err(|e| format!("Store error: {}", e))?;

    if let Some(tokens_val) = store.get("oauth-tokens") {
        if let Ok(mut tokens) = serde_json::from_value::<OAuthTokens>(tokens_val) {
            // Try to refresh if expired
            if let Some(expiry) = tokens.expiry_time {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now >= expiry {
                    if let Some(ref refresh_token) = tokens.refresh_token {
                        if let Ok(new_tokens) = refresh_access_token(refresh_token).await {
                            tokens.access_token = new_tokens.access_token;
                            tokens.expires_in = new_tokens.expires_in;
                            tokens.expiry_time = new_tokens.expires_in.map(|e| {
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() + e
                            });
                            let _ = store.set("oauth-tokens", serde_json::to_value(&tokens)?);
                            return Ok(tokens);
                        }
                    }
                } else {
                    return Ok(tokens);
                }
            } else {
                return Ok(tokens);
            }
        }
    }

    // Start OAuth loopback flow
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
        client_id={}&\
        redirect_uri={}&\
        response_type=code&\
        scope={}&\
        access_type=offline&\
        prompt=consent",
        urlencoding(&OAUTH_CLIENT_ID),
        urlencoding(&redirect_uri),
        urlencoding(&SCOPES),
    );

    // Open browser
    let _ = app.shell().open(&auth_url, None);

    // Wait for callback
    let listener = tokio::net::TcpListener::from_std(listener)?;
    let (mut stream, _) = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        listener.accept()
    ).await.map_err(|_| "Sign-in timed out. Try again.")??;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // Parse the code from the request
    let code = extract_code(&request)
        .ok_or("No authorization code received")?;

    // Exchange code for tokens
    let client = reqwest::Client::new();
    let resp = client.post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code.as_str()),
            ("client_id", OAUTH_CLIENT_ID),
            ("client_secret", OAUTH_CLIENT_SECRET),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?;

    let token_resp: TokenResponse = resp.json().await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let tokens = OAuthTokens {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        token_type: token_resp.token_type,
        expires_in: token_resp.expires_in,
        expiry_time: token_resp.expires_in.map(|e| now + e),
    };

    // Save tokens
    let _ = store.set("oauth-tokens", serde_json::to_value(&tokens)?);

    // Send success response to browser
    let html = r#"<html><body style="background:#0f0f1a;color:#e0e0e0;font-family:system-ui;display:flex;align-items:center;justify-content:center;height:100vh;margin:0">
        <div style="text-align:center"><h1 style="color:#00d68f">Connected!</h1><p>You can close this tab and return to TimeTracker.</p></div>
    </body></html>"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        html.len(), html
    );
    let _ = stream.write_all(response.as_bytes()).await;

    Ok(tokens)
}

pub async fn refresh_access_token(refresh_token: &str) -> Result<OAuthTokens, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let resp = client.post("https://oauth2.googleapis.com/token")
        .form(&[
            ("refresh_token", refresh_token),
            ("client_id", OAUTH_CLIENT_ID),
            ("client_secret", OAUTH_CLIENT_SECRET),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await?;

    let token_resp: TokenResponse = resp.json().await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(OAuthTokens {
        access_token: token_resp.access_token,
        refresh_token: Some(refresh_token.to_string()),
        token_type: token_resp.token_type,
        expires_in: token_resp.expires_in,
        expiry_time: token_resp.expires_in.map(|e| now + e),
    })
}

fn extract_code(request: &str) -> Option<String> {
    let first_line = request.lines().next()?;
    let path = first_line.split_whitespace().nth(1)?;
    if !path.starts_with("/callback") {
        return None;
    }
    let query = path.split('?').nth(1)?;
    for param in query.split('&') {
        let mut kv = param.splitn(2, '=');
        if kv.next()? == "code" {
            return Some(kv.next()?.to_string());
        }
    }
    None
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
        .replace(':', "%3A")
        .replace('/', "%2F")
        .replace('@', "%40")
}
