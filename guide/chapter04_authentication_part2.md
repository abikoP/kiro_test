# 第4章（後半）: ログイン・ログアウト機能の実装

## この章で学ぶこと

- コントローラーの実装
- HTMLフォームの処理
- ログイン処理の実装
- ログアウト処理の実装
- ルーティングの設定
- テスト用ユーザの作成

---

## 4.7 認証コントローラーの実装

### コントローラーとは

**コントローラー**は、HTTPリクエストを受け取り、レスポンスを返す役割を持ちます。

```
ブラウザ → ルーティング → コントローラー → サービス → データベース
                                ↓
ブラウザ ← レスポンス ← コントローラー
```

### src/controllers/auth.rsの作成

まず、ディレクトリを作成します。

```bash
mkdir -p src/controllers
```

### LoginFormの定義

```rust
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
```

### コードの詳細解説

#### use文
```rust
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
```

**Axumのエクストラクター:**
- `State`: アプリケーションコンテキストを取得
- `Form`: POSTされたフォームデータを取得
- `Html`: HTMLレスポンスを返す
- `Redirect`: リダイレクトレスポンスを返す

#### LoginForm構造体
```rust
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}
```

- `Deserialize`: フォームデータを自動変換
- フィールド名がHTMLのname属性と一致する必要がある

**HTMLフォームとの対応:**
```html
<input type="text" name="username">     <!-- username -->
<input type="password" name="password"> <!-- password -->
```

---

## 4.8 ログインフォームの表示

### login_form_with_error()の実装

```rust
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
```

### コードの詳細解説

#### エラーHTMLの生成
```rust
let error_html = if let Some(err) = error {
    format!(r#"<div class="error">{}</div>"#, err)
} else {
    String::new()
};
```

- `if let Some(err) = error`: エラーがある場合
- `format!()`: エラーメッセージをHTMLに埋め込む
- `String::new()`: エラーがない場合は空文字列

#### format!マクロ
```rust
let html = format!(r#"..."#, error_html);
```

- `r#"..."#`: Raw文字列リテラル
- `{}`でエスケープ不要
- `{{}}`でCSSの`{}`を表現

**なぜRaw文字列？**
```rust
// 通常の文字列: エスケープが必要
"body { color: red; }"  // エラー！

// Raw文字列: エスケープ不要
r#"body { color: red; }"#  // OK
```

#### HTMLフォーム
```html
<form method="POST" action="/auth/login">
```

- `method="POST"`: POSTリクエスト
- `action="/auth/login"`: 送信先URL

```html
<input type="text" id="username" name="username" required>
```

- `name="username"`: フォームデータのキー
- `required`: 必須入力

### login_form()の実装

```rust
/// ログインフォームを表示
pub async fn login_form() -> impl IntoResponse {
    login_form_with_error(None)
}
```

- エラーなしでログインフォームを表示
- `impl IntoResponse`: 戻り値の型を抽象化

---

## 4.9 ログイン処理の実装

### login()の実装

```rust
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
            return login_form_with_error(
                Some("ユーザ名またはパスワードが正しくありません".to_string())
            ).into_response();
        }
        Err(_) => {
            return login_form_with_error(
                Some("エラーが発生しました".to_string())
            ).into_response();
        }
    };

    // パスワードを検証
    match user.verify_password(&form.password) {
        Ok(true) => {},
        Ok(false) => {
            return login_form_with_error(
                Some("ユーザ名またはパスワードが正しくありません".to_string())
            ).into_response();
        }
        Err(_) => {
            return login_form_with_error(
                Some("エラーが発生しました".to_string())
            ).into_response();
        }
    }

    // セッションにユーザIDを保存
    if session.insert(SESSION_USER_ID_KEY, user.id).await.is_err() {
        return login_form_with_error(
            Some("セッションの保存に失敗しました".to_string())
        ).into_response();
    }

    // 管理画面へリダイレクト
    Redirect::to("/admin").into_response()
}
```

### コードの詳細解説

#### 関数の引数
```rust
pub async fn login(
    State(ctx): State<AppContext>,
    session: tower_sessions::Session,
    Form(form): Form<LoginForm>,
) -> Response {
```

**Axumのエクストラクター:**
- `State(ctx)`: アプリケーションコンテキスト
  - `ctx.db`: データベース接続
- `session`: セッション情報
- `Form(form)`: POSTされたフォームデータ

**戻り値:**
- `Response`: HTTPレスポンス（統一型）

