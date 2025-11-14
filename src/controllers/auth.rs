use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::middleware::auth::SESSION_USER_ID_KEY;
use crate::models::users::Model as User;

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// ログインフォームを表示（エラーメッセージ付き）
pub fn login_form_with_error(error: Option<String>) -> Html<String> {
    let error_html = if let Some(err) = error {
        format!(r#"<div class="error">{}</div>"#, err)
    } else {
        String::new()
    };
    
    let html = format!(r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ログイン - Telegraf設定管理</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
        }}
        .login-container {{
            background: white;
            padding: 2rem;
            border-radius: 10px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.2);
            width: 100%;
            max-width: 400px;
        }}
        h1 {{
            color: #333;
            text-align: center;
            margin-bottom: 1.5rem;
        }}
        .form-group {{
            margin-bottom: 1rem;
        }}
        label {{
            display: block;
            margin-bottom: 0.5rem;
            color: #555;
            font-weight: 500;
        }}
        input[type="text"],
        input[type="password"] {{
            width: 100%;
            padding: 0.75rem;
            border: 1px solid #ddd;
            border-radius: 5px;
            font-size: 1rem;
            box-sizing: border-box;
        }}
        input[type="text"]:focus,
        input[type="password"]:focus {{
            outline: none;
            border-color: #667eea;
        }}
        button {{
            width: 100%;
            padding: 0.75rem;
            background: #667eea;
            color: white;
            border: none;
            border-radius: 5px;
            font-size: 1rem;
            font-weight: 600;
            cursor: pointer;
            transition: background 0.3s;
        }}
        button:hover {{
            background: #5568d3;
        }}
        .error {{
            background: #fee;
            color: #c33;
            padding: 0.75rem;
            border-radius: 5px;
            margin-bottom: 1rem;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class="login-container">
        <h1>ログイン</h1>
        {}
        <form method="POST" action="/auth/login">
            <div class="form-group">
                <label for="username">ユーザ名</label>
                <input type="text" id="username" name="username" required>
            </div>
            <div class="form-group">
                <label for="password">パスワード</label>
                <input type="password" id="password" name="password" required>
            </div>
            <button type="submit">ログイン</button>
        </form>
    </div>
</body>
</html>
    "#, error_html);
    Html(html)
}

/// ログインフォームを表示
pub async fn login_form() -> impl IntoResponse {
    login_form_with_error(None)
}

/// ログイン処理
pub async fn login(
    State(ctx): State<AppContext>,
    session: tower_sessions::Session,
    Form(form): Form<LoginForm>,
) -> Response {
    // ユーザを検索
    let user = match User::find_by_username(&ctx.db, &form.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return login_form_with_error(Some("ユーザ名またはパスワードが正しくありません".to_string())).into_response();
        }
        Err(_) => {
            return login_form_with_error(Some("エラーが発生しました".to_string())).into_response();
        }
    };

    // パスワードを検証
    match user.verify_password(&form.password) {
        Ok(true) => {},
        Ok(false) => {
            return login_form_with_error(Some("ユーザ名またはパスワードが正しくありません".to_string())).into_response();
        }
        Err(_) => {
            return login_form_with_error(Some("エラーが発生しました".to_string())).into_response();
        }
    }

    // セッションにユーザIDを保存
    if session.insert(SESSION_USER_ID_KEY, user.id).await.is_err() {
        return login_form_with_error(Some("セッションの保存に失敗しました".to_string())).into_response();
    }

    // 管理画面へリダイレクト
    Redirect::to("/admin").into_response()
}

/// ログアウト処理
pub async fn logout(session: tower_sessions::Session) -> impl IntoResponse {
    // セッションをクリア
    let _ = session.delete().await;

    // ログインページへリダイレクト
    Redirect::to("/auth/login")
}
