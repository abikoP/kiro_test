# タスク6: 管理画面トップページの実装

## 概要

このタスクでは、認証済みユーザーがアクセスできる管理画面のトップページ（ダッシュボード）を実装しました。このページは、システムの概要情報（監視中のURL数など）を表示し、各管理機能へのナビゲーションを提供します。ユーザーが直感的にシステムを操作できるよう、モダンなUIデザインを採用しています。

## 実装したファイル

- `src/controllers/admin.rs` - 管理画面コントローラー（更新）
- `assets/views/admin/index.html` - 管理画面テンプレート（新規作成）
- `src/app.rs` - ルーティング設定（既存の確認）

## 学習ポイント

### 1. サービス層からのデータ取得とエラーハンドリング

```rust
use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;
use crate::services::config_service::ConfigService;

pub async fn index(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // ダッシュボード情報を取得
    let url_count = match ConfigService::get_urls() {
        Ok(urls) => urls.len(),
        Err(_) => 0,
    };
    // ...
}
```

**解説:**
- `ConfigService::get_urls()`は`Result<Vec<String>, ConfigError>`を返す
- `match`式でエラーハンドリングを行う
  - `Ok(urls)`: 成功時はURL数を取得（`urls.len()`）
  - `Err(_)`: エラー時はデフォルト値`0`を使用
- この設計により、設定ファイルが存在しない場合でもページが表示される
  - ユーザーにエラーページではなく、正常なダッシュボードを表示
  - エラーの詳細は無視し、フォールバック値を使用

**`?`演算子との違い:**

```rust
// ?演算子を使う場合（エラーを伝播）
let urls = ConfigService::get_urls()?;  // エラー時は500エラーページ
let url_count = urls.len();

// matchを使う場合（エラーを処理）
let url_count = match ConfigService::get_urls() {
    Ok(urls) => urls.len(),
    Err(_) => 0,  // エラー時もページを表示
};
```

**使い分けの指針:**
- **`?`演算子**: エラーが致命的で、処理を続行できない場合
- **`match`式**: エラーを回復可能で、デフォルト値で処理を続行できる場合

### 2. 動的データの埋め込み - `format!`マクロ

```rust
let html = format!(
    r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <title>管理画面 - Telegraf設定管理</title>
</head>
<body>
    <div class="stat-box">
        <h3>監視中のURL数</h3>
        <div class="value">{}</div>
    </div>
</body>
</html>
    "#,
    url_count
);
Ok(Html(html))
```

**解説:**
- `format!`マクロは、文字列テンプレートに値を埋め込む
  - `{}`がプレースホルダー
  - 引数の順序で値が埋め込まれる
- `r#"..."`は生文字列リテラル
  - エスケープ不要でHTMLを記述できる
  - `{{}}`はリテラルの`{`を表す（CSS用）
- `Html(html)`で`String`を`Html<String>`型にラップ
  - Content-Typeヘッダーが`text/html`に設定される

**複数の値を埋め込む場合:**

```rust
let html = format!(
    r#"<p>URL数: {}, ユーザー数: {}</p>"#,
    url_count,
    user_count
);
```

**名前付きプレースホルダー:**

```rust
let html = format!(
    r#"<p>URL数: {count}, 最終更新: {time}</p>"#,
    count = url_count,
    time = last_updated
);
```

### 3. ダッシュボードUIの設計パターン

```html
<div class="dashboard-stats">
    <div class="stat-box">
        <h3>監視中のURL数</h3>
        <div class="value">{}</div>
    </div>
</div>
```

**CSSグリッドレイアウト:**

```css
.dashboard-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin: 1.5rem 0;
}
```

**解説:**
- `display: grid`: CSS Gridレイアウトを使用
- `grid-template-columns`: カラムの定義
  - `repeat(auto-fit, ...)`: 利用可能なスペースに自動的にフィット
  - `minmax(200px, 1fr)`: 最小200px、最大は均等分割
  - レスポンシブデザインが自動的に実現される
- `gap: 1rem`: グリッドアイテム間の間隔

**統計ボックスのスタイリング:**

