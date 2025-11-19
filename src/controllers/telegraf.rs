use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use loco_rs::prelude::*;
use std::process::Command;

/// Telegraf再起動処理
pub async fn restart(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // docker-compose restart telegraf を実行
    let output = Command::new("docker-compose")
        .arg("restart")
        .arg("telegraf")
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                // 成功時のレスポンス
                let html = generate_response_html(
                    true,
                    "Telegrafコンテナを再起動しました。",
                    "",
                );
                Ok(Html(html))
            } else {
                // コマンドは実行されたがエラーが発生
                let error_message = String::from_utf8_lossy(&result.stderr);
                let html = generate_response_html(
                    false,
                    "Telegrafコンテナの再起動に失敗しました。",
                    &error_message,
                );
                Ok(Html(html))
            }
        }
        Err(e) => {
            // コマンド実行自体が失敗
            let html = generate_response_html(
                false,
                "Telegrafコンテナの再起動に失敗しました。",
                &format!("エラー: {}", e),
            );
            Ok(Html(html))
        }
    }
}

/// レスポンスHTML生成
fn generate_response_html(success: bool, message: &str, detail: &str) -> String {
    let (status_class, status_icon) = if success {
        ("success", "✓")
    } else {
        ("error", "✗")
    };

    format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Telegraf再起動 - Telegraf設定管理</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background: #f5f5f5;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
        }}
        .result-container {{
            background: white;
            padding: 3rem;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
            text-align: center;
            max-width: 500px;
        }}
        .status-icon {{
            font-size: 4rem;
            margin-bottom: 1rem;
        }}
        .status-icon.success {{
            color: #27ae60;
        }}
        .status-icon.error {{
            color: #e74c3c;
        }}
        .message {{
            font-size: 1.2rem;
            margin-bottom: 1rem;
            color: #333;
        }}
        .detail {{
            background: #f8f9fa;
            padding: 1rem;
            border-radius: 5px;
            margin: 1rem 0;
            font-family: monospace;
            font-size: 0.9rem;
            color: #666;
            white-space: pre-wrap;
            word-break: break-all;
        }}
        .actions {{
            margin-top: 2rem;
        }}
        .btn {{
            display: inline-block;
            padding: 0.75rem 1.5rem;
            background: #667eea;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            transition: background 0.3s;
        }}
        .btn:hover {{
            background: #5568d3;
        }}
    </style>
</head>
<body>
    <div class="result-container">
        <div class="status-icon {}">{}</div>
        <div class="message">{}</div>
        {}
        <div class="actions">
            <a href="/admin" class="btn">管理画面に戻る</a>
        </div>
    </div>
</body>
</html>
        "#,
        status_class,
        status_icon,
        html_escape::encode_text(message),
        if !detail.is_empty() {
            format!(
                r#"<div class="detail">{}</div>"#,
                html_escape::encode_text(detail)
            )
        } else {
            String::new()
        }
    )
}
