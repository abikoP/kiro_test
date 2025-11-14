# タスク1: Locoプロジェクトのセットアップ

## 概要

このタスクでは、Axumベースの実装からLocoフレームワークベースの実装に移行し、Telegraf設定ファイル管理システムの基盤を構築しました。

## 実装内容

### 1. 依存関係の設定（Cargo.toml）

```toml
[dependencies]
loco-rs = { version = "0.13", features = ["with-db", "auth_jwt", "cli"] }
sea-orm = { version = "1", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
migration = { path = "./migration" }

serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# 設定ファイル操作用
toml = "0.8"
url = "2.5"
thiserror = "1.0"
```

**ポイント:**
- `loco-rs`: Railsライクなフルスタックフレームワーク
  - `with-db`: データベース機能を有効化
  - `auth_jwt`: JWT認証機能を有効化
  - `cli`: CLIコマンド機能を有効化
- `sea-orm`: Locoが使用するORM
  - `sqlx-sqlite`: SQLiteバックエンド
  - `runtime-tokio-rustls`: 非同期ランタイムとTLS
  - `macros`: エンティティマクロ

### 2. アプリケーションエントリポイント（src/main.rs）

```rust
use kiro_test::app::App;
use loco_rs::cli;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App, Migrator>().await
}
```

**解説:**
- `cli::main`: Loco CLIのメイン関数
- ジェネリクス型パラメータ:
  - `App`: アプリケーション設定を定義する構造体
  - `Migrator`: データベースマイグレーションを管理する構造体
- Locoが提供するCLIコマンド（start, db, routes等）が自動的に利用可能になる

### 3. アプリケーション設定（src/app.rs）

```rust
use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use sea_orm::DatabaseConnection;

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            // ここにルートを追加していきます
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &loco_rs::bgworker::Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(_db: &DatabaseConnection) -> Result<()> {
        Ok(())
    }

    async fn seed(_db: &DatabaseConnection, _base: &std::path::Path) -> Result<()> {
        Ok(())
    }
}
```

**解説:**

#### Hooksトレイト
Locoアプリケーションのライフサイクルをカスタマイズするためのトレイト。

- **`app_name()`**: アプリケーション名を返す
  - `env!("CARGO_CRATE_NAME")`: コンパイル時にCargo.tomlから取得

- **`app_version()`**: バージョン情報を返す
  - CI/CD環境でのビルドSHAを含める

- **`boot()`**: アプリケーション起動時の初期化処理
  - `StartMode`: サーバーモード、ワーカーモード等
  - `create_app`: Locoの標準的な起動処理を実行

- **`routes()`**: ルーティング設定
  - `AppRoutes::with_default_routes()`: Locoのデフォルトルート（ヘルスチェック等）
  - 今後、カスタムルートをここに追加

- **`connect_workers()`**: バックグラウンドワーカーの設定
  - 非同期ジョブ処理が必要な場合に使用

- **`register_tasks()`**: カスタムタスクの登録
  - `cargo run -- task <name>`で実行可能なタスク

- **`truncate()`**: テスト用のデータベースクリーンアップ
  - テスト実行時にデータをクリアする処理

- **`seed()`**: 初期データの投入
  - 開発環境やテスト環境用のサンプルデータ

### 4. データベースマイグレーション

#### migration/Cargo.toml

```toml
[dependencies]
sea-orm-migration = { version = "1", features = ["runtime-tokio-rustls", "sqlx-sqlite"] }
async-trait = "0.1"
```

#### migration/src/m20240101_000001_create_users_table.rs

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
    CreatedAt,
    UpdatedAt,
}
```

**解説:**

- **`MigrationTrait`**: マイグレーションの実装トレイト
  - `up()`: マイグレーション適用時の処理（テーブル作成）
  - `down()`: ロールバック時の処理（テーブル削除）

- **`DeriveIden`**: 識別子の型安全な定義
  - SQLインジェクション対策
  - コンパイル時の型チェック

- **`timestamp_with_time_zone()`**: タイムゾーン付きタイムスタンプ
  - SQLiteでは`timestamp_with_timezone_text`型として保存される

### 5. エンティティモデル（src/models/_entities/users.rs）

```rust
use loco_rs::prelude::*;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

**解説:**

- **`DeriveEntityModel`**: SeaORMのエンティティマクロ
  - 自動的に以下を生成:
    - `Entity`: エンティティ構造体
    - `ActiveModel`: 変更可能なモデル
    - `Column`: カラム列挙型

