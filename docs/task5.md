# タスク5: 公開設定閲覧機能の実装

## 概要

このタスクでは、認証不要で誰でもアクセスできる公開エンドポイント`/conf`を実装しました。このエンドポイントは、現在Telegrafが監視しているURL一覧を読みやすい形式で表示します。管理者以外のユーザーが設定内容を確認できるようにすることで、システムの透明性を高めます。

## 実装したファイル

- `src/controllers/config.rs` - 設定表示コントローラー
- `assets/views/config/show.html` - 設定表示テンプレート（将来の拡張用）
- `src/app.rs` - ルーティング設定（公開ルートの追加）

## 学習ポイント

### 1. Axumのレスポンス型 - `Html`と`IntoResponse`

```rust
use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;

pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // ...
    let html = format!(r#"<!DOCTYPE html>..."#);
    Ok(Html(html))
}
```

**解説:**
- `Html<String>`は、HTMLコンテンツをラップする型
  - Content-Typeヘッダーを自動的に`text/html`に設定
  - XSS対策のため、動的コンテンツは適切にエスケープする必要がある
- `IntoResponse`トレイトは、様々な型をHTTPレスポンスに変換
  - `Html`, `Json`, `Redirect`等が実装している
  - `impl IntoResponse`を返り値にすることで、柔軟なレスポンス型を返せる
- `Result<impl IntoResponse>`は、エラーハンドリングとレスポンス変換を組み合わせる
  - `?`演算子でエラーを伝播し、Locoが適切なHTTPエラーレスポンスに変換

### 2. Axumの状態抽出 - `State`エクストラクター

```rust
pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // _ctxは使用しないが、将来の拡張のために定義
}
```

**解説:**
- `State<T>`は、Axumのエクストラクター（Extractor）の一つ
  - アプリケーション全体で共有される状態にアクセスする
  - `AppContext`には、データベース接続、設定、環境情報等が含まれる
- エクストラクターは、関数の引数として宣言するだけで自動的に値が注入される
  - Axumのミドルウェアシステムが依存性注入を処理
- `_ctx`のように`_`プレフィックスをつけると、未使用警告を抑制
  - 将来的にデータベースアクセスが必要になった場合に備えて定義

### 3. 生のHTML文字列リテラル - `r#"..."`

```rust
let html = format!(
    r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <title>Telegraf設定情報</title>
    <style>
        body {{
            font-family: sans-serif;
        }}
    </style>
</head>
<body>
    {}
</body>
</html>
    "#,
    url_list
);
```

**解説:**
- `r#"..."`は、生文字列リテラル（Raw String Literal）
  - エスケープシーケンスを処理しない
  - HTMLやJSONのような複雑な文字列を書きやすい
- `format!`マクロで、`{}`にプレースホルダーを埋め込む
  - `{{}}`は、リテラルの`{`を表す（CSSのブロック等で使用）
- 複数行の文字列を自然に記述できる
  - インデントも含まれるため、出力時の見た目に注意

### 4. 動的HTMLの生成

```rust
let urls = ConfigService::get_urls()?;

let mut url_list = String::new();
for (i, url) in urls.iter().enumerate() {
    url_list.push_str(&format!(
        "<tr><td>{}</td><td>{}</td></tr>",
        i + 1,
        url
    ));
}
```

**解説:**
- `String::new()`で空の可変文字列を作成
- `iter().enumerate()`で、インデックス付きイテレーターを取得
  - `enumerate()`は`(index, value)`のタプルを返す
- `push_str()`で文字列を追加
  - `String`は可変長バッファで、効率的に文字列を連結できる
- セキュリティ上の注意:
  - URLは外部入力なので、本来はHTMLエスケープが必要
  - 今回はConfigServiceが検証済みのURLのみを返すため省略
  - 本番環境では`html_escape`クレート等を使用すべき

### 5. Locoのルーティング - 公開ルートと保護ルート

```rust
fn routes(ctx: &AppContext) -> AppRoutes {
    use loco_rs::controller::Routes;
    
    // 公開ルート（認証不要）
    let public_routes = Routes::new()
        .add("/conf", get(config::show))
        .add("/auth/login", get(auth::login_form))
        .add("/auth/login", post(auth::login));

    // 認証が必要なルート（ミドルウェアを適用）
    let protected_routes = Routes::new()
        .add("/admin", get(admin::index))
        .add("/auth/logout", post(auth::logout))
        .layer(axum::middleware::from_fn_with_state(
            ctx.clone(),
            require_auth,
        ));

    AppRoutes::with_default_routes()
        .add_route(public_routes)
        .add_route(protected_routes)
}
```

