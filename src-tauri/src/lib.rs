use decruft::{parse, DecruftOptions};
use rusqlite::{params, Connection};
use std::{fs, sync::Mutex};
use std::fs::File;
use std::io::Write;
use tauri_plugin_dialog::DialogExt;
use tauri::Manager;
use serde::{Serialize, Deserialize};
use scraper::{Html, Selector};
use url::Url;
use base64::{prelude::BASE64_STANDARD, Engine};

#[derive(Serialize, Deserialize, Clone)]
pub struct Article {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub content: String,
    pub tag: Option<String>,
    pub scroll_progress: f64,
}

pub fn init_db(app_handle: &tauri::AppHandle) -> rusqlite::Result<Connection> {
    let mut db_path = app_handle.path().app_data_dir().expect("Failed to get app data dir");
    fs::create_dir_all(&db_path).expect("Failed to create app data dir");
    db_path.push("minted_articles.db");
    
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY,
            url TEXT UNIQUE NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tag TEXT,
            scroll_progress REAL DEFAULT 0.0
        )",
        [],
    ).map_err(|e| {
        eprintln!("DATABASE INIT FAILED: {}", e);
        e
    })?;

    let columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(articles)")?;
        let result: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|res| res.ok())
            .collect();
        drop(stmt);
        result
    };

    if !columns.contains(&"tag".to_string()) {
        conn.execute("ALTER TABLE articles ADD COLUMN tag TEXT", [])?;
    }
    if !columns.contains(&"scroll_progress".to_string()) {
        conn.execute("ALTER TABLE articles ADD COLUMN scroll_progress REAL DEFAULT 0.0", [])?;
    }

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
            title,
            content,
            content='articles',
            content_rowid='id'
        );",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
            INSERT INTO articles_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
        END;",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
            INSERT INTO articles_fts(articles_fts, rowid, title, content) VALUES ('delete', old.id, old.title, old.content);
        END;",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
            INSERT INTO articles_fts(articles_fts, rowid, title, content) VALUES ('delete', old.id, old.title, old.content);
            INSERT INTO articles_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
        END;",
        [],
    )?;

    Ok(conn)
}

fn inline_remote_assets(html_content: &str, source_url: &str) -> String {
    let document = Html::parse_document(html_content);
    let img_selector = Selector::parse("img").unwrap();

    let mut processed_html = html_content.to_string();
    let base_uri = Url::parse(source_url).ok();

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/120.0.0.0")
        .build()
        .unwrap_or_default();

    for element in document.select(&img_selector) {
        let (attr_name, src) = if let Some(s) = element.value().attr("data-src") {
            ("data-src", s)
        } else if let Some(s) = element.value().attr("src") {
            ("src", s)
        } else {
            continue;
        };

        if src.starts_with("data:") || src.is_empty() {
            continue;
        }

        let normalized_src = if src.starts_with("//") {
            format!("https:{}", src)
        } else {
            src.to_string()
        };

        let absolute_img_url = match Url::parse(&normalized_src) {
            Ok(url) => url,
            Err(_) => {
                if let Some(ref base) = base_uri {
                    match base.join(&normalized_src) {
                        Ok(joined) => joined,
                        Err(_) => continue,
                    }
                } else {
                    continue;
                }
            }
        };

        if let Ok(response) = client.get(absolute_img_url.as_str()).send() {
            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|val| val.to_str().ok())
                .unwrap_or("image/png")
                .to_string();
            
            if let Ok(bytes) = response.bytes() {
                let base64_str = BASE64_STANDARD.encode(&bytes);
                let data_uri = format!("data:{};base64,{}", content_type, base64_str);
                
                let target_double = format!("{}=\"{}\"", attr_name, src);
                let replacement_double = format!("src=\"{}\"", data_uri);
                
                let target_single = format!("{}='{}'", attr_name, src);
                let replacement_single = format!("src='{}'", data_uri);

                processed_html = processed_html
                    .replace(&target_double, &replacement_double)
                    .replace(&target_single, &replacement_single);

                if let Some(srcset) = element.value().attr("srcset") {
                    let srcset_double = format!("srcset=\"{}\"", srcset);
                    let srcset_single = format!("srcset='{}'", srcset);
                    processed_html = processed_html
                        .replace(&srcset_double, "")
                        .replace(&srcset_single, "");
                }
            }
        }
    }
    
    processed_html
}

fn remove_superscripts(html: &str) -> String {
    let re = regex::Regex::new(r"(?i)<sup\b[^>]*>.*?</sup>").unwrap();
    re.replace_all(html, "").to_string()
}

fn strip_anchor_tags(html: &str) -> String {
    let mut result = String::new();
    let mut input = html;
    
    while let Some(start_idx) = input.find("<a") {
        result.push_str(&input[..start_idx]);
        let rest = &input[start_idx..];
        
        if let Some(end_tag_idx) = rest.find('>') {
            if rest.starts_with("<a ") || rest.starts_with("<a>") {
                input = &rest[end_tag_idx + 1..];
                continue;
            }
        }
        result.push('<');
        input = &input[start_idx + 1..];
    }
    result.push_str(input);
    result.replace("</a>", "").replace("</A>", "")
}