```css
.stat-box {
    background: #f8f9fa;
    padding: 1.5rem;
    border-radius: 8px;
    border-left: 4px solid #667eea;  /* アクセントカラー */
}

.stat-box .value {
    font-size: 2rem;
    font-weight: 700;
    color: #333;
}
```

**デザインの原則:**
- **視覚的階層**: 数値を大きく表示し、ラベルを小さく
- **カラーアクセント**: 左ボーダーで重要度を強調
- **カードデザイン**: 背景色とシャドウで情報をグループ化

### 4. ナビゲーションバーの実装

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

**解説:**
- **リンクナビゲーション**: `<a>`タグでページ遷移
  - GETリクエストで状態を変更しない
  - ブラウザの戻るボタンが正常に動作
- **フォームによるログアウト**: `<form method="POST">`
  - POSTリクエストで状態を変更（セッション削除）
  - CSRF対策が必要（将来的に実装）
- **インラインフォーム**: `display: inline`でナビゲーションバーに配置

**CSSスタイリング:**

```css
.nav {
    background: white;
    padding: 1rem 2rem;
    box-shadow: 0 2px 4px rgba(0,0,0,0.05);
}

.nav a {
    color: #667eea;
    text-decoration: none;
    margin-right: 1.5rem;
    font-weight: 500;
}

.nav a:hover {
    text-decoration: underline;
}

.logout-form {
    display: inline;
}

.logout-btn {
    background: #e74c3c;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 5px;
    cursor: pointer;
}
```

**ユーザビリティのポイント:**
- **ホバー効果**: リンクにマウスを乗せると下線が表示
- **視覚的フィードバック**: ボタンの色で操作の重要度を示す
  - 通常のリンク: 紫色（`#667eea`）
  - ログアウト: 赤色（`#e74c3c`）- 破壊的操作を示す
- **一貫性**: すべてのページで同じナビゲーションを表示

### 5. クイックアクションボタン

```html
<div class="quick-actions">
    <a href="/admin/list">URL一覧を見る</a>
    <a href="/admin/edit">URLを編集する</a>
</div>
```

```css
.quick-actions a {
    display: inline-block;
    background: #667eea;
    color: white;
    padding: 0.75rem 1.5rem;
    border-radius: 5px;
    text-decoration: none;
    margin-right: 1rem;
    margin-bottom: 0.5rem;
    transition: background 0.3s;
}

.quick-actions a:hover {
    background: #5568d3;
}
```

**解説:**
- **ボタン風リンク**: `<a>`タグをボタンのようにスタイリング
  - `display: inline-block`: paddingとmarginが適用される
  - `text-decoration: none`: 下線を削除
- **トランジション効果**: `transition: background 0.3s`
  - ホバー時に背景色が滑らかに変化
  - ユーザーに視覚的フィードバックを提供
- **レスポンシブ対応**: `margin-bottom: 0.5rem`
  - 画面幅が狭い場合、ボタンが折り返される

### 6. Locoのルーティングと認証ミドルウェア

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
        .add("/admin", get(admin::index))  // ← 今回実装したエンドポイント
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
- `/admin`エンドポイントは`protected_routes`に含まれる
- `layer()`でミドルウェアを適用
  - すべての`protected_routes`に認証チェックが適用される
  - ミドルウェアは外側から内側に実行される

**ミドルウェアの実行フロー:**

```
1. リクエスト受信: GET /admin
2. ミドルウェア実行: require_auth()
   - セッションをチェック
   - ユーザーIDが存在するか確認
3a. 認証成功: admin::index()を実行
3b. 認証失敗: /auth/loginにリダイレクト
4. レスポンス返却
```

**`require_auth`ミドルウェアの実装（参考）:**

```rust
pub async fn require_auth(
    State(ctx): State<AppContext>,
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    // セッションからユーザーIDを取得
    match session.get::<i32>(SESSION_USER_ID_KEY).await {
        Ok(Some(_user_id)) => {
            // 認証成功: 次のハンドラーを実行
            next.run(request).await
        }
        _ => {
            // 認証失敗: ログインページにリダイレクト
            Redirect::to("/auth/login").into_response()
        }
    }
}
```

### 7. `State`エクストラクターと依存性注入

