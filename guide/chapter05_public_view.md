# 第5章: 公開設定閲覧機能の実装

## この章で学ぶこと

- コントローラーの基本
- HTMLレスポンスの生成
- ルーティングの設定
- 公開ページと保護ページの分離
- サービスとコントローラーの連携

---

## 5.1 公開ページとは

### 認証不要のページ

**公開ページ**は、ログインせずに誰でもアクセスできるページです。

**このシステムでの公開ページ:**
- `/conf`: 現在監視中のURL一覧を表示

**なぜ公開にする？**
- 監視対象URLは機密情報ではない
- 外部システムから簡単にアクセスできる
- 透明性を高める

**注意:** 機密情報を含む場合は、認証を必須にすべきです。

---

## 5.2 ConfigControllerの実装

### ディレクトリ構造

```
src/
├── controllers/
│   ├── mod.rs
│   ├── auth.rs      # 認証コントローラー（第4章）
│   └── config.rs    # 設定表示コントローラー（今回）
```

### src/controllers/config.rsの作成

```rust
use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;

use crate::services::config_service::ConfigService;

/// 設定情報を表示（公開）
pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // 設定ファイルからURLを取得
    let urls = ConfigService::get_urls()?;

    let mut url_list = String::new();
    for (i, url) in urls.iter().enumerate() {
        url_list.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            i + 1,
            url
        ));
    }

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Telegraf設定情報</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background: #f5f5f5;
        }}
        .header {{
            background: #667eea;
            color: white;
            padding: 1rem 2rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .header h1 {{
            margin: 0;
        }}
        .container {{
            max-width: 1200px;
            margin: 2rem auto;
            padding: 0 2rem;
        }}
        .card {{
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }}
        th, td {{
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background: #f8f9fa;
            font-weight: 600;
            color: #333;
        }}
        .footer {{
            text-align: center;
            margin-top: 2rem;
            color: #666;
        }}
        .footer a {{
            color: #667eea;
            text-decoration: none;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Telegraf設定情報</h1>
    </div>
    <div class="container">
        <div class="card">
            <h2>監視中のURL一覧</h2>
            <table>
                <thead>
                    <tr>
                        <th>No.</th>
                        <th>URL</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>
        <div class="footer">
            <p><a href="/admin">管理画面へ</a></p>
        </div>
    </div>
</body>
</html>
        "#,
        url_list
    );

    Ok(Html(html))
}
```

### コードの詳細解説

#### use文
```rust
use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;
use crate::services::config_service::ConfigService;
```

**Axumの型:**
- `State`: アプリケーションコンテキストを取得
- `Html`: HTMLレスポンスを返す
- `IntoResponse`: レスポンスに変換できる型

#### 関数のシグネチャ
```rust
pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
```

**引数:**
- `State(_ctx)`: アプリケーションコンテキスト
  - `_ctx`: 今は使わないが、将来のために定義
  - アンダースコアで未使用警告を抑制

**戻り値:**
- `Result<impl IntoResponse>`: 成功時はレスポンス、失敗時はエラー
- `impl IntoResponse`: 具体的な型を隠蔽

#### URLの取得
```rust
let urls = ConfigService::get_urls()?;
```

- `ConfigService::get_urls()`: 第2章で実装したサービス
- `?`: エラー時は早期リターン

#### HTMLテーブルの生成
```rust
let mut url_list = String::new();
for (i, url) in urls.iter().enumerate() {
    url_list.push_str(&format!(
        "<tr><td>{}</td><td>{}</td></tr>",
        i + 1,
        url
    ));
}
```

**処理の流れ:**

##### 1. 空文字列の作成
```rust
let mut url_list = String::new();
```

- `String::new()`: 空の可変文字列
- `mut`: 変更可能

##### 2. イテレーション
```rust
for (i, url) in urls.iter().enumerate() {
```

- `.iter()`: イテレーターを取得
- `.enumerate()`: インデックス付きイテレーター
- `(i, url)`: インデックスと値のタプル

**例:**
```rust
let urls = vec!["https://example.com", "https://api.example.com"];
for (i, url) in urls.iter().enumerate() {
    println!("{}: {}", i, url);
}
// 出力:
// 0: https://example.com
// 1: https://api.example.com
```

##### 3. 文字列の追加
```rust
url_list.push_str(&format!(
    "<tr><td>{}</td><td>{}</td></tr>",
    i + 1,
    url
));
```

- `format!()`: 文字列をフォーマット
- `i + 1`: 1から始まる番号
- `push_str()`: 文字列を追加

**生成されるHTML:**
```html
<tr><td>1</td><td>https://example.com</td></tr>
<tr><td>2</td><td>https://api.example.com</td></tr>
```

#### HTMLの生成
```rust
let html = format!(
    r#"
<!DOCTYPE html>
<html lang="ja">
...
    "#,
    url_list
);
```

- `r#"..."#`: Raw文字列リテラル
- `{}`: `url_list`を埋め込む
- `{{}}`: CSSの`{}`を表現

#### レスポンスの返却
```rust
Ok(Html(html))
```

- `Html(html)`: HTMLレスポンスを作成
- `Ok()`: 成功を表す

**Htmlの役割:**
- Content-Typeヘッダーを`text/html`に設定
- ブラウザがHTMLとして解釈

---

## 5.3 モジュールの設定

### src/controllers/mod.rsの更新

```rust
pub mod auth;
pub mod config;
```

これで、他のモジュールから使えるようになります。

```rust
use crate::controllers::config;
```

---

## 5.4 ルーティングの設定

### src/app.rsのroutes()を更新

