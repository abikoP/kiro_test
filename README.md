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

- Rust 1.85.1以上（Edition 2024対応）
- Cargo
- SQLite開発ライブラリ（`sqlite-devel`）
- Docker & docker-compose（Telegrafコンテナ用）

### ローカル開発環境

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

4. 初期ユーザーを作成
```bash
cargo run --bin seed_user
```

デフォルトログイン情報：
- ユーザー名: `admin@example.com`
- パスワード: `admin`

### EC2デプロイメント（Amazon Linux 2023）

#### 1. 前提条件の確認

```bash
# Rustのバージョン確認（1.85以上が必要）
rustc --version

# rustupがない場合はインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
```

#### 2. 必要なパッケージのインストール

```bash
# SQLite開発ライブラリをインストール
sudo yum install sqlite-devel -y

# Gitがない場合
sudo yum install git -y
```

#### 3. スワップ領域の作成（メモリ不足対策）

```bash
# 4GBのスワップファイルを作成
sudo dd if=/dev/zero of=/swapfile bs=1M count=4096 status=progress
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# 永続化
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab

# 確認
free -h
```

#### 4. プロジェクトのビルド

```bash
cd /home/ec2-user/kiro_test

# システムSQLiteを使用してビルド
export LIBSQLITE3_SYS_USE_PKG_CONFIG=1
cargo clean
cargo build --release -j 1
```

#### 5. 設定ファイルの調整

```bash
# ポート番号を設定（デフォルトは3000）
nano config/production.yaml
```

`server`セクションを編集：
```yaml
server:
  port: 8081
  host: 127.0.0.1  # Nginxリバースプロキシ経由の場合
```

#### 6. 初期ユーザーの作成

```bash
cargo run --release --bin seed_user
```

#### 7. アプリケーションの起動

```bash
# 環境変数を設定
export LOCO_ENV=production
export JWT_SECRET="your-strong-secret-$(openssl rand -hex 16)"

# フォアグラウンドで起動（テスト用）
cargo run --release --bin kiro_test -- start --port 8081

# バックグラウンドで起動（推奨）
cargo run --release --bin kiro_test -- start --port 8081 > app.log 2>&1 &

# プロセスIDを確認
echo $!

# ログを確認
tail -f app.log
```

**注意**: `--port`オプションでポート番号を指定できます。設定ファイル（`config/production.yaml`）のポート設定より優先されます。

#### 8. systemdサービス化（推奨）

```bash
# サービスファイルを作成
sudo nano /etc/systemd/system/telegraf-manager.service
```

以下の内容を貼り付け：

```ini
[Unit]
Description=Telegraf Config Manager
After=network.target docker.service
Requires=docker.service

[Service]
Type=simple
User=ec2-user
WorkingDirectory=/home/ec2-user/kiro_test
Environment=LOCO_ENV=production
Environment=JWT_SECRET=your-strong-secret-key-here
Environment=LIBSQLITE3_SYS_USE_PKG_CONFIG=1
ExecStart=/home/ec2-user/kiro_test/target/release/kiro_test start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

サービスを有効化：

```bash
sudo systemctl daemon-reload
sudo systemctl enable telegraf-manager
sudo systemctl start telegraf-manager
sudo systemctl status telegraf-manager
```

#### 9. Nginxリバースプロキシの設定

```bash
# Nginxをインストール
sudo yum install nginx -y

# 設定ファイルを作成
sudo nano /etc/nginx/conf.d/telegraf-manager.conf
```

以下の内容を貼り付け：

```nginx
server {
    listen 8081;
    server_name _;

    location / {
        proxy_pass http://127.0.0.1:8081;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Nginxを起動：

```bash
sudo systemctl enable nginx
sudo systemctl start nginx
sudo systemctl status nginx
```

#### 10. ファイアウォール設定

```bash
# firewalldが有効な場合
sudo firewall-cmd --permanent --add-port=8081/tcp
sudo firewall-cmd --reload

# iptablesの場合
sudo iptables -I INPUT -p tcp --dport 8081 -j ACCEPT
```

#### 11. AWSセキュリティグループの設定

AWSコンソールで、EC2インスタンスのセキュリティグループに以下を追加：

- **タイプ**: カスタムTCP
- **ポート範囲**: 8081
- **ソース**: 0.0.0.0/0（または特定のIPアドレス）

#### 12. アクセス確認

```bash
# ローカルから確認
curl http://localhost:8081/

# 外部から確認
# ブラウザで http://<EC2のパブリックIP>:8081/ にアクセス
```

### Telegraf設定ファイルの連携

```bash
# Telegrafコンテナの設定ファイル場所を確認
docker inspect telegraf | grep -A 10 "Mounts"

# 設定ファイルをコピー
docker cp telegraf:/etc/telegraf/telegraf.conf ./conf/telegraf.conf

# または、環境変数で指定
export TELEGRAF_CONFIG_PATH="/path/to/telegraf.conf"
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

## 機能

### 完了
- [x] Locoプロジェクトのセットアップ
- [x] データベースマイグレーション（Usersテーブル）
- [x] プロジェクト構造の整理
- [x] 設定ファイル操作機能（ConfigService）
- [x] URL検証機能（UrlValidationService）
- [x] 認証機能（ログイン・ログアウト）
- [x] 公開設定閲覧機能（/conf）
- [x] 管理画面トップページ（/admin/）
- [x] URL一覧表示機能（/admin/list/）
- [x] URL編集機能（/admin/edit/）
- [x] Telegrafコンテナ再起動機能（/admin/restart）
- [x] HTMLテンプレート（Tera）
- [x] EC2デプロイメント対応

## トラブルシューティング

### ビルドエラー: Edition 2024

```bash
# Rustを最新版にアップデート
rustup update stable
rustup default stable
```

### ビルドエラー: メモリ不足

```bash
# スワップ領域を追加
sudo dd if=/dev/zero of=/swapfile bs=1M count=4096
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# 並列ビルドジョブ数を制限
cargo build --release -j 1
```

### ビルドエラー: libsqlite3-sys

```bash
# SQLite開発ライブラリをインストール
sudo yum install sqlite-devel -y  # Amazon Linux
# または
sudo apt-get install libsqlite3-dev -y  # Ubuntu/Debian

# システムSQLiteを使用
export LIBSQLITE3_SYS_USE_PKG_CONFIG=1
cargo build --release
```

### 外部からアクセスできない

1. **リッスン状態を確認**
```bash
sudo netstat -tlnp | grep 8081
```

2. **Nginxの状態を確認**
```bash
sudo systemctl status nginx
sudo nginx -t
```

3. **ファイアウォールを確認**
```bash
sudo firewall-cmd --list-ports
sudo iptables -L -n -v | grep 8081
```

4. **AWSセキュリティグループを確認**
- EC2コンソールでポート8081が開放されているか確認

### Dockerグループの権限エラー

```bash
# ec2-userをdockerグループに追加
sudo usermod -aG docker ec2-user

# ログアウト/ログインが必要
exit
# 再度SSH接続
```

## アーキテクチャ

### ネットワーク構成（EC2デプロイメント）

```
外部ブラウザ
    ↓
EC2セキュリティグループ (ポート8081許可)
    ↓
Nginx (0.0.0.0:8081)
    ↓ リバースプロキシ
Telegraf Manager (127.0.0.1:8081)
    ↓ docker-compose restart
Telegraf Container (Docker)
```

### セキュリティ

- アプリケーションは`127.0.0.1`でリッスン（外部に直接公開されない）
- Nginxがリバースプロキシとして機能
- JWT認証によるセッション管理
- パスワードはbcryptでハッシュ化

## ライセンス

MIT
