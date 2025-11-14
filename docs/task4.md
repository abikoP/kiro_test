# タスク4: 認証機能の実装

## 概要

このタスクでは、Loco フレームワークを使用してWebアプリケーションに認証機能を実装しました。具体的には以下の3つのサブタスクを完了しました：

1. Userモデルのセットアップ
2. 認証ミドルウェアの設定
3. ログイン・ログアウト機能の実装

## 実装の全体像

```
認証フロー:
1. ユーザがログインフォームにアクセス (/auth/login)
2. ユーザ名とパスワードを入力して送信
3. サーバ側でパスワードを検証
4. 認証成功時、セッションにユーザIDを保存
5. 管理画面 (/admin) へリダイレクト
6. 以降のリクエストでは、ミドルウェアがセッションを確認
7. ログアウト時、セッションをクリア
```

---

## サブタスク 4.1: Userモデルのセットアップ

### 実装内容

既存のUserエンティティに認証関連のメソッドを追加しました。

### コード解説

#### `src/models/users.rs`

```rust
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub use super::_entities::users::{ActiveModel, Column, Entity, Model};

impl Model {
    /// パスワードをハッシュ化してユーザを作成
    pub async fn create_with_password(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> Result<Self> {
        let password_hash = hash_password(password)?;
        
        let user = ActiveModel {
            username: Set(username.to_string()),
            password_hash: Set(password_hash),
            ..Default::default()
        };
        
        let user = user.insert(db).await?;
        Ok(user)
    }
    
    /// ユーザ名でユーザを検索
    pub async fn find_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<Self>> {
        let user = Entity::find()
            .filter(Column::Username.eq(username))
            .one(db)
            .await?;
        Ok(user)
    }
    
    /// パスワードを検証
    pub fn verify_password(&self, password: &str) -> Result<bool> {
        verify_password(password, &self.password_hash)
    }
}
```

**ポイント:**

1. **`create_with_password`メソッド**
   - 平文パスワードを受け取り、bcryptでハッシュ化してからDBに保存
   - `ActiveModel`を使用してデータを挿入（SeaORMのパターン）
   - `Set()`でフィールドに値を設定
   - `..Default::default()`で他のフィールド（created_at, updated_atなど）はデフォルト値を使用

2. **`find_by_username`メソッド**
   - SeaORMのクエリビルダーを使用
   - `Entity::find()`でクエリを開始
   - `.filter()`で条件を指定
   - `.one()`で単一の結果を取得（`Option<Model>`を返す）

3. **`verify_password`メソッド**
   - 平文パスワードとハッシュ化されたパスワードを比較
   - bcryptの`verify`関数を使用

#### パスワードハッシュ化の実装

```rust
/// パスワードをbcryptでハッシュ化
fn hash_password(password: &str) -> Result<String> {
    use bcrypt::{hash, DEFAULT_COST};
    hash(password, DEFAULT_COST)
        .map_err(|e| Error::string(&format!("パスワードのハッシュ化に失敗しました: {}", e)))
}

/// パスワードを検証
fn verify_password(password: &str, hash: &str) -> Result<bool> {
    use bcrypt::verify;
    verify(password, hash)
        .map_err(|e| Error::string(&format!("パスワードの検証に失敗しました: {}", e)))
}
```

**ポイント:**

- **bcrypt**: パスワードハッシュ化の業界標準アルゴリズム
- **DEFAULT_COST**: bcryptのコスト係数（デフォルトは12）。高いほど安全だが処理時間が増加
- **エラーハンドリング**: `map_err`でbcryptのエラーをLocoの`Error`型に変換

### 依存関係の追加

`Cargo.toml`に以下を追加：

```toml
# 認証用
bcrypt = "0.15"
```

---

## サブタスク 4.2: 認証ミドルウェアの設定

### 実装内容

認証が必要なルートを保護するミドルウェアを実装しました。

### コード解説

#### `src/middleware/auth.rs`

```rust
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
```

**ポイント:**

1. **Axumのミドルウェアパターン**
   - `Request`と`Next`を引数に取る
   - `next.run(request).await`でチェーンの次のハンドラーを呼び出す
   - 認証失敗時は`next`を呼ばずにレスポンスを返す

2. **セッション管理**
   - `tower_sessions::Session`を使用
   - `.get::<T>(key)`でセッションから値を取得
   - 型パラメータ`<i32>`でユーザIDの型を指定

