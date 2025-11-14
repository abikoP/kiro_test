use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

/// セッションキー
pub const SESSION_USER_ID_KEY: &str = "user_id";

/// 認証が必要なルートを保護するミドルウェア
pub async fn require_auth(
    session: tower_sessions::Session,
    request: Request,
    next: Next,
) -> Response {
    // セッションからユーザIDを取得
    match session.get::<i32>(SESSION_USER_ID_KEY).await {
        Ok(Some(_user_id)) => {
            // 認証済み - リクエストを続行
            next.run(request).await
        }
        Ok(None) => {
            // 未認証 - ログインページへリダイレクト
            Redirect::to("/auth/login").into_response()
        }
        Err(_) => {
            // セッションエラー
            (StatusCode::INTERNAL_SERVER_ERROR, "セッションエラー").into_response()
        }
    }
}