```rust
pub async fn index(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // _ctxは現在使用していないが、将来の拡張のために定義
}
```

**解説:**
- `State<AppContext>`は、Axumのエクストラクター
  - アプリケーション全体で共有される状態にアクセス
  - `AppContext`には以下が含まれる:
    - データベース接続プール（`db: DatabaseConnection`）
    - 設定情報（`config: Config`）
    - 環境情報（`environment: Environment`）
- `_ctx`のプレフィックス`_`は、未使用警告を抑制
  - 将来的にデータベースアクセスが必要になる可能性がある
  - 例: ユーザー情報の取得、アクティビティログの記録

**将来の拡張例:**

```rust
pub async fn index(State(ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // データベースからユーザー情報を取得
    let user_count = User::count(&ctx.db).await?;
    
    // 設定ファイルからURL数を取得
    let url_count = match ConfigService::get_urls() {
        Ok(urls) => urls.len(),
        Err(_) => 0,
    };
    
    // 複数の統計情報を表示
    let html = format!(
        r#"
        <div class="stat-box">
            <h3>監視中のURL数</h3>
            <div class="value">{}</div>
        </div>
        <div class="stat-box">
            <h3>登録ユーザー数</h3>
            <div class="value">{}</div>
        </div>
        "#,
        url_count,
        user_count
    );
    Ok(Html(html))
}
```

## テンプレートファイルの作成

将来的にTeraテンプレートエンジンを導入する場合に備えて、`assets/views/admin/index.html`を作成しました。

```html
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <title>管理画面 - Telegraf設定管理</title>
    <style>
        /* スタイル定義 */
    </style>
</head>
<body>
    <div class="header">
        <h1>Telegraf設定管理システム</h1>
    </div>
    <div class="nav">
        <a href="/admin">ダッシュボード</a>
        <a href="/admin/list">URL一覧</a>
        <a href="/admin/edit">URL編集</a>
        <form method="POST" action="/auth/logout" class="logout-form">
            <button type="submit" class="logout-btn">ログアウト</button>
        </form>
    </div>
    <div class="container">
        <div class="card">
            <h2>ダッシュボード</h2>
            <div class="dashboard-stats">
                <div class="stat-box">
                    <h3>監視中のURL数</h3>
                    <div class="value">{{ url_count }}</div>
                </div>
            </div>
        </div>
        <div class="card">
            <h2>管理画面へようこそ</h2>
            <p>このシステムでは、Telegrafの設定ファイル内のHTTP監視URLを管理できます。</p>
            <div class="quick-actions">
                <a href="/admin/list">URL一覧を見る</a>
                <a href="/admin/edit">URLを編集する</a>
            </div>
        </div>
    </div>
</body>
</html>
```

**Teraテンプレートを使用する場合のコントローラー:**

```rust
use loco_rs::prelude::*;
use tera::Context;

pub async fn index(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine,
) -> Result<impl IntoResponse> {
    // ダッシュボード情報を取得
    let url_count = match ConfigService::get_urls() {
        Ok(urls) => urls.len(),
        Err(_) => 0,
    };
    
    // テンプレートコンテキストを作成
    let mut context = Context::new();
    context.insert("url_count", &url_count);
    
    // テンプレートをレンダリング
    format::render().view(&v, "admin/index.html", context)
}
```

**Teraテンプレートのメリット:**
- **HTMLとロジックの分離**: コントローラーはデータ取得に専念
- **自動HTMLエスケープ**: XSS対策が自動的に適用される
- **テンプレート継承**: 共通レイアウトを再利用できる
- **デザイナーとの協業**: HTMLファイルを直接編集可能

## CSSスタイリングの詳細解説

### カラースキーム

```css
:root {
    --primary-color: #667eea;      /* メインカラー（紫） */
    --primary-hover: #5568d3;      /* ホバー時の濃い紫 */
    --danger-color: #e74c3c;       /* 危険な操作（赤） */
    --danger-hover: #c0392b;       /* ホバー時の濃い赤 */
    --background: #f5f5f5;         /* 背景色（グレー） */
    --card-background: white;      /* カード背景（白） */
    --text-primary: #333;          /* メインテキスト */
    --text-secondary: #666;        /* サブテキスト */
    --border-color: #ddd;          /* ボーダー色 */
}
```