3. **レスポンスの返し方**
   - `Redirect::to()`でリダイレクトレスポンスを作成
   - `.into_response()`で`Response`型に変換
   - タプル`(StatusCode, &str)`も`IntoResponse`トレイトを実装しているため、そのまま返せる

#### ミドルウェアの適用（`src/app.rs`）

```rust
fn routes(ctx: &AppContext) -> AppRoutes {
    use loco_rs::controller::Routes;
    
    // 公開ルート
    let public_routes = Routes::new()
        .add("/conf", get(config::show))
        .add("/auth/login", get(auth::login_form))
        .add("/auth/login", post(auth::login));

    // 認証が必要なルート（ミドルウェアを適用）
    let protected_routes = Routes::new()
        .add("/admin", get(admin::index))
        .add("/auth/logout", post(auth::logout))
        .layer(axum::middleware::from_fn_with_state(
            ctx.clone(),
            require_auth,
        ));

    AppRoutes::with_default_routes()
        .add_route(public_routes)
        .add_route(protected_routes)
}
```

**ポイント:**

1. **Locoのルーティング**
   - `Routes::new()`で新しいルートグループを作成
   - `.add(path, handler)`でルートを追加
   - `.layer()`でミドルウェアを適用

2. **ミドルウェアの適用方法**
   - `axum::middleware::from_fn_with_state()`を使用
   - `ctx.clone()`でアプリケーションコンテキストを渡す
   - `require_auth`がミドルウェア関数

3. **ルートの分離**
   - 公開ルート（`public_routes`）: 認証不要
   - 保護されたルート（`protected_routes`）: 認証必須
   - 明確に分離することでセキュリティを向上

---

## サブタスク 4.3: ログイン・ログアウト機能の実装

### 実装内容

ログインフォームの表示、ログイン処理、ログアウト処理を実装しました。

### コード解説

#### `src/controllers/auth.rs`

##### 1. ログインフォームの表示

```rust
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
    <title>ログイン - Telegraf設定管理</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            /* ... 省略 ... */
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
```

**ポイント:**

1. **フォームデータの構造体**
   - `#[derive(Deserialize)]`でフォームデータを自動的にデシリアライズ
   - フィールド名がHTMLのinput要素のname属性と一致する必要がある

2. **HTMLテンプレート**
   - `format!`マクロでHTMLを生成
   - `{{}}`でエスケープ（CSSの`{}`と区別するため）
   - `{}`でRustの変数を埋め込み

3. **エラー表示**
   - `Option<String>`でエラーメッセージを受け取る
   - エラーがある場合のみエラーHTMLを生成

##### 2. ログイン処理

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
    if let Err(_) = session.insert(SESSION_USER_ID_KEY, user.id).await {
        return login_form_with_error(
            Some("セッションの保存に失敗しました".to_string())
        ).into_response();
    }

    // 管理画面へリダイレクト
    Redirect::to("/admin").into_response()
}
```

**ポイント:**

1. **Axumのエクストラクター**
   - `State(ctx)`: アプリケーションコンテキストを取得
   - `session`: セッション情報を取得
   - `Form(form)`: POSTされたフォームデータを取得

2. **エラーハンドリング**
   - `match`式で各ステップのエラーを処理
   - エラー時はログインフォームを再表示（エラーメッセージ付き）
   - セキュリティのため、ユーザ名とパスワードのエラーメッセージを同じにする

3. **セッションへの保存**
   - `session.insert(key, value)`でセッションに値を保存
   - 非同期処理なので`.await`が必要

4. **レスポンスの統一**
   - 戻り値の型を`Response`に統一
   - 各分岐で`.into_response()`を呼び出す

##### 3. ログアウト処理

```rust
/// ログアウト処理
pub async fn logout(session: tower_sessions::Session) -> impl IntoResponse {
    // セッションをクリア
    let _ = session.delete().await;

    // ログインページへリダイレクト
    Redirect::to("/auth/login")
}
```

**ポイント:**

1. **セッションの削除**
   - `session.delete()`でセッション全体を削除
   - エラーは無視（`let _ = ...`）

2. **シンプルな実装**
   - ログアウトは失敗しても問題ないため、エラーハンドリングは最小限

---

## エラー型の変換

### ConfigErrorからLocoのErrorへの変換

`src/services/config_service.rs`で独自のエラー型を定義している場合、Locoの`Error`型に変換する必要があります。

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("設定ファイルが見つかりません: {0}")]
    FileNotFound(String),
    // ... 他のエラー ...
}

// LocoのErrorへの変換を実装
impl From<ConfigError> for loco_rs::Error {
    fn from(err: ConfigError) -> Self {
        loco_rs::Error::string(&err.to_string())
    }
}
```