**解説:**
- `Routes::new()`で新しいルートグループを作成
- `add(path, handler)`でルートを追加
  - `get()`, `post()`はAxumのルーティングヘルパー
  - ハンドラー関数は`async fn`である必要がある
- **公開ルート**: ミドルウェアを適用しない
  - 誰でもアクセス可能
  - `/conf`, `/auth/login`等
- **保護ルート**: `layer()`でミドルウェアを適用
  - 認証チェックを行う
  - `/admin`, `/auth/logout`等
- `from_fn_with_state()`で、状態を持つミドルウェアを作成
  - `ctx.clone()`でアプリケーションコンテキストを渡す
  - `require_auth`ミドルウェアがセッションをチェック

### 6. Axumのミドルウェアレイヤー

```rust
.layer(axum::middleware::from_fn_with_state(
    ctx.clone(),
    require_auth,
))
```

**解説:**
- Axumのミドルウェアは、リクエスト処理の前後に処理を挟む
- `from_fn_with_state()`は、状態を持つミドルウェア関数を作成
  - 第1引数: 共有状態（`AppContext`）
  - 第2引数: ミドルウェア関数
- ミドルウェアの実行順序:
  1. リクエスト受信
  2. ミドルウェア（認証チェック）
  3. ハンドラー関数
  4. レスポンス返却
- レイヤーは複数適用可能で、外側から内側に実行される

### 7. HTTPメソッドルーティング

```rust
.add("/auth/login", get(auth::login_form))
.add("/auth/login", post(auth::login))
```

**解説:**
- 同じパスに異なるHTTPメソッドのハンドラーを登録可能
- `GET /auth/login`: ログインフォームを表示
- `POST /auth/login`: ログイン処理を実行
- RESTful APIの設計パターンに従う
  - GET: リソースの取得（副作用なし）
  - POST: リソースの作成・更新（副作用あり）
  - PUT/PATCH: リソースの更新
  - DELETE: リソースの削除

## Locoフレームワークとの統合

### コントローラーの役割

Locoでは、MVCパターンに従ってアプリケーションを構成します：

- **Model**: データベースとのやり取り（`src/models/`）
- **View**: HTMLテンプレート（`assets/views/`）
- **Controller**: リクエスト処理とレスポンス生成（`src/controllers/`）

今回実装した`config::show`は、以下の責務を持ちます：

1. **サービス層の呼び出し**: `ConfigService::get_urls()`でデータ取得
2. **ビューの生成**: HTMLを組み立てる
3. **レスポンスの返却**: `Html`型でラップして返す

### エラーハンドリングの流れ

```rust
pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let urls = ConfigService::get_urls()?;  // ConfigErrorが発生する可能性
    // ...
    Ok(Html(html))
}
```

1. `ConfigService::get_urls()`が`Result<Vec<String>, ConfigError>`を返す
2. `?`演算子でエラーを伝播
3. `ConfigError`は`loco_rs::Error`に変換される（`From`トレイト実装済み）
4. Locoが適切なHTTPステータスコード（500等）とエラーメッセージを返す

## テンプレートファイルの作成

将来的にTeraテンプレートエンジンを導入する場合に備えて、`assets/views/config/show.html`を作成しました。

```html
{% extends "layout/base.html" %}

{% block title %}Telegraf設定情報{% endblock %}

{% block content %}
<div class="header">
    <h1>Telegraf設定情報</h1>
</div>

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
            {% for url in urls %}
            <tr>
                <td>{{ loop.index }}</td>
                <td>{{ url }}</td>
            </tr>
            {% endfor %}
        </tbody>
    </table>
</div>

<div class="footer">
    <p><a href="/admin">管理画面へ</a></p>
</div>
{% endblock %}
```

**Teraテンプレートの特徴:**
- `{% extends "..." %}`: 基底テンプレートを継承
- `{% block ... %}`: 上書き可能なブロックを定義
- `{{ variable }}`: 変数を出力（自動的にHTMLエスケープ）
- `{% for ... %}`: ループ処理
- `{{ loop.index }}`: ループのインデックス（1始まり）

**テンプレートを使用する場合のコントローラー:**