**カラー選択の理由:**
- **紫系（`#667eea`）**: 信頼性と革新性を表現
- **赤系（`#e74c3c`）**: 注意が必要な操作を明示
- **グレー系**: 落ち着いた背景で可読性を向上

### レスポンシブデザイン

```css
.container {
    max-width: 1200px;
    margin: 2rem auto;
    padding: 0 2rem;
}

@media (max-width: 768px) {
    .container {
        padding: 0 1rem;
    }
    
    .nav a {
        display: block;
        margin-bottom: 0.5rem;
    }
    
    .dashboard-stats {
        grid-template-columns: 1fr;
    }
}
```

**ブレークポイント:**
- **デスクトップ（1200px以上）**: 複数カラムのグリッド
- **タブレット（768px〜1200px）**: 自動調整
- **モバイル（768px以下）**: 1カラムレイアウト

### シャドウとエレベーション

```css
.card {
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);  /* 軽いシャドウ */
}

.card:hover {
    box-shadow: 0 4px 8px rgba(0,0,0,0.15);  /* ホバー時に強調 */
}
```

**マテリアルデザインの原則:**
- **エレベーション**: シャドウで要素の階層を表現
- **インタラクション**: ホバー時にシャドウを強調
- **一貫性**: すべてのカードで同じシャドウを使用

## 動作確認

### サーバー起動

```bash
cargo run -- start
```

### エンドポイントへのアクセス

1. **ログインページにアクセス**
   ```
   http://localhost:3000/auth/login
   ```

2. **ログイン情報を入力**
   - ユーザー名: `admin`
   - パスワード: `password`

3. **管理画面トップページにリダイレクト**
   ```
   http://localhost:3000/admin
   ```

### 期待される表示

- **ヘッダー**: 「Telegraf設定管理システム」
- **ナビゲーションバー**:
  - ダッシュボード（現在のページ）
  - URL一覧
  - URL編集
  - ログアウトボタン
- **ダッシュボードカード**:
  - 監視中のURL数（統計ボックス）
- **ウェルカムカード**:
  - システムの説明
  - クイックアクションボタン

### curlでの確認

```bash
# 認証なしでアクセス（リダイレクトされる）
curl -i http://localhost:3000/admin

# 期待される結果: 302 Found
# Location: /auth/login

# セッション付きでアクセス
curl -i -b cookies.txt http://localhost:3000/admin

# 期待される結果: 200 OK
# Content-Type: text/html
```

## セキュリティ上の考慮事項

### 1. 認証の必須化

```rust
let protected_routes = Routes::new()
    .add("/admin", get(admin::index))
    .layer(axum::middleware::from_fn_with_state(
        ctx.clone(),
        require_auth,
    ));
```

**保護の仕組み:**
- すべての`/admin/*`エンドポイントに認証が必要
- セッションにユーザーIDが存在しない場合、ログインページにリダイレクト
- ミドルウェアがハンドラーの前に実行される

### 2. CSRF対策（将来の実装）

現在のログアウトフォームはCSRF攻撃に脆弱です：

```html
<form method="POST" action="/auth/logout" class="logout-form">
    <button type="submit" class="logout-btn">ログアウト</button>
</form>
```

**改善案:**

```html
<form method="POST" action="/auth/logout" class="logout-form">
    <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
    <button type="submit" class="logout-btn">ログアウト</button>
</form>
```

**ミドルウェアでの検証:**

```rust
pub async fn verify_csrf(
    session: Session,
    Form(form): Form<LogoutForm>,
) -> Result<()> {
    let session_token = session.get::<String>("csrf_token").await?;
    if session_token != Some(form.csrf_token) {
        return Err(Error::Unauthorized);
    }
    Ok(())
}
```

### 3. セッション管理

```rust
use tower_sessions::{Session, MemoryStore};

// セッションストアの設定
let session_store = MemoryStore::default();
let session_layer = SessionLayer::new(session_store)
    .with_secure(true)  // HTTPS必須
    .with_http_only(true)  // JavaScriptからアクセス不可
    .with_same_site(SameSite::Strict);  // CSRF対策
```

