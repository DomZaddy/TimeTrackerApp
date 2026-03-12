use crate::{AppState, LiveStatusData, SessionBlock, WeekHoursResult};
use crate::oauth;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;
use serde::Deserialize;
use chrono::Datelike;

const DEFAULT_SHEET_ID: &str = "1dGwy97xuX6Vu6OfYTmDqmOlqXRRfqsb79RYyHqS775g";
const MONTH_NAMES: &[&str] = &[
    "JANUARY", "FEBRUARY", "MARCH", "APRIL", "MAY", "JUNE",
    "JULY", "AUGUST", "SEPTEMBER", "OCTOBER", "NOVEMBER", "DECEMBER",
];

#[derive(Deserialize)]
struct ValuesResponse {
    values: Option<Vec<Vec<String>>>,
}

#[derive(Deserialize)]
struct SheetsError {
    error: Option<SheetsErrorDetail>,
}

#[derive(Deserialize)]
struct SheetsErrorDetail {
    message: Option<String>,
}

struct SheetsContext {
    access_token: String,
    refresh_token: Option<String>,
    spreadsheet_id: String,
}

async fn get_sheets_context(app: &AppHandle) -> Result<SheetsContext, Box<dyn std::error::Error + Send + Sync>> {
    let state = app.state::<AppState>();
    let tokens = state.oauth_tokens.lock().unwrap().clone()
        .ok_or("Not signed in. Connect Google in Settings.")?;

    // Check if token needs refresh
    let access_token = if let Some(expiry) = tokens.expiry_time {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now >= expiry {
            if let Some(ref refresh_token) = tokens.refresh_token {
                let new_tokens = oauth::refresh_access_token(refresh_token).await?;
                let new_access = new_tokens.access_token.clone();
                // Update stored tokens
                *state.oauth_tokens.lock().unwrap() = Some(new_tokens.clone());
                let store = app.store("timetracker-store.json")
                    .map_err(|e| format!("Store error: {}", e))?;
                let _ = store.set("oauth-tokens", serde_json::to_value(&new_tokens)?);
                new_access
            } else {
                tokens.access_token.clone()
            }
        } else {
            tokens.access_token.clone()
        }
    } else {
        tokens.access_token.clone()
    };

    // Get spreadsheet ID from store
    let store = app.store("timetracker-store.json")
        .map_err(|e| format!("Store error: {}", e))?;
    let raw_id = store.get("sheet-id")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| DEFAULT_SHEET_ID.to_string());

    // Extract ID from URL if pasted
    let spreadsheet_id = if raw_id.contains("/spreadsheets/d/") {
        raw_id.split("/spreadsheets/d/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or(&raw_id)
            .to_string()
    } else {
        raw_id
    };

    Ok(SheetsContext {
        access_token,
        refresh_token: tokens.refresh_token,
        spreadsheet_id,
    })
}

fn get_sheet_tab() -> String {
    let now = chrono::Local::now();
    format!("{}/{}", MONTH_NAMES[now.month0() as usize], now.format("%Y"))
}

fn get_sheet_tab_for_date(timestamp_ms: f64) -> String {
    let dt = chrono::DateTime::from_timestamp_millis(timestamp_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now());
    let local = dt.with_timezone(&chrono::Local);
    format!("{}/{}", MONTH_NAMES[local.month0() as usize], local.format("%Y"))
}

fn format_date_for_sheet(timestamp_ms: f64) -> String {
    let dt = chrono::DateTime::from_timestamp_millis(timestamp_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now());
    let local = dt.with_timezone(&chrono::Local);
    local.format("%m/%d/%Y").to_string()
}

fn format_time_for_sheet(timestamp_ms: f64) -> String {
    let dt = chrono::DateTime::from_timestamp_millis(timestamp_ms as i64)
        .unwrap_or_else(|| chrono::Utc::now());
    let local = dt.with_timezone(&chrono::Local);
    local.format("%-I:%M %p").to_string()
}

fn format_elapsed_short(ms: f64) -> String {
    let total_min = (ms / 60000.0).floor() as i64;
    let h = total_min / 60;
    let m = total_min % 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}