#### ステップ1: ユーザを検索
```rust
let user = match User::find_by_username(&ctx.db, &form.username).await {
    Ok(Some(user)) => user,
    Ok(None) => {
        return login_form_with_error(
            Some("ユーザ名またはパスワードが正しくありません".to_string())
        ).into_response();
    }
    Err(_) => {
        return login_form_with_error(
            Some("エラーが発生しました".to_string())
        ).into_response();
    }
};
```

**パターンマッチング:**
- `Ok(Some(user))`: ユーザが見つかった
- `Ok(None)`: ユーザが見つからない
- `Err(_)`: データベースエラー

**セキュリティのポイント:**
```rust
"ユーザ名またはパスワードが正しくありません"
```

- ユーザ名とパスワードのエラーを区別しない
- どちらが間違っているか教えない
- アカウント列挙攻撃を防ぐ

**悪い例:**
```rust
// ❌ セキュリティリスク
"ユーザ名が存在しません"  // ユーザ名の存在がわかる
"パスワードが間違っています"  // ユーザ名は正しいとわかる
```

#### ステップ2: パスワードを検証
```rust
match user.verify_password(&form.password) {
    Ok(true) => {},
    Ok(false) => {
        return login_form_with_error(
            Some("ユーザ名またはパスワードが正しくありません".to_string())
        ).into_response();
    }
    Err(_) => {
        return login_form_with_error(
            Some("エラーが発生しました".to_string())
        ).into_response();
    }
}
```

**パターンマッチング:**
- `Ok(true)`: パスワードが一致
- `Ok(false)`: パスワードが不一致
- `Err(_)`: 検証エラー

**Ok(true)の処理:**
```rust
Ok(true) => {},
```

- 何もしない（次の処理に進む）
- `{}`は空のブロック

#### ステップ3: セッションに保存
```rust
if session.insert(SESSION_USER_ID_KEY, user.id).await.is_err() {
    return login_form_with_error(
        Some("セッションの保存に失敗しました".to_string())
    ).into_response();
}
```

- `session.insert()`: セッションに値を保存
- `SESSION_USER_ID_KEY`: `"user_id"`
- `user.id`: ユーザID
- `.is_err()`: エラーかどうかをチェック

**セッションの内容:**
```
{
  "user_id": 1
}
```

#### ステップ4: リダイレクト
```rust
Redirect::to("/admin").into_response()
```

- 管理画面へリダイレクト
- `.into_response()`: `Response`型に変換

---

## 4.10 ログアウト処理の実装

### logout()の実装

```rust
/// ログアウト処理
pub async fn logout(session: tower_sessions::Session) -> impl IntoResponse {
    // セッションをクリア
    let _ = session.delete().await;

    // ログインページへリダイレクト
    Redirect::to("/auth/login")
}
```

### コードの詳細解説

#### セッションの削除
```rust
let _ = session.delete().await;
```

- `session.delete()`: セッション全体を削除
- `let _ = ...`: 戻り値を無視
- エラーは無視（ログアウトは失敗しても問題ない）

**なぜエラーを無視？**
- ログアウトは必ず成功させたい
- セッションがなくても問題ない
- ユーザ体験を優先

#### リダイレクト
```rust
Redirect::to("/auth/login")
```

- ログインページへリダイレクト
- `.into_response()`は省略可能（型推論）

---

## 4.11 モジュールの設定

### src/controllers/mod.rsの作成

```rust
pub mod auth;
```

### src/lib.rsへの追加

```rust
pub mod controllers;
```

---

## 4.12 ルーティングの設定

### src/app.rsのroutes()を更新

```rust
fn routes(_ctx: &AppContext) -> AppRoutes {
    use axum::routing::{get, post};
    use crate::controllers::auth;
    use crate::middleware::auth::require_auth;

    AppRoutes::with_default_routes()
        // 公開ルート（認証不要）
        .add("/auth/login", get(auth::login_form))
        .add("/auth/login", post(auth::login))
        
        // 保護されたルート（認証必須）
        .add("/auth/logout", post(auth::logout))
        .layer(axum::middleware::from_fn(require_auth))
}
```

### コードの詳細解説

#### ルーティングの追加
```rust
.add("/auth/login", get(auth::login_form))
.add("/auth/login", post(auth::login))
```

- 同じパス`/auth/login`に2つのルート
- `GET`: ログインフォームを表示
- `POST`: ログイン処理

#### ミドルウェアの適用
```rust
.add("/auth/logout", post(auth::logout))
.layer(axum::middleware::from_fn(require_auth))
```

- `/auth/logout`は認証必須
- `require_auth`ミドルウェアを適用

**注意:** ミドルウェアは、それより前に定義されたルートに適用されます。

---

## 4.13 テスト用ユーザの作成

### src/bin/seed_user.rsの作成

