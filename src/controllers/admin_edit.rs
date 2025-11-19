use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::services::{config_service::ConfigService, url_validation_service::UrlValidationService};

/// URL更新フォームのデータ構造体
#[derive(Debug, Deserialize)]
pub struct UrlUpdateForm {
    /// 削除対象のURLインデックス（チェックボックスで選択）
    #[serde(default, deserialize_with = "deserialize_indices")]
    pub delete_indices: Vec<usize>,
    
    /// 新規追加するURL（改行区切り）
    #[serde(default)]
    pub new_urls: String,
}

/// 単一の値または配列を受け取るカスタムデシリアライザ
fn deserialize_indices<'de, D>(deserializer: D) -> Result<Vec<usize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Deserialize};
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }
    
    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::String(s) => {
            s.parse::<usize>()
                .map(|v| vec![v])
                .map_err(de::Error::custom)
        }
        StringOrVec::Vec(v) => {
            v.into_iter()
                .map(|s| s.parse::<usize>().map_err(de::Error::custom))
                .collect()
        }
    }
}

/// URL編集フォーム表示
pub async fn edit(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // ConfigServiceから現在のURL一覧を取得
    let urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

    // URL一覧をHTMLで生成
    let mut url_list_html = String::new();
    for (idx, url) in urls.iter().enumerate() {
        url_list_html.push_str(&format!(
            r#"
            <div class="url-item">
                <input type="checkbox" name="delete_indices" value="{}" id="url_{}">
                <label for="url_{}">{}</label>
            </div>
            "#,
            idx,
            idx,
            idx,
            html_escape::encode_text(url)
        ));
    }

    let html = generate_edit_page_html(&url_list_html, "", false);
    Ok(Html(html))
}

/// URL更新処理
pub async fn update(
    State(_ctx): State<AppContext>,
    Form(form): Form<UrlUpdateForm>,
) -> Result<impl IntoResponse> {
    // 現在のURL一覧を取得
    let mut current_urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

    // 削除対象のインデックスをソートして逆順にする（後ろから削除）
    let mut delete_indices = form.delete_indices.clone();
    delete_indices.sort_unstable();
    delete_indices.reverse();

    // 削除処理
    for idx in delete_indices {
        if idx < current_urls.len() {
            current_urls.remove(idx);
        }
    }

    // 新規URL追加処理
    let new_urls: Vec<String> = form
        .new_urls
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    // 新規URLの検証
    if let Err(e) = UrlValidationService::validate_urls(&new_urls) {
        // バリデーションエラー時は編集画面を再表示
        let url_list_html = generate_url_list_html(&current_urls);
        let error_message = format!("URLの検証に失敗しました: {}", e);
        let html = generate_edit_page_html(&url_list_html, &error_message, true);
        return Ok(Html(html).into_response());
    }

    // 新規URLを追加
    current_urls.extend(new_urls);

    // ConfigServiceでURL一覧を更新
    if let Err(e) = ConfigService::update_urls(current_urls.clone()) {
        // 更新エラー時は編集画面を再表示
        let url_list_html = generate_url_list_html(&current_urls);
        let error_message = format!("設定ファイルの更新に失敗しました: {}", e);
        let html = generate_edit_page_html(&url_list_html, &error_message, true);
        return Ok(Html(html).into_response());
    }

    // 成功時は一覧ページへリダイレクト
    Ok(Redirect::to("/admin/list").into_response())
}

/// URL一覧のHTML生成（エラー時の再表示用）
fn generate_url_list_html(urls: &[String]) -> String {
    let mut html = String::new();
    for (idx, url) in urls.iter().enumerate() {
        html.push_str(&format!(
            r#"
            <div class="url-item">
                <input type="checkbox" name="delete_indices" value="{}" id="url_{}">
                <label for="url_{}">{}</label>
            </div>
            "#,
            idx,
            idx,
            idx,
            html_escape::encode_text(url)
        ));
    }
    html
}

