# 第4章: 認証機能の実装

## この章で学ぶこと

- 認証の基本概念
- パスワードハッシュ化（bcrypt）
- Userモデルの実装
- セッション管理
- 認証ミドルウェア
- ログイン・ログアウト機能

---

## 4.1 認証とは

### 認証の基本概念

**認証（Authentication）**は、ユーザが本人であることを確認するプロセスです。

**認証の流れ:**
```
1. ユーザがログインフォームにアクセス
2. ユーザ名とパスワードを入力
3. サーバがパスワードを検証
4. 認証成功 → セッションを作成
5. 以降のリクエストでセッションを確認
6. ログアウト → セッションを削除
```

### 認証 vs 認可

- **認証（Authentication）**: 「あなたは誰？」
- **認可（Authorization）**: 「あなたは何ができる？」

この章では、認証のみを扱います。

### セッションとは

**セッション**は、ユーザの状態を保持する仕組みです。

```
ブラウザ                    サーバ
   │                          │
   │  ログイン                │
   ├─────────────────────────>│
   │                          │ セッション作成
   │  Cookie（セッションID）  │
   │<─────────────────────────┤
   │                          │
   │  リクエスト + Cookie     │
   ├─────────────────────────>│
   │                          │ セッション確認
   │  レスポンス              │
   │<─────────────────────────┤
```

---

## 4.2 依存関係の追加

### 必要なクレート

`Cargo.toml`に追加（既に追加済み）：

```toml
[dependencies]
# 認証用
bcrypt = "0.15"

# セッション管理
tower-sessions = "0.13"

# 時間処理用
time = "0.3"
```

### 各クレートの役割

- **bcrypt**: パスワードのハッシュ化
- **tower-sessions**: セッション管理
- **time**: セッション有効期限の設定

---

## 4.3 Userモデルの実装

### モデルの役割

Userモデルは、ユーザ情報を扱います。

**主な機能:**
- ユーザの作成
- ユーザの検索
- パスワードの検証

### src/models/users.rsの作成

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

### コードの詳細解説

#### use文
```rust
pub use super::_entities::users::{ActiveModel, Column, Entity, Model};
```

- `super::_entities::users`: 親モジュールのエンティティ
- SeaORMが自動生成したエンティティを再エクスポート

#### create_with_password()
```rust
pub async fn create_with_password(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
) -> Result<Self> {
```

**引数:**
- `db`: データベース接続
- `username`: ユーザ名
- `password`: 平文パスワード

**戻り値:**
- `Result<Self>`: 作成されたユーザ

**処理の流れ:**

##### 1. パスワードのハッシュ化
```rust
let password_hash = hash_password(password)?;
```

- `hash_password()`: 後で定義する関数
- 平文パスワードをハッシュ化
- `?`: エラー時は早期リターン

##### 2. ActiveModelの作成
```rust
let user = ActiveModel {
    username: Set(username.to_string()),
    password_hash: Set(password_hash),
    ..Default::default()
};
```

- `ActiveModel`: SeaORMのデータ挿入用構造体
- `Set()`: フィールドに値を設定
- `..Default::default()`: 他のフィールドはデフォルト値

**なぜActiveModel?**
- SeaORMでは、データの挿入・更新に`ActiveModel`を使う
- `Model`は読み取り専用

##### 3. データベースへの挿入
```rust
let user = user.insert(db).await?;
Ok(user)
```

- `.insert()`: データベースに挿入
- `.await`: 非同期処理の完了を待つ
- 挿入されたユーザを返す

#### find_by_username()
```rust
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
```

**SeaORMのクエリビルダー:**

##### Entity::find()
```rust
Entity::find()
```

- クエリの起点
- `SELECT * FROM users`に相当

##### .filter()
```rust
.filter(Column::Username.eq(username))
```

- 条件を追加
- `WHERE username = ?`に相当
- `Column::Username`: 型安全なカラム指定

##### .one()
```rust
.one(db).await?
```

- 単一の結果を取得
- `Option<Model>`を返す
  - `Some(user)`: ユーザが見つかった
  - `None`: ユーザが見つからない

**なぜOption?**
- ユーザが存在しない場合もある
- エラーではなく、正常なケース

#### verify_password()
```rust
pub fn verify_password(&self, password: &str) -> Result<bool> {
    verify_password(password, &self.password_hash)
}
```

- `&self`: 自分自身への参照
- `self.password_hash`: データベースに保存されたハッシュ
- `password`: 検証する平文パスワード

**戻り値:**
- `Ok(true)`: パスワードが一致
- `Ok(false)`: パスワードが不一致
- `Err(...)`: 検証エラー

### パスワードハッシュ化の実装

#### hash_password()
```rust
fn hash_password(password: &str) -> Result<String> {
    use bcrypt::{hash, DEFAULT_COST};
    hash(password, DEFAULT_COST)
        .map_err(|e| Error::string(&format!("パスワードのハッシュ化に失敗しました: {}", e)))
}
```

**bcryptとは:**
- パスワードハッシュ化の業界標準
- レインボーテーブル攻撃に強い
- 計算コストを調整可能

**DEFAULT_COSTとは:**
- bcryptのコスト係数（デフォルトは12）
- 高いほど安全だが、処理時間が増加
- 2^12 = 4096回のハッシュ計算

**なぜ平文パスワードを保存しない？**
- データベースが漏洩しても安全
- 管理者でもパスワードを知ることができない

