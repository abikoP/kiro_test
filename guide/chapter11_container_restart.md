# 第11章: コンテナ再起動機能の実装

## この章で学ぶこと

- 外部コマンドの実行（`std::process::Command`）
- コマンド実行結果の処理
- エラーハンドリングとユーザーフィードバック
- JavaScriptによる確認ダイアログ
- システム管理機能の実装

---

## 11.1 なぜコンテナ再起動機能が必要か

### 背景

このシステムは、Telegrafの設定ファイル（`telegraf.conf`）を編集するためのものです。

**Telegrafの動作環境：**
- TelegrafはDockerコンテナとして稼働
- InfluxDBやGrafanaと一緒に`docker-compose`で管理
- 設定ファイルを変更しても、コンテナを再起動しないと反映されない

**従来の問題点：**
```bash
# 設定を変更した後、毎回SSHでサーバーにログインして...
ssh user@server
cd /path/to/project
docker-compose restart telegraf
```

これは面倒で、ミスも起こりやすいです。

**解決策：**
Web画面から直接コンテナを再起動できるボタンを追加！

---

## 11.2 外部コマンドの実行

### std::process::Command とは

Rustから外部のコマンド（シェルコマンド）を実行するための標準ライブラリです。

**基本的な使い方：**

```rust
use std::process::Command;

// コマンドを実行
let output = Command::new("ls")
    .arg("-la")
    .output();
```

これは、シェルで`ls -la`を実行するのと同じです。

### docker-compose restart の実行

Telegrafコンテナを再起動するコマンド：

```bash
docker-compose restart telegraf
```

これをRustで実行します：

```rust
let output = Command::new("docker-compose")
    .arg("restart")
    .arg("telegraf")
    .output();
```

**ポイント：**
- `Command::new("docker-compose")`：実行するコマンド
- `.arg("restart")`：第1引数
- `.arg("telegraf")`：第2引数
- `.output()`：コマンドを実行して結果を取得

---

## 11.3 コントローラーの実装

### ファイル作成

`src/controllers/telegraf.rs`を新規作成します：

```rust
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use loco_rs::prelude::*;
use std::process::Command;

/// Telegraf再起動処理
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

### コードの詳細解説

#### 1. コマンド実行

```rust
let output = Command::new("docker-compose")
    .arg("restart")
    .arg("telegraf")
    .output();
```

**`.output()`の戻り値：**
- `Result<Output, Error>`型
- 成功：`Ok(Output)`
- 失敗：`Err(Error)`

#### 2. 結果の判定

```rust
match output {
    Ok(result) => {
        if result.status.success() {
            // コマンド成功
        } else {
            // コマンド失敗（終了コードが0以外）
        }
    }
    Err(e) => {
        // コマンド実行自体が失敗
    }
}
```

**3つのケース：**

1. **`Ok(result)` かつ `result.status.success()`**
   - コマンドが正常に実行され、成功した
   - 例：Telegrafコンテナが正常に再起動

2. **`Ok(result)` かつ `!result.status.success()`**
   - コマンドは実行されたが、エラーが発生
   - 例：Telegrafコンテナが見つからない

3. **`Err(e)`**
   - コマンド実行自体が失敗
   - 例：`docker-compose`コマンドが見つからない

#### 3. エラーメッセージの取得

```rust
let error_message = String::from_utf8_lossy(&result.stderr);
```

**`result.stderr`とは：**
- コマンドの標準エラー出力
- バイト列（`Vec<u8>`）として格納されている

**`String::from_utf8_lossy()`とは：**
- バイト列を文字列に変換
- 不正なUTF-8文字は`�`に置換
- エラーを返さず、必ず文字列を返す

**例：**
```rust
// バイト列
let bytes = vec![72, 101, 108, 108, 111];  // "Hello"