```rust
fn routes(_ctx: &AppContext) -> AppRoutes {
    use axum::routing::get;
    use crate::controllers::{auth, config};

    AppRoutes::with_default_routes()
        // 公開ルート（認証不要）
        .add("/conf", get(config::show))
        .add("/auth/login", get(auth::login_form))
        .add("/auth/login", post(auth::login))
}
```

### コードの詳細解説

#### ルートの追加
```rust
.add("/conf", get(config::show))
```

- `/conf`: URLパス
- `get()`: GETリクエスト
- `config::show`: ハンドラー関数

**HTTPメソッド:**
- `get()`: データの取得（読み取り専用）
- `post()`: データの作成・更新
- `put()`: データの更新
- `delete()`: データの削除

#### 公開ルートと保護ルート

```rust
// 公開ルート（認証不要）
.add("/conf", get(config::show))
.add("/auth/login", get(auth::login_form))

// 保護ルート（認証必須）
.add("/admin", get(admin::index))
.layer(axum::middleware::from_fn(require_auth))
```

**ポイント:**
- 公開ルートはミドルウェアを適用しない
- 保護ルートは`layer()`でミドルウェアを適用

---

## 5.5 動作確認

### サーバーの起動

```bash
cargo run -- start
```

### ブラウザでアクセス

```
http://localhost:3000/conf
```

**表示される内容:**
- ヘッダー: 「Telegraf設定情報」
- テーブル: 監視中のURL一覧
- フッター: 「管理画面へ」のリンク

### curlでの確認

```bash
curl http://localhost:3000/conf
```

HTMLが返ってくることを確認できます。

---

## 5.6 エラーハンドリング

### エラーの流れ

```rust
pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let urls = ConfigService::get_urls()?;  // ← エラーが発生する可能性
    // ...
    Ok(Html(html))
}
```

**エラーが発生した場合:**

1. `ConfigService::get_urls()`が`Err(ConfigError)`を返す
2. `?`演算子でエラーを伝播
3. `ConfigError`が`loco_rs::Error`に変換される
4. Locoが適切なHTTPエラーレスポンスを返す

**例: 設定ファイルが見つからない**
```
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "error": "設定ファイルが見つかりません: ./conf/telegraf.conf"
}
```

### エラーハンドリングの改善

より詳細なエラーハンドリングをする場合：

```rust
pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let urls = match ConfigService::get_urls() {
        Ok(urls) => urls,
        Err(e) => {
            // エラーページを表示
            let error_html = format!(
                r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <title>エラー</title>
</head>
<body>
    <h1>エラーが発生しました</h1>
    <p>{}</p>
</body>
</html>
                "#,
                e
            );
            return Ok(Html(error_html));
        }
    };
    
    // 通常の処理...
    Ok(Html(html))
}
```

---

## 5.7 セキュリティの考慮

### XSS対策

現在の実装では、URLを直接HTMLに埋め込んでいます。

```rust
url_list.push_str(&format!(
    "<tr><td>{}</td><td>{}</td></tr>",
    i + 1,
    url
));
```

**潜在的なリスク:**
- URLに`<script>`タグが含まれる場合、XSS攻撃が可能

**対策1: HTMLエスケープ**

```rust
use html_escape::encode_text;

url_list.push_str(&format!(
    "<tr><td>{}</td><td>{}</td></tr>",
    i + 1,
    encode_text(url)
));
```

**対策2: URL検証**
- 第3章で実装した`UrlValidationService`を使う
- 無効なURLは保存されない

**今回の実装:**
- ConfigServiceが検証済みのURLのみを返す
- 追加のエスケープは不要

### 認証の設計判断

このエンドポイントは意図的に認証不要にしています。

**理由:**
- 監視対象URLは機密情報ではない
- 外部システムから簡単にアクセスできる

**もし機密性が必要な場合:**

```rust
// 保護ルートに移動
.add("/conf", get(config::show))
.layer(axum::middleware::from_fn(require_auth))
```

---

## 5.8 CSSスタイリング

### デザインのポ��ント

実装したHTMLには、モダンなUIデザインを適用しています。

```css
body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: #f5f5f5;
}

.header {
    background: #667eea;  /* 紫系のアクセントカラー */
    color: white;
    padding: 1rem 2rem;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.card {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}
```

**デザインの原則:**
- **カードレイアウト**: コンテンツを白い背景で囲む
- **シャドウ**: 軽いbox-shadowで奥行きを表現
- **カラースキーム**: 紫系のアクセントカラー
- **タイポグラフィ**: システムフォントで読みやすさを確保

---

## 5.9 まとめ

この章では、以下を学びました：

- ✅ コントローラーの基本的な実装
- ✅ HTMLレスポンスの生成
- ✅ ルーティングの設定
- ✅ 公開ページと保護ページの分離
- ✅ サービスとコントローラーの連携
- ✅ エラーハンドリング
- ✅ セキュリティの考慮

**重要なポイント:**
- コントローラーはリクエストを受け取り、レスポンスを返す
- サービス層でビジネスロジックを処理
- ルーティングで公開/保護を制御
- エラーは`?`演算子で伝播

**MVCパターンの理解:**
```
リクエスト
    ↓
コントローラー (config::show)
    ↓
サービス (ConfigService::get_urls)
    ↓
ファイル (telegraf.conf)
    ↓
レスポンス (HTML)
```

---

## 次のステップ

次の章では、**管理画面トップページ**を実装します。認証が必要なページの作り方を学びます。

[第6章: 管理画面トップページの実装](./chapter06_admin_dashboard.md)に進みましょう！
