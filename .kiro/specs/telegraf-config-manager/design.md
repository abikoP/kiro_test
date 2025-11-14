# 設計ドキュメント

## 概要

Telegraf設定ファイル管理システムは、Rust + Locoフレームワークを使用したWEBアプリケーションです。ブラウザ上でTelegrafの`[[inputs.http_response]]`セクションのURL一覧を閲覧・編集できる機能を提供します。認証機能により、権限のあるユーザのみが設定を変更できます。

## アーキテクチャ

### 技術スタック

- **言語**: Rust (Edition 2024)
- **フレームワーク**: Loco (Railsライクなフルスタックフレームワーク)
- **認証**: Locoの組み込み認証機能
- **テンプレートエンジン**: Tera (Locoデフォルト)
- **設定ファイル解析**: toml crate
- **URL検証**: url crate

### アーキテクチャパターン

MVC (Model-View-Controller) パターンを採用します。Locoフレームワークの標準構造に従います。

```
┌─────────────┐
│   Browser   │
└──────┬──────┘
       │ HTTP Request
       ▼
┌─────────────────────────────────┐
│        Router (Loco)            │
│  - /conf                        │
│  - /admin/                      │
│  - /admin/list/                 │
│  - /admin/edit/                 │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│       Controllers               │
│  - ConfigController             │
│  - AdminController              │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│         Services                │
│  - ConfigService                │
│  - UrlValidationService         │
└──────┬──────────────────────────┘
       │
       ▼
┌─────────────────────────────────┐
│      File System                │
│  ./conf/telegraf.conf           │
└─────────────────────────────────┘
```

## コンポーネントと インターフェース

### 1. Models

#### User Model
```rust
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Locoの認証機能を使用してユーザ管理を実装します。

### 2. Controllers

#### ConfigController
公開用の設定閲覧コントローラー

**責務:**
- `/conf`エンドポイントの処理
- 設定ファイルの読み込みと表示

**主要メソッド:**
- `show()`: 現在の設定を表示

#### AdminController
管理画面用コントローラー

**責務:**
- `/admin/`エンドポイントの処理（ダッシュボード）
- 認証チェック

**主要メソッド:**
- `index()`: 管理画面トップページ表示

#### AdminListController
URL一覧表示コントローラー

**責務:**
- `/admin/list/`エンドポイントの処理
- URL一覧の取得と表示

**主要メソッド:**
- `index()`: URL一覧を表示

#### AdminEditController
URL編集コントローラー

**責務:**
- `/admin/edit/`エンドポイントの処理
- URL追加・削除処理
- 設定ファイルへの保存

**主要メソッド:**
- `edit()`: 編集フォーム表示
- `update()`: 変更の保存処理

### 3. Services

#### ConfigService
設定ファイル操作サービス

**責務:**
- telegraf.confファイルの読み書き
- TOML形式のパース
- `[[inputs.http_response]]`セクションの抽出と更新

**主要メソッド:**
```rust
pub struct ConfigService;

impl ConfigService {
    // 設定ファイルを読み込む
    pub fn read_config() -> Result<TelegrafConfig, ConfigError>;
    
    // URL一覧を取得
    pub fn get_urls() -> Result<Vec<String>, ConfigError>;
    
    // URL一覧を更新
    pub fn update_urls(urls: Vec<String>) -> Result<(), ConfigError>;
    
    // 設定ファイル全体を取得（表示用）
    pub fn get_raw_config() -> Result<String, ConfigError>;
}
```

#### UrlValidationService
URL検証サービス

**責務:**
- URLの形式検証
- HTTP/HTTPSスキームチェック

**主要メソッド:**
```rust
pub struct UrlValidationService;

impl UrlValidationService {
    // 単一URLの検証
    pub fn validate_url(url: &str) -> Result<(), ValidationError>;
    
    // 複数URLの検証
    pub fn validate_urls(urls: &[String]) -> Result<(), ValidationError>;
}
```

### 4. Middleware

#### AuthMiddleware
Locoの組み込み認証ミドルウェアを使用

**責務:**
- セッション管理
- 認証状態の確認
- 未認証ユーザのリダイレクト

## データモデル

### TelegrafConfig構造体
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegrafConfig {
    pub global_tags: Option<toml::Value>,
    pub agent: Option<AgentConfig>,
    pub outputs: Option<toml::Value>,
    pub inputs: Option<InputsConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub interval: Option<String>,
    pub round_interval: Option<bool>,
    // ... その他のフィールド
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputsConfig {
    pub http_response: Option<Vec<HttpResponseInput>>,
    // ... その他のinputタイプ
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponseInput {
    pub urls: Vec<String>,
    pub response_timeout: Option<String>,
    pub method: Option<String>,
    pub follow_redirects: Option<bool>,
}
```

### フォームデータ

#### UrlUpdateForm
```rust
#[derive(Debug, Deserialize)]
pub struct UrlUpdateForm {
    pub urls: Vec<String>,  // 更新後のURL一覧
}
```

## エラーハンドリング