**ハッシュの例:**
```
平文: "password123"
ハッシュ: "$2b$12$3B6FO2nkUlEbLyZnqKr1iuR.fMKrgoZezyS9SDL3qW4tZ94/okRQG"
```

#### verify_password()
```rust
fn verify_password(password: &str, hash: &str) -> Result<bool> {
    use bcrypt::verify;
    verify(password, hash)
        .map_err(|e| Error::string(&format!("パスワードの検証に失敗しました: {}", e)))
}
```

**検証の仕組み:**
1. 平文パスワードを受け取る
2. ハッシュに含まれるソルトを使って再ハッシュ化
3. 結果を比較

**重要:** 同じパスワードでも、毎回異なるハッシュが生成されます（ソルトのため）。

---

## 4.4 セッション管理の設定

### セッションミドルウェアの追加

`src/app.rs`に`after_routes`フックを実装します。

```rust
async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
    // セッションストアを設定
    use tower_sessions::{MemoryStore, SessionManagerLayer};
    use time::Duration;

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // 開発環境ではfalse
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::hours(1)));

    Ok(router.layer(session_layer))
}
```

### コードの詳細解説

#### after_routesフック
```rust
async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
```

- Locoのライフサイクルフック
- ルーティング設定後に呼ばれる
- ミドルウェアを追加するのに使う

#### MemoryStore
```rust
let session_store = MemoryStore::default();
```

- セッションデータをメモリに保存
- 開発環境に適している
- **本番環境では、Redisなどを使う**

**なぜメモリストア？**
- 簡単にセットアップできる
- 開発・テストに十分

**本番環境の注意:**
- サーバ再起動でセッションが消える
- 複数サーバでセッションを共有できない

#### SessionManagerLayer
```rust
let session_layer = SessionManagerLayer::new(session_store)
    .with_secure(false)
    .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::hours(1)));
```

**with_secure(false):**
- HTTPSを必須にしない
- 開発環境では`false`
- **本番環境では`true`にする**

**with_expiry():**
- セッションの有効期限
- `OnInactivity`: 最後のアクセスから1時間
- アクセスがあれば期限が延長される

#### ミドルウェアの適用
```rust
Ok(router.layer(session_layer))
```

- `router.layer()`: ミドルウェアを追加
- すべてのルートに適用される

---

## 4.5 認証ミドルウェアの実装

### ミドルウェアの役割

認証ミドルウェアは、保護されたルートへのアクセスを制御します。

```
リクエスト
    ↓
認証ミドルウェア
    ├─ セッションあり → 次の処理へ
    └─ セッションなし → ログインページへリダイレクト
```

### src/middleware/auth.rsの作成

まず、ディレクトリを作成します。

```bash
mkdir -p src/middleware
```

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

### コードの詳細解説

#### セッションキー
```rust
pub const SESSION_USER_ID_KEY: &str = "user_id";
```

- セッションに保存するキー
- 定数にすることでタイポを防ぐ

#### ミドルウェア関数
```rust
pub async fn require_auth(
    session: tower_sessions::Session,
    request: Request,
    next: Next,
) -> Response {
```

**引数:**
- `session`: セッション情報
- `request`: HTTPリクエスト
- `next`: 次のハンドラー

**戻り値:**
- `Response`: HTTPレスポンス

#### セッションからユーザIDを取得
```rust
match session.get::<i32>(SESSION_USER_ID_KEY).await {
```

- `.get::<T>(key)`: セッションから値を取得
- `<i32>`: ユーザIDの型
- `.await`: 非同期処理

**戻り値:**
- `Ok(Some(user_id))`: ユーザIDがある
- `Ok(None)`: ユーザIDがない
- `Err(...)`: エラー

#### 認証済みの場合
```rust
Ok(Some(_user_id)) => {
    next.run(request).await
}
```

- `next.run()`: 次のハンドラーを呼び出す
- リクエスト処理を続行

**なぜ_user_id?**
- アンダースコアは「使わない」という意味
- ユーザIDは取得するが、この時点では使わない

#### 未認証の場合
```rust
Ok(None) => {
    Redirect::to("/auth/login").into_response()
}
```

- ログインページへリダイレクト
- `.into_response()`: `Response`型に変換

#### エラーの場合
```rust
Err(_) => {
    (StatusCode::INTERNAL_SERVER_ERROR, "セッションエラー").into_response()
}
```

- 500エラーを返す
- タプル`(StatusCode, &str)`も`IntoResponse`を実装

### src/middleware/mod.rsの作成

```rust
pub mod auth;
```

### src/lib.rsへの追加

```rust
pub mod middleware;
```

---

## 4.6 まとめ（前半）

この前半部分では、以下を学びました：

- ✅ 認証の基本概念
- ✅ bcryptによるパスワードハッシュ化
- ✅ Userモデルの実装
- ✅ SeaORMのクエリビルダー
- ✅ セッション管理の設定
- ✅ 認証ミドルウェアの実装

**重要なポイント:**
- パスワードは必ずハッシュ化して保存
- セッションでユーザの状態を管理
- ミドルウェアで保護されたルートを制御

---

## 次のセクション

後半では、以下を実装します：

- ログインフォームの表示
- ログイン処理
- ログアウト処理
- ルーティングの設定
- テスト用ユーザの作成

[第4章（後半）: ログイン・ログアウト機能](./chapter04_authentication_part2.md)に進みましょう！