/// 編集ページのHTML生成
fn generate_edit_page_html(url_list_html: &str, error_message: &str, is_error: bool) -> String {
    let error_html = if is_error && !error_message.is_empty() {
        format!(
            r#"
            <div class="error-message">
                <strong>エラー:</strong> {}
            </div>
            "#,
            html_escape::encode_text(error_message)
        )
    } else if !is_error && !error_message.is_empty() {
        format!(
            r#"
            <div class="success-message">
                {}
            </div>
            "#,
            html_escape::encode_text(error_message)
        )
    } else {
        String::new()
    };

    format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>URL編集 - Telegraf設定管理</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background: #f5f5f5;
        }}
        .header {{
            background: #667eea;
            color: white;
            padding: 1rem 2rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .header h1 {{
            margin: 0;
        }}
        .nav {{
            background: white;
            padding: 1rem 2rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.05);
        }}
        .nav a {{
            color: #667eea;
            text-decoration: none;
            margin-right: 1.5rem;
            font-weight: 500;
        }}
        .nav a:hover {{
            text-decoration: underline;
        }}
        .container {{
            max-width: 1200px;
            margin: 2rem auto;
            padding: 0 2rem;
        }}
        .card {{
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 1rem;
        }}
        .card h2 {{
            margin-top: 0;
            color: #333;
        }}
        .url-item {{
            padding: 0.75rem;
            margin: 0.5rem 0;
            background: #f8f9fa;
            border-radius: 5px;
            display: flex;
            align-items: center;
        }}
        .url-item input[type="checkbox"] {{
            margin-right: 0.75rem;
            width: 18px;
            height: 18px;
            cursor: pointer;
        }}
        .url-item label {{
            cursor: pointer;
            flex: 1;
            word-break: break-all;
        }}
        .form-group {{
            margin: 1.5rem 0;
        }}
        .form-group label {{
            display: block;
            margin-bottom: 0.5rem;
            font-weight: 600;
            color: #333;
        }}
        .form-group textarea {{
            width: 100%;
            min-height: 150px;
            padding: 0.75rem;
            border: 1px solid #ddd;
            border-radius: 5px;
            font-family: monospace;
            font-size: 0.9rem;
            resize: vertical;
        }}
        .form-group textarea:focus {{
            outline: none;
            border-color: #667eea;
        }}
        .form-actions {{
            display: flex;
            gap: 1rem;
            margin-top: 1.5rem;
        }}
        .btn {{
            padding: 0.75rem 1.5rem;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            font-size: 1rem;
            text-decoration: none;
            display: inline-block;
            transition: background 0.3s;
        }}
        .btn-primary {{
            background: #667eea;
            color: white;
        }}
        .btn-primary:hover {{
            background: #5568d3;
        }}
        .btn-secondary {{
            background: #6c757d;
            color: white;
        }}
        .btn-secondary:hover {{
            background: #5a6268;
        }}
        .error-message {{
            background: #f8d7da;
            color: #721c24;
            padding: 1rem;
            border-radius: 5px;
            margin-bottom: 1rem;
            border: 1px solid #f5c6cb;
        }}
        .success-message {{
            background: #d4edda;
            color: #155724;
            padding: 1rem;
            border-radius: 5px;
            margin-bottom: 1rem;
            border: 1px solid #c3e6cb;
        }}
        .restart-form {{
            display: inline;
            margin-right: 0.5rem;
        }}
        .restart-btn {{
            background: #f39c12;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 5px;
            cursor: pointer;
            font-size: 0.9rem;
        }}
        .restart-btn:hover {{
            background: #e67e22;
        }}
        .logout-form {{
            display: inline;
        }}
        .logout-btn {{
            background: #e74c3c;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 5px;
            cursor: pointer;
            font-size: 0.9rem;
        }}
        .logout-btn:hover {{
            background: #c0392b;
        }}
        .help-text {{
            color: #666;
            font-size: 0.9rem;
            margin-top: 0.5rem;
        }}
        .empty-state {{
            text-align: center;
            padding: 2rem;
            color: #666;
            background: #f8f9fa;
            border-radius: 5px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Telegraf設定管理システム</h1>
    </div>
    <div class="nav">
        <a href="/admin">ダッシュボード</a>
        <a href="/admin/list">URL一覧</a>
        <a href="/admin/edit">URL編集</a>
        <form method="POST" action="/admin/telegraf/restart" class="restart-form" onsubmit="return confirm('Telegrafコンテナを再起動しますか？');">
            <button type="submit" class="restart-btn">Telegraf再起動</button>
        </form>
        <form method="POST" action="/auth/logout" class="logout-form">
            <button type="submit" class="logout-btn">ログアウト</button>
        </form>
    </div>
    <div class="container">
        <div class="card">
            <h2>URL編集</h2>
            {}
            <form method="POST" action="/admin/edit">
                <div class="form-group">
                    <label>現在のURL一覧（削除する場合はチェック）</label>
                    {}
                </div>
                
                <div class="form-group">
                    <label for="new_urls">新規URL追加（1行に1つのURLを入力）</label>
                    <textarea name="new_urls" id="new_urls" placeholder="https://example.com&#10;https://example.org"></textarea>
                    <div class="help-text">
                        ※ HTTP/HTTPSスキームを持つ有効なURLを入力してください<br>
                        ※ 複数のURLを追加する場合は、改行で区切ってください
                    </div>
                </div>
                
                <div class="form-actions">
                    <button type="submit" class="btn btn-primary">保存</button>
                    <a href="/admin/list" class="btn btn-secondary">キャンセル</a>
                </div>
            </form>
        </div>
    </div>
</body>
</html>
        "#,
        error_html,
        if url_list_html.is_empty() {
            r#"<div class="empty-state">現在、監視中のURLはありません。</div>"#
        } else {
            url_list_html
        }
    )
}
