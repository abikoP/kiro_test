# Task 11: Telegraf再起動機能の実装

## 概要

このタスクでは、管理画面からTelegrafコンテナを再起動できる機能を実装しました。telegraf.confファイルを編集した後、設定を反映させるためにコンテナの再起動が必要になるため、この機能を追加しました。

## 背景

Telegrafは、InfluxDBとGrafanaとともにDockerコンテナとして稼働しています。telegraf.confの設定（URL一覧など）を変更した場合、変更を反映させるにはTelegrafコンテナを再起動する必要があります。

この機能により、管理者はWebインターフェースから直接コンテナを再起動でき、サーバーにSSH接続してコマンドを実行する手間が省けます。

## 実装内容

### 11.1 TelegrafControllerの実装

`src/controllers/telegraf.rs`を新規作成し、Telegraf再起動機能のコントローラーを実装しました。

#### restart()メソッド - Telegraf再起動処理

```rust
use std::process::Command;

pub async fn restart(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // docker-compose restart telegraf を実行
    let output = Command::new("docker-compose")
        .arg("restart")
        .arg("telegraf")
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                // 成功時のレスポンス
                let html = generate_response_html(
                    true,
                    "Telegrafコンテナを再起動しました。",
                    "",
                );
                Ok(Html(html))
            } else {
                // コマンドは実行されたがエラーが発生
                let error_message = String::from_utf8_lossy(&result.stderr);
                let html = generate_response_html(
                    false,
                    "Telegrafコンテナの再起動に失敗しました。",
                    &error_message,
                );
                Ok(Html(html))
            }
        }
        Err(e) => {
            // コマンド実行自体が失敗
            let html = generate_response_html(
                false,
                "Telegrafコンテナの再起動に失敗しました。",
                &format!("エラー: {}", e),
            );
            Ok(Html(html))
        }
    }
}
```

**学習ポイント:**

1. **std::process::Command**
   ```rust
   let output = Command::new("docker-compose")
       .arg("restart")
       .arg("telegraf")
       .output();
   ```
   - `Command::new()`: 実行するコマンドを指定
   - `.arg()`: コマンドライン引数を追加
   - `.output()`: コマンドを実行し、出力を取得

2. **コマンド実行結果の処理**
   ```rust
   match output {
       Ok(result) => {
           if result.status.success() {
               // 成功処理
           } else {
               // コマンドは実行されたがエラー
           }
       }
       Err(e) => {
           // コマンド実行自体が失敗
       }
   }
   ```
   - `Ok(result)`: コマンドが実行された（成功/失敗は`result.status`で判定）
   - `Err(e)`: コマンド実行自体が失敗（コマンドが見つからないなど）

3. **標準エラー出力の取得**
   ```rust
   let error_message = String::from_utf8_lossy(&result.stderr);
   ```
   - `result.stderr`: コマンドの標準エラー出力（バイト列）
   - `String::from_utf8_lossy()`: バイト列を文字列に変換（不正なUTF-8は置換）

#### generate_response_html() - レスポンスHTML生成

```rust
fn generate_response_html(success: bool, message: &str, detail: &str) -> String {
    let (status_class, status_icon) = if success {
        ("success", "✓")
    } else {
        ("error", "✗")
    };

    format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <title>Telegraf再起動 - Telegraf設定管理</title>
    <style>
        .result-container {{
            background: white;
            padding: 3rem;
            border-radius: 8px;
            text-align: center;
        }}
        .status-icon.success {{
            color: #27ae60;
        }}
        .status-icon.error {{
            color: #e74c3c;
        }}
    </style>
</head>
<body>
    <div class="result-container">
        <div class="status-icon {}">{}</div>
        <div class="message">{}</div>
        {}
        <a href="/admin" class="btn">管理画面に戻る</a>
    </div>
</body>
</html>
        "#,
        status_class,
        status_icon,
        html_escape::encode_text(message),
        if !detail.is_empty() {
            format!(r#"<div class="detail">{}</div>"#, html_escape::encode_text(detail))
        } else {
            String::new()
        }
    )
}
```

**学習ポイント:**
- 成功/失敗に応じて異なるアイコンと色を表示
- エラー詳細がある場合のみ表示
- HTMLエスケープでXSS対策

### 11.2 ルーティングの設定

`src/app.rs`にルートを追加しました。

```rust
use crate::controllers::{admin, admin_edit, admin_list, auth, config, telegraf};

fn routes(ctx: &AppContext) -> AppRoutes {
    // ...
    
    let protected_routes = Routes::new()
        .add("/admin", get(admin::index))
        .add("/admin/list", get(admin_list::index))
        .add("/admin/edit", get(admin_edit::edit))
        .add("/admin/edit", post(admin_edit::update))
        .add("/admin/telegraf/restart", post(telegraf::restart))  // 追加
        .add("/auth/logout", post(auth::logout))
        .layer(axum::middleware::from_fn_with_state(
            ctx.clone(),
            require_auth,
        ));
    
    // ...
}
```

**ポイント:**
- `POST /admin/telegraf/restart`: POSTメソッドで再起動を実行
- 認証ミドルウェア（`require_auth`）で保護
- 認証済みユーザーのみアクセス可能

### 11.3 UIの実装

全ての管理画面（ダッシュボード、URL一覧、URL編集）のヘッダーに「Telegraf再起動」ボタンを追加しました。

#### ナビゲーションバーの更新