**ポイント:**

1. **`From`トレイトの実装**
   - `?`演算子で自動的にエラー変換が行われる
   - `thiserror`クレートで`#[error(...)]`を使用すると`to_string()`で人間が読めるメッセージが得られる

2. **Locoのエラー型**
   - `Error::string()`で文字列からエラーを作成
   - 他にも`Error::Unauthorized()`、`Error::NotFound()`などがある

---

## テスト用ユーザの作成

開発時にテスト用ユーザを簡単に作成できるよう、専用のバイナリを作成しました。

### `src/bin/seed_user.rs`

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
    let password = "password";

    match User::create_with_password(&db, username, password).await {
        Ok(user) => {
            println!("✅ ユーザを作成しました:");
            println!("   ユーザ名: {}", user.username);
            println!("   ID: {}", user.id);
        }
        Err(e) => {
            println!("❌ ユーザの作成に失敗しました: {}", e);
        }
    }

    Ok(())
}
```

### `Cargo.toml`への追加

```toml
[[bin]]
name = "seed_user"
path = "src/bin/seed_user.rs"
```

### 実行方法

```bash
cargo run --bin seed_user
```

**ポイント:**

1. **複数のバイナリ**
   - `[[bin]]`セクションで追加のバイナリを定義
   - メインアプリケーションとは別に実行可能

2. **環境変数の使用**
   - `std::env::var()`で環境変数を取得
   - `.unwrap_or_else()`でデフォルト値を設定

---

## 学習ポイント

### 1. Locoフレームワークの理解

- **AppContext**: データベース接続などのアプリケーション全体で共有される状態
- **Routes**: ルーティングの定義方法
- **ミドルウェア**: リクエスト処理の前後に共通処理を挟む方法

### 2. Axumの基礎

- **エクストラクター**: `State`, `Form`, `session`などでリクエストから情報を取得
- **レスポンス**: `IntoResponse`トレイトで様々な型をレスポンスに変換
- **ミドルウェア**: `from_fn_with_state`でカスタムミドルウェアを作成

### 3. SeaORMの使い方

- **ActiveModel**: データの挿入・更新に使用
- **Entity**: クエリの起点
- **クエリビルダー**: `.find()`, `.filter()`, `.one()`などのメソッドチェーン

### 4. セキュリティのベストプラクティス

- **パスワードのハッシュ化**: 平文パスワードは絶対に保存しない
- **bcrypt**: 適切なコスト係数で安全にハッシュ化
- **エラーメッセージ**: ユーザ名とパスワードのエラーを区別しない（情報漏洩防止）
- **セッション管理**: 認証状態をサーバ側で管理

### 5. Rustの非同期プログラミング

- **async/await**: 非同期関数の定義と呼び出し
- **Result型**: エラーハンドリングの基本
- **match式**: パターンマッチングでエラーを処理

---

## 動作確認

### 1. サーバーの起動

```bash
cargo run -- start
```

### 2. ログインページへアクセス

ブラウザで `http://localhost:5150/auth/login` を開く

### 3. ログイン

- ユーザ名: `admin`
- パスワード: `password`

### 4. 管理画面の確認

ログイン成功後、`http://localhost:5150/admin` にリダイレクトされる

### 5. ログアウト

管理画面の「ログアウト」ボタンをクリック

---

## まとめ

このタスクでは、Webアプリケーションの基本的な認証機能を実装しました。以下の技術を学びました：

- **Locoフレームワーク**: Rustの高速なWebフレームワーク
- **Axum**: 型安全なWebアプリケーション構築
- **SeaORM**: Rustの非同期ORM
- **bcrypt**: パスワードの安全なハッシュ化
- **tower-sessions**: セッション管理
- **ミドルウェアパターン**: 横断的関心事の実装

これらの知識は、他のWebアプリケーション開発にも応用できます。
