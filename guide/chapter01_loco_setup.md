# 第1章: Locoプロジェクトのセットアップ

## この章で学ぶこと

- Locoフレームワークとは何か
- プロジェクトの作成と構造
- 依存関係の設定
- データベースマイグレーション
- 基本的なLocoコマンド

---

## 1.1 Locoフレームワークとは

### Locoの特徴

**Loco**は、Rustで書かれたWebアプリケーションフレームワークです。Ruby on Railsに影響を受けており、「Convention over Configuration（設定より規約）」の哲学を持っています。

**主な特徴:**
- 🚀 **高速**: Rustの性能を活かした高速な実行
- 🛠️ **フルスタック**: データベース、認証、ルーティングなど全て含む
- 📦 **バッテリー同梱**: すぐに使える機能が豊富
- 🔒 **型安全**: Rustの型システムによる安全性
- 🎯 **開発者体験**: Railsライクな使いやすさ

### Locoが提供するもの

- **ルーティング**: URLとコントローラーの紐付け
- **ORM**: データベース操作（SeaORM）
- **マイグレーション**: データベーススキーマ管理
- **認証**: ユーザ認証とセッション管理
- **CLI**: 便利なコマンドラインツール
- **テスト**: 統合テストのサポート

---

## 1.2 プロジェクトの作成

### 新規プロジェクトの作成

Locoには専用のCLIツールがありますが、このテキストでは既存のプロジェクト構造を使います。

```bash
# プロジェクトディレクトリに移動
cd kiro_test

# プロジェクト構造を確認
ls -la
```

### プロジェクト構造の理解

```
kiro_test/
├── src/                    # ソースコード
│   ├── main.rs            # エントリポイント
│   ├── app.rs             # アプリケーション設定
│   ├── lib.rs             # ライブラリルート
│   ├── controllers/       # コントローラー（後で作成）
│   ├── models/            # データモデル
│   └── services/          # ビジネスロジック（後で作成）
├── migration/             # データベースマイグレーション
├── config/                # 設定ファイル
├── assets/                # 静的ファイル・テンプレート
├── tests/                 # テスト
├── Cargo.toml            # プロジェクト設定
└── .env                  # 環境変数
```

**各ディレクトリの役割:**