// 文字列に変換
let text = String::from_utf8_lossy(&bytes);
println!("{}", text);  // "Hello"
```

---

## 11.4 レスポンスHTMLの生成

### generate_response_html 関数

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
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background: #f5f5f5;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
        }}
        .result-container {{
            background: white;
            padding: 3rem;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
            text-align: center;
            max-width: 500px;
        }}
        .status-icon {{
            font-size: 4rem;
            margin-bottom: 1rem;
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

### コードの解説

**1. 成功/失敗の判定**
```rust
let (status_class, status_icon) = if success {
    ("success", "✓")
} else {
    ("error", "✗")
};
```

- タプルで2つの値を同時に設定
- 成功：緑色のチェックマーク
- 失敗：赤色のバツマーク

**2. 条件付きHTML生成**
```rust
if !detail.is_empty() {
    format!(r#"<div class="detail">{}</div>"#, html_escape::encode_text(detail))
} else {
    String::new()
}
```

- エラー詳細がある場合のみ表示
- ない場合は空文字列

---

## 11.5 ルーティングの設定

### モジュール登録

`src/controllers/mod.rs`に追加：

```rust
pub mod auth;
pub mod admin;
pub mod admin_list;
pub mod admin_edit;
pub mod config;
pub mod telegraf;  // 追加
```

### ルート追加

`src/app.rs`を更新：

```rust
use crate::controllers::{admin, admin_edit, admin_list, auth, config, telegraf};

// ...

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

**ポイント：**
- `POST /admin/telegraf/restart`：POSTメソッドで実行
- `protected_routes`に追加：認証が必要
- ログインしていないユーザーはアクセス不可

---

## 11.6 UIの実装

### ナビゲーションバーにボタンを追加

全ての管理画面（`admin.rs`、`admin_list.rs`、`admin_edit.rs`）のHTMLを更新します。

**修正前：**
```html
<div class="nav">
    <a href="/admin">ダッシュボード</a>
    <a href="/admin/list">URL一覧</a>
    <a href="/admin/edit">URL編集</a>
    <form method="POST" action="/auth/logout" class="logout-form">
        <button type="submit" class="logout-btn">ログアウト</button>
    </form>
</div>
```

**修正後：**
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

### JavaScriptによる確認ダイアログ

```html
onsubmit="return confirm('Telegrafコンテナを再起動しますか？');"
```

**動作：**
1. ユーザーがボタンをクリック
2. 確認ダイアログが表示される
3. 「OK」をクリック → フォームが送信される
4. 「キャンセル」をクリック → 何もしない

**`confirm()`関数：**
- ブラウザ標準の確認ダイアログ
- 戻り値：`true`（OK）または`false`（キャンセル）
- `return confirm(...)`で、`false`の場合はフォーム送信をキャンセル

### CSSスタイルの追加

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

**色の選択理由：**
- オレンジ色（`#f39c12`）：注意を促す色
- ログアウトボタン（赤）：危険な操作
- 再起動ボタン（オレンジ）：重要だが危険ではない

---

## 11.7 動作確認

### 1. ビルドと起動

```bash
cargo build
cargo run --bin kiro_test -- start
```

### 2. ログイン

ブラウザで`http://localhost:3000/auth/login`にアクセスし、ログインします。

### 3. 再起動ボタンをクリック

1. 管理画面のヘッダーにある「Telegraf再起動」ボタンをクリック
2. 確認ダイアログが表示される：「Telegrafコンテナを再起動しますか？」
3. 「OK」をクリック
4. 再起動が実行される
5. 結果画面が表示される

### 4. 結果の確認

**成功時：**
- 緑色のチェックマーク（✓）
- 「Telegrafコンテナを再起動しました。」
- 「管理画面に戻る」ボタン

**失敗時：**
- 赤色のバツマーク（✗）
- 「Telegrafコンテナの再起動に失敗しました。」
- エラー詳細（標準エラー出力）
- 「管理画面に戻る」ボタン

---

## 11.8 トラブルシューティング

### 問題1：docker-composeコマンドが見つからない

**エラー：**
```
エラー: No such file or directory (os error 2)
```

**原因：**
- `docker-compose`コマンドがインストールされていない
- PATHが通っていない

**解決方法：**
```bash
# インストール確認
which docker-compose

# macOS (Homebrew)
brew install docker-compose

# Linux
sudo apt-get install docker-compose
```

### 問題2：権限エラー

**エラー：**
```
permission denied while trying to connect to the Docker daemon socket
```

**原因：**
- Dockerデーモンへのアクセス権限がない

**解決方法：**
```bash
# ユーザーをdockerグループに追加
sudo usermod -aG docker $USER

# ログアウト/ログインして反映
```

### 問題3：Telegrafコンテナが存在しない

**エラー：**
```
No such service: telegraf
```

**原因：**
- `docker-compose.yml`に`telegraf`サービスが定義されていない
- 別のディレクトリでコマンドを実行している

**解決方法：**
- `docker-compose.yml`の場所を確認
- 必要に応じて作業ディレクトリを変更

---

## 11.9 セキュリティ考慮事項

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

### 3. エラー情報の表示

```rust
let error_message = String::from_utf8_lossy(&result.stderr);
```

- エラー詳細をユーザーに表示
- デバッグに役立つ
- 本番環境では詳細を隠すことも検討

---

## まとめ

この章で学んだ主要な概念：

1. **外部コマンドの実行**：`std::process::Command`でシステムコマンドを実行
2. **コマンド結果の処理**：成功/失敗を適切に判定
3. **エラーハンドリング**：3つのケースを処理（成功、コマンドエラー、実行エラー）
4. **ユーザーフィードバック**：結果を分かりやすく表示
5. **セキュリティ**：認証と確認ダイアログで保護
6. **UI/UX**：誤操作を防ぐインターフェース設計

これらの技術を組み合わせることで、安全で使いやすいシステム管理機能を実装できました。

---

## 次のステップ

おめでとうございます！これで、Telegraf設定管理システムの主要な機能がすべて実装されました。

**実装した機能：**
- 設定ファイルの読み書き
- URL検証
- 認証システム
- 管理画面（ダッシュボード、一覧、編集）
- テンプレートエンジン
- テスト
- **コンテナ再起動** ← New!

**さらに学ぶには：**
- [索引・用語集](./glossary.md)で重要な用語を復習
- [終わりに](./conclusion.md)で次のステップを確認

Happy Coding with Rust! 🦀
