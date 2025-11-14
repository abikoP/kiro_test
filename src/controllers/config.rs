use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;

use crate::services::config_service::ConfigService;

/// 設定情報を表示（公開）
pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // 設定ファイルからURLを取得
    let urls = ConfigService::get_urls()?;

    let mut url_list = String::new();
    for (i, url) in urls.iter().enumerate() {
        url_list.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            i + 1,
            url
        ));
    }

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Telegraf設定情報</title>
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
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }}
        th, td {{
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background: #f8f9fa;
            font-weight: 600;
            color: #333;
        }}
        .footer {{
            text-align: center;
            margin-top: 2rem;
            color: #666;
        }}
        .footer a {{
            color: #667eea;
            text-decoration: none;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Telegraf設定情報</h1>
    </div>
    <div class="container">
        <div class="card">
            <h2>監視中のURL一覧</h2>
            <table>
                <thead>
                    <tr>
                        <th>No.</th>
                        <th>URL</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>
        <div class="footer">
            <p><a href="/admin">管理画面へ</a></p>
        </div>
    </div>
</body>
</html>
        "#,
        url_list
    );

    Ok(Html(html))
}
