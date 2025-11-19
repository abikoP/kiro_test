use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;

use crate::services::config_service::ConfigService;

/// URL一覧表示ページ
pub async fn index(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // ConfigServiceからURL一覧を取得
    let urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

    let mut url_rows = String::new();
    for (idx, url) in urls.iter().enumerate() {
        url_rows.push_str(&format!(
            r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
            </tr>
            "#,
            idx + 1,
            html_escape::encode_text(url)
        ));
    }

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>URL一覧 - Telegraf設定管理</title>
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
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }}
        th, td {{
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #e0e0e0;
        }}
        th {{
            background: #f8f9fa;
            font-weight: 600;
            color: #333;
        }}
        tr:hover {{
            background: #f8f9fa;
        }}
        .actions {{
            margin-top: 1.5rem;
        }}
        .btn {{
            display: inline-block;
            background: #667eea;
            color: white;
            padding: 0.75rem 1.5rem;
            border-radius: 5px;
            text-decoration: none;
            transition: background 0.3s;
        }}
        .btn:hover {{
            background: #5568d3;
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
        .empty-state {{
            text-align: center;
            padding: 3rem;
            color: #666;
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
            <h2>監視中のURL一覧</h2>
            {}
            <div class="actions">
                <a href="/admin/edit" class="btn">URLを編集する</a>
            </div>
        </div>
    </div>
</body>
</html>
        "#,
        if urls.is_empty() {
            r#"<div class="empty-state">
                <p>現在、監視中のURLはありません。</p>
            </div>"#.to_string()
        } else {
            format!(
                r#"<table>
                <thead>
                    <tr>
                        <th>No</th>
                        <th>URL</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>"#,
                url_rows
            )
        }
    );

    Ok(Html(html))
}