```html
<div class="nav">
    <a href="/admin">ダッシュボード</a>
    <a href="/admin/list">URL一覧</a>
    <a href="/admin/edit">URL編集</a>
    <form method="POST" action="/admin/telegraf/restart" class="restart-form" 
          onsubmit="return confirm('Telegrafコンテナを再起動しますか？');">
        <button type="submit" class="restart-btn">Telegraf再起動</button>
    </form>
    <form method="POST" action="/auth/logout" class="logout-form">
        <button type="submit" class="logout-btn">ログアウト</button>
    </form>
</div>
```

**学習ポイント:**

1. **JavaScriptによる確認ダイアログ**
   ```html
   onsubmit="return confirm('Telegrafコンテナを再起動しますか？');"
   ```
   - `confirm()`: ブラウザの確認ダイアログを表示
   - ユーザーが「OK」をクリックした場合のみフォームを送信
   - 「キャンセル」をクリックした場合は何もしない

2. **フォームの配置**
   - URL編集とログアウトボタンの間に配置
   - `display: inline`で横並びに表示

#### CSSスタイルの追加

```css
.restart-form {
    display: inline;
    margin-right: 0.5rem;
}
.restart-btn {
    background: #f39c12;  /* オレンジ色 */
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 5px;
    cursor: pointer;
    font-size: 0.9rem;
}
.restart-btn:hover {
    background: #e67e22;  /* ホバー時は濃いオレンジ */
}
```

**デザインの意図:**
- オレンジ色（`#f39c12`）: 注意を促す色（警告ではないが重要な操作）
- ログアウトボタン（赤）とは異なる色で区別
- ダッシュボードリンク（青）とも異なる色

### 11.4 モジュールの登録

`src/controllers/mod.rs`に新しいコントローラーを登録しました。

```rust
pub mod auth;
pub mod admin;
pub mod admin_list;
pub mod admin_edit;
pub mod config;
pub mod telegraf;  // 追加
```

## セキュリティ考慮事項

### 1. 認証の必須化

```rust
let protected_routes = Routes::new()
    // ...
    .layer(axum::middleware::from_fn_with_state(
        ctx.clone(),
        require_auth,
    ));
```

- 認証ミドルウェアで保護
- ログインしていないユーザーはアクセス不可

### 2. 確認ダイアログ

```javascript
onsubmit="return confirm('Telegrafコンテナを再起動しますか？');"
```

- 誤操作を防ぐ
- ユーザーに再確認を促す

### 3. エラーハンドリング

```rust
match output {
    Ok(result) => {
        if result.status.success() {
            // 成功処理
        } else {
            // エラー処理
        }
    }
    Err(e) => {
        // コマンド実行失敗の処理
    }
}
```

- すべてのエラーケースを処理
- エラー詳細をユーザーに表示

## 動作確認

### 1. サーバーの起動

```bash
cargo run -- start
```

### 2. ログイン

ブラウザで`http://localhost:3000/auth/login`にアクセスし、ログインします。

### 3. Telegraf再起動ボタンのクリック

1. 管理画面のヘッダーにある「Telegraf再起動」ボタンをクリック
2. 確認ダイアログが表示される
3. 「OK」をクリック
4. 再起動が実行される
5. 結果画面が表示される

### 4. 結果の確認

**成功時:**
- 緑色のチェックマーク（✓）
- 「Telegrafコンテナを再起動しました。」というメッセージ
- 「管理画面に戻る」ボタン

**失敗時:**
- 赤色のバツマーク（✗）
- 「Telegrafコンテナの再起動に失敗しました。」というメッセージ
- エラー詳細（標準エラー出力）
- 「管理画面に戻る」ボタン

## トラブルシューティング

### docker-composeコマンドが見つからない

**エラー:**
```
エラー: No such file or directory (os error 2)
```

**原因:**
- `docker-compose`コマンドがインストールされていない
- PATHが通っていない

**解決方法:**
```bash
# docker-composeのインストール確認
which docker-compose

# インストールされていない場合
# macOS (Homebrew)
brew install docker-compose

# Linux
sudo apt-get install docker-compose
```

### 権限エラー

**エラー:**
```
permission denied while trying to connect to the Docker daemon socket
```

**原因:**
- Dockerデーモンへのアクセス権限がない

**解決方法:**
```bash
# ユーザーをdockerグループに追加
sudo usermod -aG docker $USER

# ログアウト/ログインして反映
```

### Telegrafコンテナが存在しない

**エラー:**
```
No such service: telegraf
```

**原因:**
- docker-compose.ymlにtelegrafサービスが定義されていない
- 別のディレクトリでコマンドを実行している

**解決方法:**
- docker-compose.ymlの場所を確認
- 必要に応じてコマンドに`-f`オプションでファイルパスを指定

## まとめ

このタスクで学んだ主要な概念：

1. **外部コマンドの実行**: `std::process::Command`でシステムコマンドを実行
2. **エラーハンドリング**: コマンド実行の成功/失敗を適切に処理
3. **ユーザーフィードバック**: 実行結果を分かりやすく表示
4. **セキュリティ**: 認証と確認ダイアログで保護
5. **UI/UX**: 誤操作を防ぐインターフェース設計

これらの技術を組み合わせることで、安全で使いやすいコンテナ管理機能を実装できました。

## 参考リンク

- [std::process::Command - Rust Documentation](https://doc.rust-lang.org/std/process/struct.Command.html)
- [Docker Compose CLI Reference](https://docs.docker.com/compose/reference/)
- [HTML confirm() Method - MDN](https://developer.mozilla.org/en-US/docs/Web/API/Window/confirm)