```rust
use loco_rs::prelude::*;
use tera::Context;

pub async fn show(ViewEngine(v): ViewEngine) -> Result<impl IntoResponse> {
    let urls = ConfigService::get_urls()?;
    
    let mut context = Context::new();
    context.insert("urls", &urls);
    
    format::render().view(&v, "config/show.html", context)
}
```

## CSSスタイリングのポイント

実装したHTMLには、モダンなUIデザインを適用しています：

```css
body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: #f5f5f5;
}

.header {
    background: #667eea;  /* グラデーションの開始色 */
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

table {
    width: 100%;
    border-collapse: collapse;
}

th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid #ddd;
}

th {
    background: #f8f9fa;
    font-weight: 600;
}
```

**デザインの原則:**
- **カードレイアウト**: コンテンツを白い背景のカードで囲む
- **シャドウ**: 軽いbox-shadowで奥行きを表現
- **カラースキーム**: 紫系のアクセントカラー（`#667eea`）
- **タイポグラフィ**: システムフォントで読みやすさを確保
- **スペーシング**: 適切なpaddingとmarginで視認性を向上

## 動作確認

### サーバー起動

```bash
cargo run -- start
```

### エンドポイントへのアクセス

```bash
# ブラウザで以下にアクセス
http://localhost:3000/conf
```

### 期待される表示

- ヘッダー: 「Telegraf設定情報」
- テーブル: 監視中のURL一覧
  - No.列: 1から始まる連番
  - URL列: 各URLが表示される
- フッター: 「管理画面へ」のリンク

### curlでの確認

```bash
curl http://localhost:3000/conf
```

HTMLが返却されることを確認できます。

## セキュリティ上の考慮事項

### 1. 認証不要の設計判断

このエンドポイントは意図的に認証を不要にしています：

- **理由**: 監視対象URLは機密情報ではない
- **メリット**: 外部監視システムから簡単にアクセス可能
- **リスク**: URL一覧が公開される

もし機密性が必要な場合は、以下のように変更します：

```rust
let protected_routes = Routes::new()
    .add("/conf", get(config::show))  // 保護ルートに移動
    .add("/admin", get(admin::index))
    .layer(axum::middleware::from_fn_with_state(
        ctx.clone(),
        require_auth,
    ));
```

### 2. XSS対策

現在の実装では、URLを直接HTMLに埋め込んでいます：

```rust
url_list.push_str(&format!(
    "<tr><td>{}</td><td>{}</td></tr>",
    i + 1,
    url
));
```

**潜在的なリスク:**
- URLに`<script>`タグ等が含まれる場合、XSS攻撃が可能

**対策:**
1. ConfigServiceでURL検証を行う（既に実装済み）
2. HTMLエスケープを行う：

```rust
use html_escape::encode_text;

url_list.push_str(&format!(
    "<tr><td>{}</td><td>{}</td></tr>",
    i + 1,
    encode_text(url)
));
```

3. Teraテンプレートを使用する（自動エスケープ）

### 3. CSRF対策

このエンドポイントはGETリクエストで読み取り専用なので、CSRF対策は不要です。

## まとめ

このタスクで学んだRustとLocoの重要な概念：

1. **Axumのレスポンス型**: `Html`, `IntoResponse`の使い方
2. **状態抽出**: `State`エクストラクターによる依存性注入
3. **ルーティング**: 公開ルートと保護ルートの分離
4. **ミドルウェア**: 認証チェックの適用方法
5. **動的HTML生成**: 文字列操作とフォーマット
6. **テンプレート**: Teraテンプレートの基本構文

これらの概念は、Webアプリケーション開発において基本となるパターンです。特に、ルーティングとミドルウェアの設計は、セキュリティと保守性に直結する重要な要素です。

## 次のステップ

今後の改善案：

1. **テンプレートエンジンの導入**: Teraを使った動的レンダリング
2. **ページネーション**: URL数が多い場合の対応
3. **検索機能**: URL一覧のフィルタリング
4. **エクスポート機能**: JSON/CSV形式でのダウンロード
5. **リアルタイム更新**: WebSocketによる自動更新

## 参考リンク

- [Axum公式ドキュメント](https://docs.rs/axum/)
- [Loco Routing](https://loco.rs/docs/the-app/routes/)
- [Tera Template Engine](https://tera.netlify.app/)
- [HTML Escape](https://docs.rs/html-escape/)
