# 第10章: テストと品質保証

## この章で学ぶこと

- 統合テストの書き方
- テスト環境のセットアップ
- セッション管理の実装
- Clippyによるコード品質チェック
- テストデータの準備

---

## 10.1 なぜテストが重要なのか

### テストの目的

**テスト**は、コードが期待通りに動作することを確認する作業です。

**テストのメリット：**
- ✅ バグの早期発見
- ✅ リファクタリングの安全性
- ✅ ドキュメントとしての役割
- ✅ 品質の保証
- ✅ 自信を持ってデプロイできる

### テストの種類

**1. 単体テスト（Unit Test）**
- 個別の関数やメソッドをテスト
- 高速で実行できる
- 例：`UrlValidationService::validate_url()`のテスト

**2. 統合テスト（Integration Test）**
- 複数のコンポーネントを組み合わせてテスト
- 実際の使用シナリオに近い
- 例：ログインからURL編集までの一連の流れ

**3. E2Eテスト（End-to-End Test）**
- ブラウザを使った実際のユーザー操作をテスト
- 最も現実に近いが、実行が遅い

この章では、**統合テスト**に焦点を当てます。

---

## 10.2 テスト環境のセットアップ

### テスト用設定ファイル

`config/test.yaml`を作成します：

```yaml
server:
  port: 3001
  host: 127.0.0.1

database:
  uri: "sqlite::memory:"  # インメモリデータベース
  enable_logging: false
  auto_migrate: true
  min_connections: 1
  max_connections: 1
  connect_timeout: 500
  idle_timeout: 500

auth:
  jwt:
    secret: test-secret-key
    expiration: 86400

logger:
  enable: false
  level: error
  format: compact
```

### 設定のポイント

**`uri: "sqlite::memory:"`**
- インメモリデータベースを使用
- テストごとにクリーンな状態から開始
- ファイルI/Oがないため高速

**注意：引用符が必要**
```yaml
# 正しい
uri: "sqlite::memory:"

# 間違い（YAMLパースエラー）
uri: sqlite::memory:
```

理由：コロン（`:`）がYAMLのキーと値の区切りと誤認識されるため。

**`enable_logging: false`**
- テスト出力をクリーンに保つ
- エラーレベルのみ記録

**`port: 3001`**
- 開発環境（3000）と競合しない


### 依存関係の追加

`Cargo.toml`にテスト用の依存関係を追加します：

```toml
[dev-dependencies]
serial_test = "3"  # テストの順次実行
loco-rs = { version = "0.13", features = ["testing"] }
```

**`[dev-dependencies]`**
- テスト時のみ使用される依存関係
- 本番ビルドには含まれない

**`serial_test`**
- テストを順次実行するためのクレート
- データベースの状態を分離するために必要

---

## 10.3 セッション管理の実装

### なぜセッション管理が必要か

**セッション**は、ユーザーのログイン状態を保持する仕組みです。

**HTTPの特性：**
- HTTPはステートレス（状態を持たない）
- リクエストごとに独立している
- ログイン状態を保持できない

**セッションの仕組み：**
```
1. ユーザーがログイン
2. サーバーがセッションIDを生成
3. セッションIDをCookieでブラウザに送信
4. ブラウザは以降のリクエストでセッションIDを送信
5. サーバーがセッションIDからユーザーを特定
```

### セッションミドルウェアの追加

`src/app.rs`に`after_routes`フックを実装します：

```rust
use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    environment::Environment,
    Result,
};

pub struct App;

#[async_trait]
impl Hooks for App {
    // ... 既存のメソッド ...
    
    async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
        use tower_sessions::{MemoryStore, SessionManagerLayer};
        use time::Duration;

        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false) // 開発環境ではfalse
            .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::hours(1)));

        Ok(router.layer(session_layer))
    }
}
```

### コードの解説

**`after_routes`フック**
- ルーティング設定後に呼ばれる
- ミドルウェアを追加するタイミング

**`MemoryStore`**
- インメモリセッションストア
- 開発・テスト環境に適している
- サーバー再起動でセッションが消える

**本番環境では：**
```rust
// Redisなどの永続化ストアを使用
use tower_sessions_redis_store::{RedisStore, RedisPool};

let pool = RedisPool::connect("redis://127.0.0.1/").await?;
let session_store = RedisStore::new(pool);
```

**`with_secure(false)`**
- HTTPS必須かどうか
- 開発環境：`false`（HTTPでも動作）
- 本番環境：`true`（HTTPS必須）