開発時にテスト用ユーザを簡単に作成できるツールを作ります。

```rust
use kiro_test::models::users::Model as User;
use sea_orm::{Database, DatabaseConnection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // データベースに接続
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://db.sqlite?mode=rwc".to_string());
    let db: DatabaseConnection = Database::connect(&db_url).await?;

    // テスト用ユーザを作成
    let username = "admin";
    let password = "admin123";

    // 既存のユーザをチェック
    match User::find_by_username(&db, username).await {
        Ok(Some(user)) => {
            println!("ℹ️  テストユーザ '{}' は既に存在します (ID: {})", username, user.id);
            println!("   パスワード: {}", password);
        }
        Ok(None) => {
            // ユーザを作成
            match User::create_with_password(&db, username, password).await {
                Ok(user) => {
                    println!("✅ テストユーザを作成しました:");
                    println!("   ユーザ名: {}", user.username);
                    println!("   パスワード: {}", password);
                    println!("   ID: {}", user.id);
                }
                Err(e) => {
                    println!("❌ ユーザの作成に失敗しました: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ ユーザの検索に失敗しました: {}", e);
        }
    }

    Ok(())
}
```

### コードの詳細解説

#### #[tokio::main]
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
```

- 非同期ランタイムを初期化
- `Box<dyn std::error::Error>`: 任意のエラー型

#### 環境変数の取得
```rust
let db_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "sqlite://db.sqlite?mode=rwc".to_string());
```

- `std::env::var()`: 環境変数を取得
- `.unwrap_or_else()`: 環境変数がない場合のデフォルト値

#### 重複チェック
```rust
match User::find_by_username(&db, username).await {
    Ok(Some(user)) => {
        println!("ℹ️  テストユーザ '{}' は既に存在します", username);
    }
    Ok(None) => {
        // ユーザを作成
    }
}
```

- 既存ユーザがいる場合は作成しない
- 重複エラーを防ぐ

### Cargo.tomlへの追加

```toml
[[bin]]
name = "seed_user"
path = "src/bin/seed_user.rs"
```

### 実行方法

```bash
cargo run --bin seed_user
```

**出力例:**
```
✅ テストユーザを作成しました:
   ユーザ名: admin
   パスワード: admin123
   ID: 1
```

---

## 4.14 動作確認

### 1. サーバーの起動

```bash
cargo run -- start
```

### 2. テストユーザの作成

別のターミナルで：

```bash
cargo run --bin seed_user
```

### 3. ログインページへアクセス

ブラウザで `http://localhost:3000/auth/login` を開く

### 4. ログイン

- ユーザ名: `admin`
- パスワード: `admin123`

### 5. 管理画面の確認

ログイン成功後、`http://localhost:3000/admin` にリダイレクトされます。

### 6. ログアウト

管理画面の「ログアウト」ボタンをクリック

---

## 4.15 トラブルシューティング

### エラー: セッションが取得できない

```
Can't extract session. Is `SessionManagerLayer` enabled?
```

**原因:**
- セッションミドルウェアが設定されていない

**解決方法:**
- `src/app.rs`の`after_routes`フックを確認
- `SessionManagerLayer`が追加されているか確認

### エラー: パスワードが一致しない

**確認事項:**
1. テストユーザが作成されているか
2. パスワードが正しいか（`admin123`）
3. bcryptが正しくインストールされているか

### エラー: リダイレクトループ

**原因:**
- ミドルウェアの適用順序が間違っている

**解決方法:**
- ログインページは認証不要にする
- ミドルウェアの適用範囲を確認

---

## 4.16 まとめ

この章では、以下を学びました：

- ✅ 認証の基本概念
- ✅ bcryptによるパスワードハッシュ化
- ✅ Userモデルの実装
- ✅ SeaORMのクエリビルダー
- ✅ セッション管理
- ✅ 認証ミドルウェア
- ✅ ログイン・ログアウト機能
- ✅ HTMLフォームの処理
- ✅ ルーティングの設定

**重要なポイント:**
- パスワードは必ずハッシュ化
- セッションでユーザ状態を管理
- エラーメッセージでユーザ名の存在を教えない
- ミドルウェアで保護されたルートを制御

**セキュリティのベストプラクティス:**
- パスワードの平文保存は絶対にしない
- bcryptで適切にハッシュ化
- エラーメッセージは慎重に設計
- セッションの有効期限を設定
- 本番環境ではHTTPSを使用

---

## 次のステップ

次の章では、**公開設定閲覧機能**を実装します。認証不要のページの作り方を学びます。

[第5章: 公開設定閲覧機能の実装](./chapter05_public_view.md)に進みましょう！