- **src/**: Rustのソースコードを配置
- **migration/**: データベーススキーマの変更履歴
- **config/**: 環境別の設定ファイル（development, production）
- **assets/**: HTML、CSS、画像などの静的ファイル
- **tests/**: 統合テスト

---

## 1.3 依存関係の設定

### Cargo.tomlの理解

`Cargo.toml`は、Rustプロジェクトの設定ファイルです。依存関係（使用するライブラリ）を定義します。

```toml
[package]
name = "kiro_test"
version = "0.1.0"
edition = "2021"
description = "Telegraf設定ファイル管理システム"

[dependencies]
# Locoフレームワーク
loco-rs = { version = "0.13", features = ["with-db", "auth_jwt", "cli"] }

# データベースORM
sea-orm = { version = "1", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }

# マイグレーション
migration = { path = "./migration" }

# シリアライゼーション
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 非同期ランタイム
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# 設定ファイル操作用
toml = "0.8"
url = "2.5"
thiserror = "1.0"

# 認証用
bcrypt = "0.15"

# HTMLエスケープ用
html-escape = "0.2"

# 時間処理用
time = "0.3"
```

### 主要な依存関係の説明

#### loco-rs
Locoフレームワーク本体です。

```toml
loco-rs = { version = "0.13", features = ["with-db", "auth_jwt", "cli"] }
```

**features（機能フラグ）:**
- `with-db`: データベース機能を有効化
- `auth_jwt`: JWT認証機能を有効化
- `cli`: CLIコマンド機能を有効化

#### sea-orm
データベース操作のためのORM（Object-Relational Mapping）です。

```toml
sea-orm = { version = "1", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

**features:**
- `sqlx-sqlite`: SQLiteデータベースを使用
- `runtime-tokio-rustls`: 非同期ランタイムとTLS
- `macros`: エンティティマクロを有効化

#### serde
データのシリアライゼーション（変換）ライブラリです。

```toml
serde = { version = "1", features = ["derive"] }
```

**用途:**
- JSON ↔ Rust構造体の変換
- TOML ↔ Rust構造体の変換

#### tokio
非同期プログラミングのためのランタイムです。

```toml
tokio = { version = "1", features = ["full"] }
```

Locoは非同期処理を多用するため、tokioが必要です。

### 依存関係のインストール

依存関係は、初回ビルド時に自動的にダウンロードされます。

```bash
cargo build
```

**初回は時間がかかります（5〜10分程度）。**

---

## 1.4 エントリポイントの作成

### main.rsの実装

`src/main.rs`は、アプリケーションのエントリポイント（起動地点）です。

```rust
use kiro_test::app::App;
use loco_rs::cli;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::main::<App, Migrator>().await
}
```

**コードの説明:**

#### use文
```rust
use kiro_test::app::App;
use loco_rs::cli;
use migration::Migrator;
```

- `kiro_test::app::App`: 後で作成するアプリケーション設定
- `loco_rs::cli`: LocoのCLI機能
- `migration::Migrator`: データベースマイグレーション管理

#### #[tokio::main]
```rust
#[tokio::main]
async fn main() -> loco_rs::Result<()> {
```

- `#[tokio::main]`: 非同期ランタイムを初期化するマクロ
- `async fn`: 非同期関数の定義
- `loco_rs::Result<()>`: Locoのエラー型を返す

#### cli::main
```rust
cli::main::<App, Migrator>().await
```

- `cli::main`: LocoのCLIメイン関数
- `<App, Migrator>`: ジェネリクス型パラメータ
  - `App`: アプリケーション設定
  - `Migrator`: マイグレーション管理
- `.await`: 非同期処理の完了を待つ

**この1行で、以下のコマンドが使えるようになります:**
- `cargo run -- start`: サーバー起動
- `cargo run -- db migrate`: マイグレーション実行
- `cargo run -- routes`: ルート一覧表示

---

## 1.5 アプリケーション設定

### app.rsの実装

`src/app.rs`は、アプリケーションの設定を定義します。

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

### コードの詳細解説

#### Appの定義
```rust
pub struct App;
```

空の構造体ですが、`Hooks`トレイトを実装することで、アプリケーションの動作をカスタマイズします。

#### Hooksトレイト
```rust
#[async_trait]
impl Hooks for App {
```

`Hooks`トレイトは、Locoアプリケーションのライフサイクルをカスタマイズするためのものです。

#### app_name()
```rust
fn app_name() -> &'static str {
    env!("CARGO_CRATE_NAME")
}
```

- `env!`: コンパイル時にCargo.tomlから値を取得するマクロ
- `CARGO_CRATE_NAME`: プロジェクト名（"kiro_test"）
- `&'static str`: 静的な文字列スライス（プログラム全体で有効）

#### app_version()
```rust
fn app_version() -> String {
    format!(
        "{} ({})",
        env!("CARGO_PKG_VERSION"),
        option_env!("BUILD_SHA")
            .or(option_env!("GITHUB_SHA"))
            .unwrap_or("dev")
    )
}
```

- `CARGO_PKG_VERSION`: Cargo.tomlのバージョン
- `BUILD_SHA`: ビルド時のGitコミットハッシュ（CI/CD環境）
- `unwrap_or("dev")`: 環境変数がない場合は"dev"

#### boot()
```rust
async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
    create_app::<Self, Migrator>(mode, environment).await
}
```

アプリケーション起動時の初期化処理です。

- `StartMode`: サーバーモード、ワーカーモードなど
- `Environment`: 環境設定（development, production）
- `create_app`: Locoの標準的な起動処理

#### routes()
```rust
fn routes(_ctx: &AppContext) -> AppRoutes {
    AppRoutes::with_default_routes()
        // ここにルートを追加していきます
}
```

ルーティング設定です。後の章で、ここにカスタムルートを追加します。

- `AppRoutes::with_default_routes()`: Locoのデフォルトルート
  - `/_ping`: ヘルスチェック
  - `/_health`: 詳細なヘルスチェック

#### その他のメソッド

```rust
async fn connect_workers(...) -> Result<()> { Ok(()) }
fn register_tasks(...) {}
async fn truncate(...) -> Result<()> { Ok(()) }
async fn seed(...) -> Result<()> { Ok(()) }
```

今は空実装ですが、後で必要に応じて実装します。

- `connect_workers`: バックグラウンドジョブ
- `register_tasks`: カスタムタスク
- `truncate`: テスト用データクリア
- `seed`: 初期データ投入

---

## 1.6 データベースマイグレーション

### マイグレーションとは

**マイグレーション**は、データベーススキーマ（テーブル構造）の変更履歴を管理する仕組みです。

**メリット:**
- 変更履歴が残る
- チーム開発で同じスキーマを共有できる
- ロールバック（元に戻す）が可能

### Usersテーブルのマイグレーション

`migration/src/m20240101_000001_create_users_table.rs`を作成します。

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

### コードの詳細解説

#### MigrationTrait
```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
```

マイグレーションの実装トレイトです。

#### up()メソッド
```rust
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
```

マイグレーション適用時の処理（テーブル作成）です。

**テーブル作成:**
```rust
manager
    .create_table(
        Table::create()
            .table(Users::Table)
            .if_not_exists()
```

- `create_table`: テーブル作成
- `table(Users::Table)`: テーブル名
- `if_not_exists()`: 既に存在する場合はスキップ

**カラム定義:**
```rust
.col(
    ColumnDef::new(Users::Id)
        .integer()
        .not_null()
        .auto_increment()
        .primary_key(),
)
```

- `ColumnDef::new(Users::Id)`: カラム名
- `.integer()`: 整数型
- `.not_null()`: NULL不可
- `.auto_increment()`: 自動採番
- `.primary_key()`: 主キー

```rust
.col(ColumnDef::new(Users::Username).string().not_null().unique_key())
```

- `.string()`: 文字列型
- `.unique_key()`: ユニーク制約（重複不可）

```rust
.col(
    ColumnDef::new(Users::CreatedAt)
        .timestamp_with_time_zone()
        .not_null()
        .default(Expr::current_timestamp()),
)
```

- `.timestamp_with_time_zone()`: タイムゾーン付きタイムスタンプ
- `.default(Expr::current_timestamp())`: デフォルト値（現在時刻）

#### down()メソッド
```rust
async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .drop_table(Table::drop().table(Users::Table).to_owned())
        .await
}
```

ロールバック時の処理（テーブル削除）です。

#### DeriveIden
```rust
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

識別子の型安全な定義です。

- `Table`: テーブル名
- その他: カラム名

**メリット:**
- タイポ（入力ミス）を防ぐ
- コンパイル時にチェック
- SQLインジェクション対策

### マイグレーションの登録

`migration/src/lib.rs`にマイグレーションを登録します。

```rust
pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users_table::Migration),
        ]
    }
}
```

---

## 1.7 設定ファイル

### development.yamlの作成

`config/development.yaml`を作成します。

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

### 設定項目の説明

#### server
```yaml
server:
  port: 3000
  host: 127.0.0.1
```

- `port`: HTTPサーバーのポート番号
- `host`: バインドするIPアドレス（127.0.0.1はローカルホストのみ）

#### database
```yaml
database:
  uri: sqlite://db.sqlite?mode=rwc
  enable_logging: true
  auto_migrate: true
```

- `uri`: データベース接続文字列
  - `sqlite://db.sqlite`: SQLiteファイルのパス
  - `?mode=rwc`: 読み書き可能、存在しない場合は作成
- `enable_logging`: SQLログを出力
- `auto_migrate`: 起動時に自動マイグレーション

#### auth
```yaml
auth:
  jwt:
    secret: development-secret-key-change-in-production
    expiration: 86400
```

- `secret`: JWT署名用の秘密鍵（**本番環境では必ず変更**）
- `expiration`: トークン有効期限（秒）86400秒 = 24時間

#### logger
```yaml
logger:
  enable: true
  level: debug
  format: compact
```

- `enable`: ロギング有効化
- `level`: ログレベル（trace, debug, info, warn, error）
- `format`: 出力形式（compact, json）

---

## 1.8 プロジェクトのビルドと実行

### ビルド

```bash
cargo build
```

**初回は時間がかかります。**依存関係のダウンロードとコンパイルが行われます。

### マイグレーション実行

```bash
cargo run -- db migrate
```

`db.sqlite`ファイルが作成され、`users`テーブルが作成されます。

### サーバー起動

```bash
cargo run -- start
```

以下のような出力が表示されます：

```
                      ▄     ▀                     
                                 ▀  ▄             
                  ▄       ▀     ▄  ▄ ▄▀           
                                    ▄ ▀▄▄         
                        ▄     ▀    ▀  ▀▄▀█▄       
                                          ▀█▄     
▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄▄▄ ▀▀█    
 ██████  █████   ███ █████   ███ █████   ███ ▀█   
 ██████  █████   ███ █████   ▀▀▀ █████   ███ ▄█▄  
 ██████  █████   ███ █████       █████   ███ ████▄
 ██████  █████   ███ █████   ▄▄▄ █████   ███ █████
 ██████  █████   ███  ████   ███ █████   ███ ████▀
   ▀▀▀██▄ ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀ ██▀  
       ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀    
                https://loco.rs
environment: development
   database: logging, automigrate
     logger: debug
compilation: debug
      modes: server
listening on http://localhost:3000
```

### 動作確認

ブラウザまたはcurlで確認します。

```bash
curl http://localhost:3000/_ping
```

レスポンス：
```json
{"ok":true}
```

---

## 1.9 便利なLocoコマンド

### ルート一覧表示

```bash
cargo run -- routes
```

現在登録されているルートが表示されます。

```
[GET] /_ping
[GET] /_health
```

### 設定診断

```bash
cargo run -- doctor
```

プロジェクトの設定状態を診断します。

### データベース操作

```bash
# マイグレーション実行
cargo run -- db migrate

# マイグレーション状態確認
cargo run -- db status

# データベースリセット
cargo run -- db reset
```

---

## 1.10 トラブルシューティング

### エラー: rustc バージョン互換性

```
error: rustc 1.85.1 is not supported by the following package:
  home@0.5.12 requires rustc 1.88
```

**解決方法:**
```bash
cargo update home@0.5.12 --precise 0.5.9
```

### エラー: 設定ファイルのフィールド不足

```
Error: YAMLFile(Error("database: missing field `min_connections`", ...))
```

**解決方法:**
`config/development.yaml`に不足しているフィールドを追加します。

### エラー: ポートが既に使用中

```
Error: Address already in use (os error 48)
```

**解決方法:**
1. 他のプロセスを停止する
2. または、`config/development.yaml`でポート番号を変更する

```yaml
server:
  port: 3001  # 別のポートに変更
```

---

## 1.11 まとめ

この章では、以下を学びました：

- ✅ Locoフレームワークの概要
- ✅ プロジェクト構造の理解
- ✅ 依存関係の設定（Cargo.toml）
- ✅ エントリポイントの作成（main.rs）
- ✅ アプリケーション設定（app.rs）
- ✅ データベースマイグレーション
- ✅ 設定ファイル（development.yaml）
- ✅ 基本的なLocoコマンド

**重要なポイント:**
- Locoは「Convention over Configuration」の哲学
- マイグレーションでデータベーススキーマを管理
- 設定ファイルで環境別の設定を管理
- CLIコマンドで開発を効率化

---

## 次のステップ

次の章では、**設定ファイル操作機能**を実装します。TOMLファイルの読み書きと、サービス層の設計を学びます。

[第2章: 設定ファイル操作機能の実装](./chapter02_config_service.md)に進みましょう！