**`with_expiry`**
- セッションの有効期限
- `OnInactivity(Duration::hours(1))`：1時間操作がないと期限切れ

**重要：`time::Duration`を使用**
```rust
use time::Duration;  // ✅ 正しい

// ❌ 間違い
use std::time::Duration;  // tower-sessionsは受け付けない
```

### 依存関係の追加

```toml
[dependencies]
tower-sessions = "0.12"
time = "0.3"
```

---

## 10.4 テストユーザーの作成

### テストデータの準備

テストを実行する前に、テストユーザーを作成する必要があります。

`src/bin/seed_user.rs`を作成：

```rust
use kiro_test::app::App;
use loco_rs::cli;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    // アプリケーションを初期化
    let ctx = cli::boot::<App, Migrator>().await?;
    
    // テスト用ユーザを作成
    let username = "admin";
    let password = "admin123";
    
    use kiro_test::models::users::Model as User;
    
    // 既存のユーザをチェック
    match User::find_by_username(&ctx.db, username).await {
        Ok(Some(user)) => {
            println!("ℹ️  テストユーザ '{}' は既に存在します (ID: {})", username, user.id);
            println!("   パスワード: {}", password);
        }
        Ok(None) => {
            // ユーザを作成
            match User::create_with_password(&ctx.db, username, password).await {
                Ok(user) => {
                    println!("✅ テストユーザを作成しました:");
                    println!("   ユーザ名: {}", user.username);
                    println!("   パスワード: {}", password);
                    println!("   ID: {}", user.id);
                }
                Err(e) => {
                    eprintln!("❌ ユーザの作成に失敗しました: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ ユーザの検索に失敗しました: {}", e);
        }
    }
    
    Ok(())
}
```

### コードの解説

**`cli::boot::<App, Migrator>()`**
- アプリケーションを初期化
- データベース接続を確立
- マイグレーションを実行

**重複チェック**
```rust
match User::find_by_username(&ctx.db, username).await {
    Ok(Some(user)) => { /* 既に存在 */ }
    Ok(None) => { /* 作成 */ }
    Err(e) => { /* エラー */ }
}
```

- `Ok(Some(user))`：ユーザーが見つかった
- `Ok(None)`：ユーザーが見つからない
- `Err(e)`：検索エラー

### 実行方法

```bash
cargo run --bin seed_user
```

出力例：
```
✅ テストユーザを作成しました:
   ユーザ名: admin
   パスワード: admin123
   ID: 1
```

---

## 10.5 統合テストの実装

### テストファイルの作成

`tests/integration_test.rs`を作成します：

```rust
use loco_rs::testing;
use kiro_test::app::App;
use serial_test::serial;
use axum::http::StatusCode;

#[tokio::test]
#[serial]
async fn test_public_config_access() {
    testing::request::<App, _, _>(|request, _ctx| async move {
        // 公開ページにアクセス
        let response = request.get("/conf").await;
        assert_eq!(response.status_code(), StatusCode::OK);
    })
    .await;
}
```

### コードの解説

**`#[tokio::test]`**
- 非同期テスト用のマクロ
- `async fn`をテストとして実行できる

**`#[serial]`**
- テストを順次実行
- データベースの状態を分離
- `serial_test`クレートが提供

**`testing::request`**
- Locoのテストヘルパー
- テスト用のHTTPリクエストを送信

**`|request, _ctx|`**
- `request`：HTTPリクエストを送信するオブジェクト
- `_ctx`：アプリケーションコンテキスト（DB接続など）


### 認証テスト

#### ログイン成功テスト

```rust
#[tokio::test]
#[serial]
async fn test_auth_login_success() {
    testing::request::<App, _, _>(|request, ctx| async move {
        // テストユーザを作成
        use kiro_test::models::users::Model as User;
        let _ = User::create_with_password(&ctx.db, "admin", "admin123").await;
        
        // ログインページにアクセス
        let response = request.get("/auth/login").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        // 正しい認証情報でログイン
        let response = request
            .post("/auth/login")
            .form(&[("username", "admin"), ("password", "admin123")])
            .await;
        
        // ログイン成功後は管理画面にリダイレクト
        assert!(
            response.status_code() == StatusCode::FOUND 
            || response.status_code() == StatusCode::SEE_OTHER,
            "Expected redirect, got: {:?}", response.status_code()
        );
    })
    .await;
}
```

### コードの詳細解説

**テストユーザーの作成**
```rust
let _ = User::create_with_password(&ctx.db, "admin", "admin123").await;
```
- テストごとに新しいユーザーを作成
- インメモリDBなので、テスト終了後に消える
- `let _ =`でエラーを無視（既に存在する場合）

