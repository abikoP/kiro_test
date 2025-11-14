use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;

use crate::services::config_service::ConfigService;

/// 管理画面トップページ
pub async fn index(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // ダッシュボード情報を取得
    let url_count = match ConfigService::get_urls() {
        Ok(urls) => urls.len(),
        Err(_) => 0,
    };
    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>管理画面 - Telegraf設定管理</title>
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
        .dashboard-stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin: 1.5rem 0;
        }}
        .stat-box {{
            background: #f8f9fa;
            padding: 1.5rem;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }}
        .stat-box h3 {{
            margin: 0 0 0.5rem 0;
            color: #666;
            font-size: 0.9rem;
            font-weight: 500;
        }}
        .stat-box .value {{
            font-size: 2rem;
            font-weight: 700;
            color: #333;
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
        .quick-actions {{
            margin-top: 1.5rem;
        }}
        .quick-actions a {{
            display: inline-block;
            background: #667eea;
            color: white;
            padding: 0.75rem 1.5rem;
            border-radius: 5px;
            text-decoration: none;
            margin-right: 1rem;
            margin-bottom: 0.5rem;
            transition: background 0.3s;
        }}
        .quick-actions a:hover {{
            background: #5568d3;
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
        <form method="POST" action="/auth/logout" class="logout-form">
            <button type="submit" class="logout-btn">ログアウト</button>
        </form>
    </div>
    <div class="container">
        <div class="card">
            <h2>ダッシュボード</h2>
            <div class="dashboard-stats">
                <div class="stat-box">
                    <h3>監視中のURL数</h3>
                    <div class="value">{}</div>
                </div>
            </div>
        </div>
        <div class="card">
            <h2>管理画面へようこそ</h2>
            <p>このシステムでは、Telegrafの設定ファイル内のHTTP監視URLを管理できます。</p>
            <div class="quick-actions">
                <a href="/admin/list">URL一覧を見る</a>
                <a href="/admin/edit">URLを編集する</a>
            </div>
        </div>
    </div>
</body>
</html>
        "#,
        url_count
    );
    Ok(Html(html))
}