#[tauri::command]
fn get_articles(
    state: tauri::State<'_, Mutex<Connection>>
) -> Result<Vec<Article>, String> {
    let conn = state.lock().map_err(|_| "Failed to lock database".to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, url, title, content, tag, scroll_progress FROM articles ORDER BY id DESC")
        .map_err(|e| e.to_string())?;
        
    let article_iter = stmt
        .query_map([], |row| {
            Ok(Article {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                tag: row.get(4)?,
                scroll_progress: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut articles = Vec::new();
    for article in article_iter {
        articles.push(article.map_err(|e| e.to_string())?);
    }
    Ok(articles)
}

#[tauri::command]
fn mint_archive(
    state: tauri::State<'_, Mutex<Connection>>,
    url: String
) -> Result<Article, String> {
    let jina_url = format!("https://r.jina.ai/{}", url);
    
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/120.0.0.0")
        .build()
        .map_err(|e| format!("Failed to build network client: {}", e))?;

    let response = client.get(&jina_url)
        .header("X-Respond-With", "html")
        .send()
        .map_err(|e| format!("Network protocol coordination failure: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("The website blocked the request (Error {}). It likely has anti-bot protection.", response.status()));
    }

    let full_html = response.text().map_err(|e| e.to_string())?;
    
    if full_html.contains("Attention Required! | Cloudflare") {
        return Err("Minting failed: Blocked by Cloudflare Captcha.".to_string());
    }
    
    let options = DecruftOptions::default();
    let decrufted = parse(&full_html, &options);
    
    let title = decrufted.title;
    let content = decrufted.content;
    
    let compiled_standalone_content = inline_remote_assets(&content, &url);
    let no_sups = remove_superscripts(&compiled_standalone_content);
    let fully_cleaned_content = strip_anchor_tags(&no_sups);

    let conn = state.lock().map_err(|_| "Failed to lock database".to_string())?;

    conn.execute(
        "INSERT INTO articles (url, content, title) VALUES (?1, ?2, ?3)
         ON CONFLICT(url) DO UPDATE SET content = excluded.content, title = excluded.title",
        params![url, fully_cleaned_content, title],
    ).map_err(|e| format!("Database error: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, url, title, content, tag, scroll_progress FROM articles WHERE url = ?1")
        .map_err(|e| e.to_string())?;

    let article = stmt.query_row(params![url], |row| {
        Ok(Article {
            id: row.get(0)?,
            url: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            tag: row.get(4)?,
            scroll_progress: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    Ok(article)
}

#[tauri::command]
fn delete_article(state: tauri::State<'_, Mutex<Connection>>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|_| "Failed to lock database".to_string())?;
    conn.execute("DELETE FROM articles WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_article_tag(state: tauri::State<'_, Mutex<Connection>>, id: i64, tag: Option<String>) -> Result<(), String> {
    let conn = state.lock().map_err(|_| "Failed to lock database".to_string())?;
    conn.execute("UPDATE articles SET tag = ?1 WHERE id = ?2", params![tag, id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_scroll_progress(state: tauri::State<'_, Mutex<Connection>>, id: i64, progress: f64) -> Result<(), String> {
    let conn = state.lock().map_err(|_| "Failed to lock database".to_string())?;
    conn.execute(
        "UPDATE articles SET scroll_progress = ?1 WHERE id = ?2",
        params![progress, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn deep_fts_search(state: tauri::State<'_, Mutex<Connection>>, query: String) -> Result<Vec<Article>, String> {
    let conn = state.lock().map_err(|_| "Failed to lock database".to_string())?;
    let mut stmt = conn.prepare(
        "SELECT a.id, a.url, a.title, a.content, a.tag, a.scroll_progress 
         FROM articles a
         JOIN articles_fts fts ON a.id = fts.rowid 
         WHERE articles_fts MATCH ?1 
         ORDER BY bm25(articles_fts)"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![query], |row| {
        Ok(Article {
            id: row.get(0)?,
            url: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            tag: row.get(4)?,
            scroll_progress: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows {
        if let Ok(art) = row { results.push(art); }
    }
    Ok(results)
}

#[tauri::command]
fn export_to_markdown(app_handle: tauri::AppHandle, title: String, url: String, content: String) -> Result<(), String> {
    let re_img = regex::Regex::new(r"(?i)<img\b[^>]*>").unwrap();
    let content_no_images = re_img.replace_all(&content, "").to_string();

    let stripped_text = html2md::parse_html(&content_no_images);

    let markdown_payload = format!(
        "---\ntitle: {}\nsource: {}\nminted_via: ArchiveMinter\n---\n\n{}", 
        title, url, stripped_text
    );

    let sanitized_title: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '-' })
        .collect();

    app_handle.dialog()
        .file()
        .set_file_name(&format!("{}.md", sanitized_title.trim()))
        .save_file(move |file_path| {
            if let Some(path_enum) = file_path {
                if let Ok(path) = path_enum.into_path() {
                    if let Ok(mut file) = File::create(path) {
                        let _ = file.write_all(markdown_payload.as_bytes());
                    }
                }
            }
        });

   Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let conn = init_db(&app.app_handle()).expect("Failed to initialize database");
            app.manage(Mutex::new(conn));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init()) 
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_articles,
            mint_archive,
            delete_article,
            update_article_tag,
            save_scroll_progress,
            deep_fts_search,
            export_to_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}