**フォームデータの送信**
```rust
.form(&[("username", "admin"), ("password", "admin123")])
```
- `application/x-www-form-urlencoded`形式
- HTMLフォームと同じ形式
- `.json()`とは異なる

**リダイレクトの確認**
```rust
assert!(
    response.status_code() == StatusCode::FOUND 
    || response.status_code() == StatusCode::SEE_OTHER
);
```
- `302 Found`または`303 See Other`を期待
- Axumは通常`303`を返す
- どちらもリダイレクトを意味する

#### ログイン失敗テスト

```rust
#[tokio::test]
#[serial]
async fn test_auth_login_failure() {
    testing::request::<App, _, _>(|request, ctx| async move {
        use kiro_test::models::users::Model as User;
        let _ = User::create_with_password(&ctx.db, "admin", "admin123").await;
        
        // 誤った認証情報でログイン
        let response = request
            .post("/auth/login")
            .form(&[("username", "admin"), ("password", "wrongpassword")])
            .await;
        
        // ログイン失敗（200でエラーメッセージ表示）
        assert_eq!(response.status_code(), StatusCode::OK);
        
        // レスポンスにエラーメッセージが含まれていることを確認
        let body = response.text();
        assert!(body.contains("ユーザ名またはパスワードが正しくありません"));
    })
    .await;
}
```

**設計の特徴：**
- ログイン失敗時は`401 Unauthorized`ではなく`200 OK`を返す
- エラーメッセージをHTMLに埋め込んで再表示
- セキュリティ：ユーザー名とパスワードのどちらが間違っているか特定させない

#### 未認証アクセステスト

```rust
#[tokio::test]
#[serial]
async fn test_unauthorized_access_to_admin() {
    testing::request::<App, _, _>(|request, _ctx| async move {
        // 未認証で管理画面にアクセス
        let response = request.get("/admin").await;
        
        // 未認証の場合はログインページにリダイレクト
        assert!(
            response.status_code() == StatusCode::FOUND
            || response.status_code() == StatusCode::SEE_OTHER,
            "Expected redirect, got: {:?}", response.status_code()
        );
    })
    .await;
}
```

**ミドルウェアの動作確認：**
- `require_auth`ミドルウェアが正しく動作
- 未認証ユーザーを自動的にログインページへリダイレクト

---

## 10.6 設定更新フローのテスト

### URL追加テスト

```rust
#[tokio::test]
#[serial]
async fn test_config_update_add_url() {
    testing::request::<App, _, _>(|request, ctx| async move {
        use kiro_test::models::users::Model as User;
        let _ = User::create_with_password(&ctx.db, "admin", "admin123").await;
        
        // ログイン
        let response = request
            .post("/auth/login")
            .form(&[("username", "admin"), ("password", "admin123")])
            .await;
        assert!(response.status_code() == StatusCode::FOUND 
            || response.status_code() == StatusCode::SEE_OTHER);
        
        // URL追加
        let response = request
            .post("/admin/edit")
            .form(&[
                ("new_urls", "https://newsite.com"),
            ])
            .await;
        
        assert!(
            response.status_code() == StatusCode::FOUND 
            || response.status_code() == StatusCode::SEE_OTHER
            || response.status_code() == StatusCode::OK,
            "Expected redirect or OK, got: {:?}", response.status_code()
        );
    })
    .await;
}
```

**テストの流れ：**
1. テストユーザーを作成
2. ログインしてセッションを確立
3. URL追加リクエストを送信
4. 成功レスポンスを確認

### 一括URL追加テスト

```rust
#[tokio::test]
#[serial]
async fn test_config_update_bulk_add() {
    testing::request::<App, _, _>(|request, ctx| async move {
        use kiro_test::models::users::Model as User;
        let _ = User::create_with_password(&ctx.db, "admin", "admin123").await;
        
        // ログイン
        let response = request
            .post("/auth/login")
            .form(&[("username", "admin"), ("password", "admin123")])
            .await;
        
        // 複数URL一括追加
        let response = request
            .post("/admin/edit")
            .form(&[
                ("new_urls", "https://site1.com\nhttps://site2.com\nhttps://site3.com"),
            ])
            .await;
        
        assert!(
            response.status_code() == StatusCode::FOUND 
            || response.status_code() == StatusCode::SEE_OTHER
            || response.status_code() == StatusCode::OK
        );
    })
    .await;
}
```