async fn sheets_get(ctx: &SheetsContext, range: &str) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
        ctx.spreadsheet_id,
        urlencoding(range)
    );
    let client = reqwest::Client::new();
    let resp = client.get(&url)
        .bearer_auth(&ctx.access_token)
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await?;
        if let Ok(err) = serde_json::from_str::<SheetsError>(&text) {
            if let Some(detail) = err.error {
                return Err(detail.message.unwrap_or(text).into());
            }
        }
        return Ok(vec![]);
    }

    let data: ValuesResponse = resp.json().await?;
    Ok(data.values.unwrap_or_default())
}

async fn sheets_update(ctx: &SheetsContext, range: &str, values: Vec<Vec<String>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sheets_update_with_option(ctx, range, values, "USER_ENTERED").await
}

async fn sheets_update_raw(ctx: &SheetsContext, range: &str, values: Vec<Vec<String>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sheets_update_with_option(ctx, range, values, "RAW").await
}

async fn sheets_update_with_option(ctx: &SheetsContext, range: &str, values: Vec<Vec<String>>, input_option: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?valueInputOption={}",
        ctx.spreadsheet_id,
        urlencoding(range),
        input_option
    );
    let client = reqwest::Client::new();
    let body = serde_json::json!({ "values": values });
    let resp = client.put(&url)
        .bearer_auth(&ctx.access_token)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await?;
        return Err(format!("Sheets update error: {}", text).into());
    }
    Ok(())
}

async fn sheets_append(ctx: &SheetsContext, range: &str, values: Vec<Vec<String>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}:append?valueInputOption=USER_ENTERED&insertDataOption=INSERT_ROWS",
        ctx.spreadsheet_id,
        urlencoding(range)
    );
    let client = reqwest::Client::new();
    let body = serde_json::json!({ "values": values });
    let resp = client.post(&url)
        .bearer_auth(&ctx.access_token)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let text = resp.text().await?;
        return Err(format!("Sheets append error: {}", text).into());
    }
    Ok(())
}

async fn sheets_clear(ctx: &SheetsContext, range: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}:clear",
        ctx.spreadsheet_id,
        urlencoding(range)
    );
    let client = reqwest::Client::new();
    let resp = client.post(&url)
        .bearer_auth(&ctx.access_token)
        .json(&serde_json::json!({}))
        .send()
        .await?;

    if !resp.status().is_success() {
        // Silently fail
    }
    Ok(())
}

fn dates_match(cell_date: &str, target_date: &str) -> bool {
    // Exact match first
    if cell_date == target_date {
        return true;
    }
    // Normalize: strip leading zeros and compare
    let normalize = |s: &str| -> String {
        s.split('/')
            .map(|p| p.trim_start_matches('0'))
            .collect::<Vec<_>>()
            .join("/")
    };
    normalize(cell_date) == normalize(target_date)
}

async fn find_date_row(ctx: &SheetsContext, sheet_tab: &str, date_str: &str) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let range = format!("'{}'!A:B", sheet_tab);
    let values = sheets_get(ctx, &range).await.unwrap_or_default();

    let mut last_row_for_date: i64 = -1;
    let mut empty_date_row: i64 = -1; // Row with matching date but empty task (reusable)
    let mut first_empty_row: i64 = -1;

    for (i, row) in values.iter().enumerate() {
        let cell_date = row.first().map(|s| s.trim()).unwrap_or("");
        let cell_task = row.get(1).map(|s| s.trim()).unwrap_or("");

        if dates_match(cell_date, date_str) {
            if cell_task.starts_with("Currently working") {
                return Ok((i + 1) as i64);
            }
            if cell_task.is_empty() && empty_date_row < 0 {
                // Date exists but task is empty — reuse this row
                empty_date_row = (i + 1) as i64;
            }
            last_row_for_date = (i + 1) as i64;
        }

        // Track first completely empty row (skip header row 1)
        if i > 0 && cell_date.is_empty() && cell_task.is_empty() && first_empty_row < 0 {
            first_empty_row = (i + 1) as i64;
        }
    }

    // Priority: reuse empty date row > after last date row > first empty row > append
    if empty_date_row > 0 {
        Ok(empty_date_row)
    } else if last_row_for_date > 0 {
        Ok(last_row_for_date + 1)
    } else if first_empty_row > 0 {
        Ok(first_empty_row)
    } else {
        Ok((values.len() + 1) as i64)
    }
}