- **`Model`**: データベースレコードを表す構造体
  - `#[sea_orm(primary_key)]`: 主キーの指定
  - `DateTimeWithTimeZone`: Locoが提供する型エイリアス

- **`Relation`**: テーブル間のリレーション定義
  - 現時点では関連なし（空の列挙型）

- **`ActiveModelBehavior`**: モデルのライフサイクルフック
  - `before_save`, `after_save`等のフックを実装可能

### 6. 設定ファイル（config/development.yaml）

```yaml
server:
  port: 3000
  host: 127.0.0.1

database:
  uri: sqlite://db.sqlite?mode=rwc
  enable_logging: true
  auto_migrate: true
  min_connections: 1
  max_connections: 10
  connect_timeout: 500
  idle_timeout: 500

auth:
  jwt:
    secret: development-secret-key-change-in-production
    expiration: 86400

logger:
  enable: true
  level: debug
  format: compact
  override_filter: null
```

**解説:**

- **server**: HTTPサーバー設定
  - `port`: リッスンポート
  - `host`: バインドアドレス

- **database**: データベース接続設定
  - `uri`: SQLite接続文字列（`mode=rwc`は読み書き可能で存在しない場合作成）
  - `auto_migrate`: 起動時に自動マイグレーション実行
  - `min_connections/max_connections`: コネクションプール設定

- **auth**: 認証設定
  - `jwt.secret`: JWT署名用の秘密鍵（本番環境では環境変数から取得）
  - `jwt.expiration`: トークン有効期限（秒）

- **logger**: ロギング設定
  - `level`: ログレベル（trace, debug, info, warn, error）
  - `format`: 出力形式（compact, json）

## プロジェクト構造

```
kiro_test/
├── src/
│   ├── main.rs              # CLIエントリポイント
│   ├── app.rs               # アプリケーション設定
│   ├── lib.rs               # ライブラリルート
│   ├── controllers/         # HTTPコントローラー
│   │   └── mod.rs
│   ├── models/              # データモデル
│   │   ├── mod.rs
│   │   ├── _entities/       # SeaORM生成エンティティ
│   │   │   ├── mod.rs
│   │   │   └── users.rs
│   │   └── users.rs         # カスタムモデルロジック
│   └── services/            # ビジネスロジック
│       └── mod.rs
├── migration/               # データベースマイグレーション
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── m20240101_000001_create_users_table.rs
├── config/                  # 環境別設定
│   ├── development.yaml
│   └── production.yaml
├── assets/                  # 静的ファイル・テンプレート
│   └── views/
├── conf/                    # アプリケーション固有設定
│   └── telegraf.conf
├── tests/                   # 統合テスト
├── .env                     # 環境変数
└── Cargo.toml
```

## 実行コマンド

### 開発

```bash
# サーバー起動
cargo run -- start

# データベースマイグレーション
cargo run -- db migrate

# ルート一覧表示
cargo run -- routes

# 設定診断
cargo run -- doctor
```

### ビルド

```bash
# 開発ビルド
cargo build

# リリースビルド
cargo build --release
```

### テスト

```bash
# テスト実行
cargo test

# コード品質チェック
cargo clippy

# フォーマット
cargo fmt
```

## トラブルシューティング

### Rustバージョン互換性エラー

```bash
error: rustc 1.85.1 is not supported by the following package:
  home@0.5.12 requires rustc 1.88
```

**解決方法:**
```bash
cargo update home@0.5.12 --precise 0.5.9
```

### 設定ファイルのフィールド不足エラー

```
Error: YAMLFile(Error("database: missing field `min_connections`", ...))
```

**解決方法:**
設定ファイルに必要なフィールドを追加する。Locoのバージョンによって必要なフィールドが異なる場合があるため、エラーメッセージを確認して対応する。

## 次のステップ

タスク2以降で以下を実装していきます：

1. **ConfigService**: telegraf.confファイルの読み書き
2. **UrlValidationService**: URL検証ロジック
3. **認証機能**: ログイン・ログアウト
4. **コントローラー**: 各エンドポイントの実装
5. **ビュー**: Teraテンプレートの作成

## 参考リンク

- [Loco公式ドキュメント](https://loco.rs)
- [SeaORM公式ドキュメント](https://www.sea-ql.org/SeaORM/)
- [Tokio公式ドキュメント](https://tokio.rs)