**セキュリティ設定:**
- **Secure**: HTTPS接続でのみCookieを送信
- **HttpOnly**: JavaScriptからCookieにアクセス不可（XSS対策）
- **SameSite**: クロスサイトリクエストでCookieを送信しない（CSRF対策）

### 4. XSS対策

現在の実装では、動的データは数値のみです：

```rust
let html = format!(
    r#"<div class="value">{}</div>"#,
    url_count  // 数値なのでXSSリスクなし
);
```

**文字列を埋め込む場合の対策:**

```rust
use html_escape::encode_text;

let html = format!(
    r#"<div class="value">{}</div>"#,
    encode_text(&user_input)  // HTMLエスケープ
);
```

## パフォーマンス最適化

### 1. 静的アセットのキャッシュ

現在はインラインCSSを使用していますが、本番環境では外部ファイルを推奨：

```html
<link rel="stylesheet" href="/static/css/admin.css">
```

**Axumでの静的ファイル配信:**

```rust
use tower_http::services::ServeDir;

let app = Router::new()
    .nest_service("/static", ServeDir::new("assets/static"))
    .merge(routes);
```

**キャッシュヘッダーの設定:**

```rust
use tower_http::set_header::SetResponseHeaderLayer;
use http::header::{CACHE_CONTROL, HeaderValue};

let static_files = ServeDir::new("assets/static")
    .layer(SetResponseHeaderLayer::if_not_present(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000"),
    ));
```

### 2. データベースクエリの最適化

将来的にデータベースから統計情報を取得する場合：

```rust
// 非効率: 複数のクエリ
let url_count = Url::count(&ctx.db).await?;
let user_count = User::count(&ctx.db).await?;
let log_count = Log::count(&ctx.db).await?;

// 効率的: 1つのクエリで複数の統計を取得
let stats = sqlx::query!(
    r#"
    SELECT
        (SELECT COUNT(*) FROM urls) as url_count,
        (SELECT COUNT(*) FROM users) as user_count,
        (SELECT COUNT(*) FROM logs) as log_count
    "#
)
.fetch_one(&ctx.db)
.await?;
```

### 3. HTMLの最小化

本番環境では、HTMLを最小化してサイズを削減：

```rust
use minify_html::{Cfg, minify};

let html = format!(r#"<!DOCTYPE html>..."#, url_count);
let minified = minify(html.as_bytes(), &Cfg::default());
Ok(Html(String::from_utf8(minified)?))
```

## まとめ

このタスクで学んだRustとLocoの重要な概念：

1. **エラーハンドリング**: `match`式によるグレースフルなエラー処理
2. **動的HTML生成**: `format!`マクロによるテンプレート
3. **ダッシュボードUI**: CSS Gridによるレスポンシブデザイン
4. **ナビゲーション**: リンクとフォームの使い分け
5. **認証ミドルウェア**: ルートグループへの適用
6. **状態管理**: `State`エクストラクターによる依存性注入

これらの概念は、管理画面やダッシュボードを構築する際の基本パターンです。特に、エラーハンドリングとUIデザインは、ユーザーエクスペリエンスに直結する重要な要素です。

## 次のステップ

今後の改善案：

1. **リアルタイム更新**: WebSocketで統計情報を自動更新
2. **グラフ表示**: Chart.jsでURL数の推移を可視化
3. **通知機能**: 重要なイベントをダッシュボードに表示
4. **カスタマイズ**: ユーザーごとにダッシュボードをカスタマイズ
5. **エクスポート**: 統計情報をPDF/CSVでエクスポート
6. **アクセスログ**: 最近のアクティビティを表示
7. **システムステータス**: Telegrafの稼働状況を監視

## 参考リンク

- [Axum Extractors](https://docs.rs/axum/latest/axum/extract/index.html)
- [Loco Authentication](https://loco.rs/docs/the-app/authentication/)
- [CSS Grid Layout](https://developer.mozilla.org/ja/docs/Web/CSS/CSS_Grid_Layout)
- [Material Design](https://material.io/design)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