pub async fn push_session(app: &AppHandle, rows: Vec<SessionBlock>) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let ctx = get_sheets_context(app).await?;
    let state = app.state::<AppState>();

    // Derive sheet tab from first block's date
    let sheet_tab = if !rows.is_empty() {
        // Parse MM/DD/YYYY
        let parts: Vec<&str> = rows[0].date.split('/').collect();
        if parts.len() == 3 {
            let month: usize = parts[0].parse().unwrap_or(1);
            let year = parts[2];
            format!("{}/{}", MONTH_NAMES[month.saturating_sub(1)], year)
        } else {
            get_sheet_tab()
        }
    } else {
        get_sheet_tab()
    };

    let count = rows.len();

    let status_row = state.live_status_row.lock().unwrap().clone();
    let status_tab = state.live_status_tab.lock().unwrap().clone();

    // Determine the starting row for the push
    let start_row = if let (Some(row), Some(ref tab)) = (status_row, &status_tab) {
        if *tab == sheet_tab { row } else {
            find_date_row(&ctx, &sheet_tab, &rows[0].date).await?
        }
    } else {
        find_date_row(&ctx, &sheet_tab, &rows[0].date).await?
    };

    // Write only data columns, skip formula columns (C, D, H, I, J, etc.)
    for (i, row) in rows.iter().enumerate() {
        let r = start_row + i as i64;
        println!("[push_session] row {} → date={}, task={}, notes_len={}, notes={}",
            r, row.date, row.task.len(), row.notes.len(), row.notes);

        // A-B: Date, Task (RAW to preserve leading zeros in dates)
        let ab_range = format!("'{}'!A{}:B{}", sheet_tab, r, r);
        sheets_update_raw(&ctx, &ab_range, vec![vec![
            row.date.clone(), row.task.clone(),
        ]]).await?;

        // E-G: Check In, Check Out, Break Hours
        let eg_range = format!("'{}'!E{}:G{}", sheet_tab, r, r);
        sheets_update(&ctx, &eg_range, vec![vec![
            row.check_in.clone(), row.check_out.clone(), format!("{}", row.break_hours),
        ]]).await?;

        // H: Total Hours formula (USER_ENTERED so Sheets interprets the formula)
        let h_range = format!("'{}'!H{}", sheet_tab, r);
        let h_formula = format!("=IF(OR(E{}=\"\",F{}=\"\"),0,(F{}-E{})*24-G{})", r, r, r, r, r);
        sheets_update(&ctx, &h_range, vec![vec![h_formula]]).await?;

        // K: Notes (RAW to write as plain text)
        if !row.notes.is_empty() {
            let k_range = format!("'{}'!K{}", sheet_tab, r);
            println!("[push_session] writing notes to {}: {}", k_range, row.notes);
            sheets_update_raw(&ctx, &k_range, vec![vec![row.notes.clone()]]).await?;
        }
    }

    // Clean up stale data: if the live status row is outside the written range, clear it
    let last_written_row = start_row + count as i64 - 1;
    if let (Some(status_r), Some(ref status_t)) = (status_row, &status_tab) {
        if *status_t == sheet_tab && (status_r < start_row || status_r > last_written_row) {
            // Status row exists outside our written blocks — clear it
            println!("[push_session] clearing stale status row {} (wrote rows {}-{})", status_r, start_row, last_written_row);
            let stale_range = format!("'{}'!A{}:K{}", sheet_tab, status_r, status_r);
            let _ = sheets_clear(&ctx, &stale_range).await;
        }
    }

    // Clear any gap rows between last written row and the old status row
    if let (Some(status_r), Some(ref status_t)) = (status_row, &status_tab) {
        if *status_t == sheet_tab && status_r > last_written_row + 1 {
            // There are empty/stale rows between our blocks and the old status row
            for gap_row in (last_written_row + 1)..status_r {
                println!("[push_session] clearing gap row {}", gap_row);
                let gap_range = format!("'{}'!A{}:K{}", sheet_tab, gap_row, gap_row);
                let _ = sheets_clear(&ctx, &gap_range).await;
            }
        }
    }

    // Clear live status tracking
    *state.live_status_row.lock().unwrap() = None;
    *state.live_status_tab.lock().unwrap() = None;

    let store = app.store("timetracker-store.json")
        .map_err(|e| format!("Store error: {}", e))?;
    let _ = store.delete("live-status-row");
    let _ = store.delete("live-status-tab");

    Ok(count)
}