**改行区切りのURL：**
- フォームデータで改行文字（`\n`）を使用
- サーバー側で`lines()`メソッドで分割処理

---

## 10.7 テストの実行

### テスト実行コマンド

```bash
# テスト環境で実行
LOCO_ENV=test cargo test

# 詳細な出力
LOCO_ENV=test cargo test -- --nocapture

# 特定のテストのみ実行
LOCO_ENV=test cargo test test_auth_login_success
```

### 実行結果の例

```
running 9 tests
test test_public_config_access ... ok
test test_config_file_persistence ... ok
test test_unauthorized_access_to_admin ... ok
test test_unauthorized_access_to_admin_edit ... ok
test test_auth_login_success ... ok
test test_config_update_add_url ... ok
test test_config_update_delete_url ... ok
test test_config_update_bulk_add ... ok
test test_auth_login_failure ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured
```

**テスト結果の見方：**
- `ok`：テスト成功
- `FAILED`：テスト失敗
- `ignored`：スキップされたテスト
- `measured`：ベンチマークテスト

---

## 10.8 Clippyによるコード品質チェック

### Clippyとは

**Clippy**は、Rustの公式リンター（静的解析ツール）です。

**Clippyができること：**
- コードの問題点を指摘
- Rustの慣用的なパターンを提案
- パフォーマンスの改善提案
- 潜在的なバグの検出

### Clippyの実行

```bash
# 警告を表示
cargo clippy

# 警告をエラーとして扱う
cargo clippy -- -D warnings
```

### 修正例1：冗長なパターンマッチング

**修正前：**
```rust
if let Err(_) = session.insert(SESSION_USER_ID_KEY, user.id).await {
    return login_form_with_error(Some("セッションの保存に失敗しました".to_string())).into_response();
}
```

**Clippyの警告：**
```
warning: redundant pattern matching, consider using `is_err()`
```

**修正後：**
```rust
if session.insert(SESSION_USER_ID_KEY, user.id).await.is_err() {
    return login_form_with_error(Some("セッションの保存に失敗しました".to_string())).into_response();
}
```

**理由：**
- `is_err()`メソッドの方が簡潔で読みやすい
- パターンマッチングのオーバーヘッドを削減
- Rustの慣用的な書き方


### 修正例2：不要な条件分岐

**修正前：**
```rust
let content = if url_list_html.is_empty() {
    r#"<div class="empty-state">URLはありません</div>"#
} else {
    url_list_html
};
```

**問題点：**
- `url_list_html`が空の場合と空でない場合で異なる処理
- しかし、空の場合の処理が適切でない可能性

**修正後：**
```rust
let content = if url_list_html.is_empty() {
    r#"<div class="empty-state">現在、監視中のURLはありません。</div>"#.to_string()
} else {
    url_list_html
};
```

### 修正例3：不要なクローン

**修正前：**
```rust
let urls = urls.clone();
for url in urls.iter() {
    // 処理
}
```

**Clippyの警告：**
```
warning: unnecessary clone
```

**修正後：**
```rust
for url in urls.iter() {
    // 処理
}
```

**理由：**
- イテレータを使う場合、クローンは不要
- メモリとパフォーマンスの無駄

### Clippyの設定

プロジェクトルートに`.clippy.toml`を作成：

```toml
# 許可する警告
allow = [
    # 例：長い関数名を許可
    "too_many_arguments",
]

# 拒否する警告（エラーとして扱う）
deny = [
    "redundant_pattern_matching",
    "unnecessary_clone",
]
```

---

## 10.9 リリースビルド

### リリースビルドとは

**リリースビルド**は、本番環境用の最適化されたビルドです。

**デバッグビルドとの違い：**

| 項目 | デバッグビルド | リリースビルド |
|-----|-------------|-------------|
| コマンド | `cargo build` | `cargo build --release` |
| 最適化 | なし | 最大 |
| デバッグ情報 | あり | なし |
| ビルド時間 | 速い | 遅い |
| 実行速度 | 遅い | 速い |
| バイナリサイズ | 大きい | 小さい |

### リリースビルドの実行

```bash
cargo build --release
```

出力：
```
   Compiling kiro_test v0.1.0
    Finished `release` profile [optimized] target(s) in 1m 28s
```

**生成されるバイナリ：**
```
target/release/kiro_test
```

### リリースビルドの設定

`Cargo.toml`でリリースビルドをカスタマイズできます：