### エラータイプ

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("設定ファイルが見つかりません: {0}")]
    FileNotFound(String),
    
    #[error("設定ファイルの読み込みに失敗しました: {0}")]
    ReadError(String),
    
    #[error("設定ファイルの書き込みに失敗しました: {0}")]
    WriteError(String),
    
    #[error("TOMLのパースに失敗しました: {0}")]
    ParseError(String),
    
    #[error("http_responseセクションが見つかりません")]
    HttpResponseSectionNotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("無効なURL形式です: {0}")]
    InvalidUrlFormat(String),
    
    #[error("HTTP/HTTPSスキームが必要です: {0}")]
    InvalidScheme(String),
    
    #[error("URLが空です")]
    EmptyUrl,
}
```

### エラーレスポンス

- 設定ファイル読み込みエラー: 500 Internal Server Error + エラーメッセージ
- URL検証エラー: 400 Bad Request + 検証エラー詳細
- 認証エラー: 401 Unauthorized + ログインページへリダイレクト
- 権限エラー: 403 Forbidden

## ルーティング

```rust
// routes.rs
pub fn routes() -> Routes {
    Routes::new()
        // 公開エンドポイント
        .add("/conf", get(config_controller::show))
        
        // 認証エンドポイント（Locoデフォルト）
        .add("/auth/login", get(auth::login_form))
        .add("/auth/login", post(auth::login))
        .add("/auth/logout", post(auth::logout))
        
        // 管理画面（認証必須）
        .add("/admin", get(admin_controller::index))
        .add("/admin/list", get(admin_list_controller::index))
        .add("/admin/edit", get(admin_edit_controller::edit))
        .add("/admin/edit", post(admin_edit_controller::update))
}
```

## ビュー構成

### テンプレート一覧

1. **layout/base.html**: 共通レイアウト
2. **config/show.html**: 設定表示ページ (`/conf`)
3. **auth/login.html**: ログインフォーム
4. **admin/index.html**: 管理画面トップ (`/admin/`)
5. **admin/list.html**: URL一覧ページ (`/admin/list/`)
6. **admin/edit.html**: URL編集ページ (`/admin/edit/`)

### UI設計

#### /conf ページ
- ヘッダー: "Telegraf設定情報"
- 現在監視中のURL一覧を表形式で表示
- フッター: 管理画面へのリンク

#### /admin/ ページ
- ヘッダー: ナビゲーションバー（一覧、編集、ログアウト）
- ダッシュボード: 現在のURL数、最終更新日時
- クイックアクションボタン

#### /admin/list/ ページ
- URL一覧テーブル
  - 列: No, URL
- 編集ページへのボタン

#### /admin/edit/ ページ
- 現在のURL一覧（削除チェックボックス付き）
- 新規URL追加フォーム（複数行テキストエリア）
- 保存ボタン、キャンセルボタン
- バリデーションエラー表示エリア

## テスト戦略

### 単体テスト

1. **ConfigService**
   - 設定ファイル読み込みテスト
   - URL抽出テスト
   - URL更新テスト
   - エラーハンドリングテスト

2. **UrlValidationService**
   - 有効なURL検証テスト
   - 無効なURL検証テスト
   - スキーム検証テスト

### 統合テスト

1. **認証フロー**
   - ログイン成功/失敗
   - セッション管理
   - 未認証アクセス制御

2. **設定更新フロー**
   - URL追加
   - URL削除
   - 複数URL一括操作
   - ファイル書き込み確認

### E2Eテスト（手動）

1. ブラウザで全ページにアクセス
2. ログイン→編集→保存の一連の流れ
3. 実際のtelegraf.confファイルの変更確認

## セキュリティ考慮事項

1. **認証・認可**
   - Locoのセッションベース認証を使用
   - パスワードはbcryptでハッシュ化
   - CSRF保護（Locoデフォルト）

2. **ファイルアクセス**
   - 設定ファイルパスは固定（./conf/telegraf.conf）
   - パストラバーサル攻撃対策

3. **入力検証**
   - URL形式の厳密な検証
   - XSS対策（Teraテンプレートの自動エスケープ）

4. **エラー情報**
   - 本番環境では詳細なエラー情報を隠蔽
   - ログには詳細を記録

## デプロイメント

### 環境変数

```env
# データベース（Locoデフォルト）
DATABASE_URL=sqlite://db.sqlite

# アプリケーション設定
RUST_LOG=info
LOCO_ENV=production

# 設定ファイルパス（オプション）
TELEGRAF_CONFIG_PATH=./conf/telegraf.conf
```

### ディレクトリ構造

```
kiro_test/
├── src/
│   ├── app.rs              # アプリケーションエントリポイント
│   ├── controllers/
│   │   ├── config.rs
│   │   ├── admin.rs
│   │   ├── admin_list.rs
│   │   └── admin_edit.rs
│   ├── models/
│   │   └── user.rs
│   ├── services/
│   │   ├── config_service.rs
│   │   └── url_validation_service.rs
│   ├── views/
│   └── lib.rs
├── assets/
│   └── views/
│       ├── layout/
│       ├── config/
│       ├── auth/
│       └── admin/
├── conf/
│   └── telegraf.conf       # 編集対象ファイル
├── migrations/             # データベースマイグレーション
├── tests/
└── Cargo.toml
```

## パフォーマンス考慮事項

1. **ファイルI/O**
   - 設定ファイルは小さいため、都度読み込みで問題なし
   - 将来的にキャッシュ機構を検討

2. **同時編集**
   - 現バージョンでは同時編集制御なし
   - 将来的にファイルロック機構を検討

## 拡張性

将来の機能追加に備えた設計：

1. **複数設定ファイル対応**
   - ConfigServiceを抽象化
   - ファイルパスを動的に指定可能に

2. **他のinputプラグイン対応**
   - InputsConfigを拡張
   - プラグイン別の編集画面追加

3. **変更履歴管理**
   - 設定変更のログ記録
   - ロールバック機能

4. **API提供**
   - REST APIエンドポイント追加
   - 外部システムとの連携