pub async fn status_start(app: &AppHandle, data: LiveStatusData) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let ctx = get_sheets_context(app).await?;
    let state = app.state::<AppState>();

    let sheet_tab = get_sheet_tab_for_date(data.start_time);
    let date_str = format_date_for_sheet(data.start_time);
    let time_str = format_time_for_sheet(data.start_time);
    let status_text = if data.task.is_empty() {
        "Currently working...".to_string()
    } else {
        format!("Currently working on: {}", data.task)
    };

    let target_row = find_date_row(&ctx, &sheet_tab, &date_str).await?;

    // Write only data columns, preserve formula columns
    // A-B: Date, Status text (RAW to preserve leading zeros)
    let ab_range = format!("'{}'!A{}:B{}", sheet_tab, target_row, target_row);
    sheets_update_raw(&ctx, &ab_range, vec![vec![date_str, status_text]]).await?;

    // E-F: Check In, empty check-out
    let ef_range = format!("'{}'!E{}:F{}", sheet_tab, target_row, target_row);
    sheets_update(&ctx, &ef_range, vec![vec![time_str.clone(), String::new()]]).await?;

    // K: Notes
    let k_range = format!("'{}'!K{}", sheet_tab, target_row);
    sheets_update_raw(&ctx, &k_range, vec![vec![format!("Started at {}", time_str)]]).await?;

    *state.live_status_row.lock().unwrap() = Some(target_row);
    *state.live_status_tab.lock().unwrap() = Some(sheet_tab.clone());

    let store = app.store("timetracker-store.json")
        .map_err(|e| format!("Store error: {}", e))?;
    let _ = store.set("live-status-row", serde_json::json!(target_row));
    let _ = store.set("live-status-tab", serde_json::json!(sheet_tab));

    Ok(target_row)
}

pub async fn status_update(app: &AppHandle, data: LiveStatusData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = get_sheets_context(app).await?;
    let state = app.state::<AppState>();

    let row = state.live_status_row.lock().unwrap().clone()
        .ok_or("No status row tracked")?;
    let sheet_tab = state.live_status_tab.lock().unwrap().clone()
        .unwrap_or_else(get_sheet_tab);

    let date_str = format_date_for_sheet(data.start_time);
    let time_str = format_time_for_sheet(data.start_time);
    let elapsed_str = format_elapsed_short(data.elapsed);
    let status_text = if data.task.is_empty() {
        format!("Currently working... ({})", elapsed_str)
    } else {
        format!("Currently working on: {} ({})", data.task, elapsed_str)
    };
    let now_ms = chrono::Local::now().timestamp_millis() as f64;
    let update_time = format_time_for_sheet(now_ms);

    // Write only data columns, preserve formula columns
    // A-B: Date, Status text (RAW to preserve leading zeros)
    let ab_range = format!("'{}'!A{}:B{}", sheet_tab, row, row);
    sheets_update_raw(&ctx, &ab_range, vec![vec![date_str, status_text]]).await?;

    // E-F: Check In, empty check-out
    let ef_range = format!("'{}'!E{}:F{}", sheet_tab, row, row);
    sheets_update(&ctx, &ef_range, vec![vec![time_str, String::new()]]).await?;

    // K: Notes
    let k_range = format!("'{}'!K{}", sheet_tab, row);
    sheets_update_raw(&ctx, &k_range, vec![vec![format!("Last update: {}", update_time)]]).await?;

    Ok(())
}