```toml
[profile.release]
opt-level = 3        # 最適化レベル（0-3）
lto = true           # Link Time Optimization
codegen-units = 1    # コード生成ユニット数（少ないほど最適化）
strip = true         # デバッグシンボルを削除
```

**最適化レベル：**
- `0`：最適化なし
- `1`：基本的な最適化
- `2`：標準的な最適化
- `3`：最大の最適化

---

## 10.10 トラブルシューティング

### 問題1：SessionManagerLayerが有効になっていない

**エラーメッセージ：**
```
Can't extract session. Is `SessionManagerLayer` enabled?
```

**原因：**
- セッションミドルウェアが設定されていない

**解決方法：**
```rust
// src/app.rs
async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
    use tower_sessions::{MemoryStore, SessionManagerLayer};
    use time::Duration;

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::hours(1)));

    Ok(router.layer(session_layer))
}
```

### 問題2：YAMLパースエラー

**エラーメッセージ：**
```
mapping values are not allowed in this context
```

**原因：**
- YAML内の`sqlite::memory:`がコロンを含むため、パーサーがキーと値の区切りと誤認識

**解決方法：**
```yaml
# 正しい
database:
  uri: "sqlite::memory:"

# 間違い
database:
  uri: sqlite::memory:
```

### 問題3：型の不一致（Duration）

**エラーメッセージ：**
```
expected `time::Duration`, found `std::time::Duration`
```

**原因：**
- `tower-sessions`は`time::Duration`を期待
- `std::time::Duration`とは異なる型

**解決方法：**
```rust
use time::Duration;  // ✅ 正しい

// ❌ 間違い
use std::time::Duration;
```

### 問題4：テストが失敗する

**症状：**
```
test test_auth_login_success ... FAILED
```

**デバッグ方法：**

1. **詳細な出力を表示**
```bash
LOCO_ENV=test cargo test -- --nocapture
```

2. **特定のテストのみ実行**
```bash
LOCO_ENV=test cargo test test_auth_login_success
```

3. **レスポンスの内容を確認**
```rust
let response = request.get("/admin").await;
println!("Status: {:?}", response.status_code());
println!("Body: {}", response.text());
```

---

## 10.11 継続的インテグレーション（CI）

### GitHub Actionsの設定

`.github/workflows/ci.yml`を作成：

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: clippy
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Cache cargo index
      uses: actions/cache@v3
      with:
        path: ~/.cargo/git
        key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Cache target directory
      uses: actions/cache@v3
      with:
        path: target
        key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run tests
      run: LOCO_ENV=test cargo test
      
    - name: Run clippy
      run: cargo clippy -- -D warnings
      
    - name: Build release
      run: cargo build --release
```

**CIの利点：**
- プッシュごとに自動テスト
- コードレビュー前に品質チェック
- チーム全体で品質を保証

---

## 10.12 まとめ

この章では、以下を学びました：

✅ テスト環境のセットアップ
✅ セッション管理の実装
✅ 統合テストの書き方
✅ 認証フローのテスト
✅ 設定更新フローのテスト
✅ Clippyによるコード品質チェック
✅ リリースビルドの作成

**重要なポイント：**

1. **テストは品質保証の基本**
   - バグの早期発見
   - リファクタリングの安全性

2. **セッション管理は認証の要**
   - `tower-sessions`を使用
   - `time::Duration`に注意

3. **統合テストで実際の動作を確認**
   - `#[serial]`で順次実行
   - インメモリDBで高速化

4. **Clippyでコード品質向上**
   - 慣用的なパターンの学習
   - パフォーマンスの改善

**テスト結果：**
```
✅ 統合テスト: 9/9 成功
✅ Clippyチェック: 警告なし
✅ リリースビルド: 成功
```

---

## 次のステップ

おめでとうございます！これで、Telegraf設定管理システムの実装が完了しました。

**学んだ技術：**
- Rustの基本（第0章）
- Locoフレームワーク（第1章）
- サービス層の設計（第2章）
- バリデーション（第3章）
- 認証システム（第4章）
- ビュー実装（第5-8章）
- テンプレートエンジン（第9章）
- テストと品質保証（第10章）

**さらに学ぶには：**
- データベースの最適化
- キャッシュの実装
- API設計
- デプロイメント戦略
- 監視とロギング

**参考リソース：**
- [Loco公式ドキュメント](https://loco.rs/docs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum公式ドキュメント](https://docs.rs/axum/)
- [SeaORM公式ドキュメント](https://www.sea-ql.org/SeaORM/)

これで、あなたは本格的なRust Webアプリケーションを構築できるようになりました！
