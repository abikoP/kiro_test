# Telegraf設定ファイル管理システム

ブラウザ上でTelegrafの設定ファイル（telegraf.conf）内のHTTP監視URL一覧を編集できるWEBアプリケーションです。

## 技術スタック

- **言語**: Rust (Edition 2021)
- **Webフレームワーク**: Loco (Railsライクなフルスタックフレームワーク)
- **ORM**: SeaORM
- **データベース**: SQLite
- **テンプレートエンジン**: Tera
- **設定ファイル解析**: toml
- **URL検証**: url

## プロジェクト構造

```
kiro_test/
├── src/
│   ├── main.rs              # アプリケーションエントリポイント
│   ├── lib.rs               # ライブラリルート
│   ├── controllers/         # コントローラー
│   │   ├── mod.rs
│   │   ├── config.rs        # 公開設定閲覧
│   │   ├── admin.rs         # 管理画面トップ
│   │   ├── admin_list.rs    # URL一覧表示
│   │   └── admin_edit.rs    # URL編集
│   ├── models/              # データモデル
│   │   └── mod.rs
│   └── services/            # ビジネスロジック
│       ├── mod.rs
│       ├── config_service.rs           # 設定ファイル操作
│       └── url_validation_service.rs   # URL検証
├── conf/
│   └── telegraf.conf        # 編集対象の設定ファイル
├── config/                  # アプリケーション設定
│   ├── development.yaml
│   └── production.yaml
├── assets/
│   └── views/               # HTMLテンプレート
│       ├── layout/
│       ├── config/
│       ├── auth/
│       └── admin/
├── tests/                   # テスト
└── Cargo.toml
```

## セットアップ

### 前提条件

- Rust 1.85.1以上
- Loco CLI（オプション）

### インストール

1. リポジトリをクローン
```bash
git clone <repository-url>
cd kiro_test
```

2. 依存関係をインストール
```bash
cargo build
```

3. 環境変数を設定
```bash
cp .env.example .env
# .envファイルを編集（必要に応じて）
```

4. データベースマイグレーションを実行
```bash
cargo run -- db migrate
```

## 開発

### ビルド

```bash
# 開発ビルド
cargo build

# リリースビルド
cargo build --release
```

### 実行

```bash
# 開発モードでサーバーを起動
cargo run -- start

# 本番モードで起動
cargo run --release -- start --environment production

# ポートを指定して起動
cargo run -- start --port 8080
```

サーバーは `http://localhost:3000` で起動します。

### その他のLocoコマンド

```bash
# ルート一覧を表示
cargo run -- routes

# データベース操作
cargo run -- db migrate    # マイグレーション実行
cargo run -- db reset      # データベースリセット
cargo run -- db status     # マイグレーション状態確認

# コード生成
cargo run -- generate scaffold <name>  # CRUD機能の生成
cargo run -- generate model <name>     # モデルの生成

# 設定診断
cargo run -- doctor
```

### テスト

```bash
# すべてのテストを実行
cargo test

# コード品質チェック
cargo clippy

# コードフォーマット
cargo fmt
```

## 実装状況

### 完了
- [x] Locoプロジェクトのセットアップ
- [x] データベースマイグレーション（Usersテーブル）
- [x] プロジェクト構造の整理
- [x] 基本的なLocoアプリケーションの構築

### TODO
- [ ] 設定ファイル操作機能（ConfigService）
- [ ] URL検証機能（UrlValidationService）
- [ ] 認証機能（ログイン・ログアウト）
- [ ] 公開設定閲覧機能（/conf）
- [ ] 管理画面トップページ（/admin/）
- [ ] URL一覧表示機能（/admin/list/）
- [ ] URL編集機能（/admin/edit/）
- [ ] HTMLテンプレート（Tera）
- [ ] 統合テスト

## ライセンス

MIT