pub async fn status_clear(app: &AppHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = get_sheets_context(app).await?;
    let state = app.state::<AppState>();

    let row = state.live_status_row.lock().unwrap().clone();
    let sheet_tab = state.live_status_tab.lock().unwrap().clone()
        .unwrap_or_else(get_sheet_tab);

    if let Some(row) = row {
        let range = format!("'{}'!A{}:K{}", sheet_tab, row, row);
        let _ = sheets_clear(&ctx, &range).await;
    }

    *state.live_status_row.lock().unwrap() = None;
    *state.live_status_tab.lock().unwrap() = None;

    let store = app.store("timetracker-store.json")
        .map_err(|e| format!("Store error: {}", e))?;
    let _ = store.delete("live-status-row");
    let _ = store.delete("live-status-tab");

    Ok(())
}

pub async fn fetch_week_hours(app: &AppHandle) -> Result<WeekHoursResult, Box<dyn std::error::Error + Send + Sync>> {
    let ctx = get_sheets_context(app).await?;

    let now = chrono::Local::now();
    let day_of_week = now.weekday().num_days_from_sunday();
    let sunday = now - chrono::Duration::days(day_of_week as i64);
    let sunday = sunday.date_naive().and_hms_opt(0, 0, 0).unwrap();

    let mut tabs = std::collections::HashSet::new();
    let sunday_month = sunday.month0() as usize;
    let sunday_year = sunday.format("%Y").to_string();
    tabs.insert(format!("{}/{}", MONTH_NAMES[sunday_month], sunday_year));

    let now_month = now.month0() as usize;
    let now_year = now.format("%Y").to_string();
    let current_month_tab = format!("{}/{}", MONTH_NAMES[now_month], now_year);
    tabs.insert(current_month_tab.clone());

    let mut week_ms: f64 = 0.0;
    let mut monthly_pay: f64 = 0.0;
    let mut monthly_hours: f64 = 0.0;

    for sheet_tab in &tabs {
        let range = format!("'{}'!A:Q", sheet_tab);
        let rows = match sheets_get(&ctx, &range).await {
            Ok(r) => r,
            Err(_) => continue,
        };

        let is_current_month = sheet_tab == &current_month_tab;

        for row in &rows {
            let date_str = row.first().map(|s| s.trim()).unwrap_or("");
            let total_hours_str = row.get(7).map(|s| s.trim()).unwrap_or("");
            let pay_str = row.get(16).map(|s| s.trim()).unwrap_or("");

            if date_str.is_empty() || total_hours_str.is_empty() {
                continue;
            }

            let task_str = row.get(1).map(|s| s.trim()).unwrap_or("");
            if task_str.starts_with("Currently working") {
                continue;
            }

            let hours: f64 = match total_hours_str.parse() {
                Ok(h) if h > 0.0 => h,
                _ => continue,
            };

            // Parse date MM/DD/YYYY
            let parts: Vec<&str> = date_str.split('/').collect();
            if parts.len() != 3 { continue; }
            let month: u32 = parts[0].parse().unwrap_or(0);
            let day: u32 = parts[1].parse().unwrap_or(0);
            let year: i32 = parts[2].parse().unwrap_or(0);

            if let Some(row_date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                // Weekly hours
                if row_date >= sunday.date() && row_date <= now.date_naive() {
                    week_ms += hours * 3600000.0;
                }
            }

            // Monthly totals
            if is_current_month {
                monthly_hours += hours;
                let pay_clean = pay_str.replace(['$', ','], "");
                if let Ok(pay_val) = pay_clean.parse::<f64>() {
                    monthly_pay += pay_val;
                }
            }
        }
    }

    let week_start = sunday.and_utc().timestamp_millis() as f64;

    Ok(WeekHoursResult {
        success: true,
        error: None,
        week_ms,
        monthly_pay,
        monthly_hours,
        week_start,
    })
}

fn urlencoding(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('\'', "%27")
        .replace('!', "%21")
        .replace(':', "%3A")
        .replace('/', "%2F")
}
