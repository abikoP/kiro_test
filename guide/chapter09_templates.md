# 第9章: テンプレートエンジンとレイアウト

## この章で学ぶこと

- Teraテンプレートエンジンの基本
- テンプレート継承とブロック
- 共通レイアウトの作成
- エラーページの実装
- HTMLからテンプレートへの移行

---

## 9.1 なぜテンプレートエンジンを使うのか

### これまでの問題点

第5章から第8章まで、私たちはHTMLを文字列として直接生成してきました：

```rust
let html = format!(
    r#"
<!DOCTYPE html>
<html>
<head><title>ページタイトル</title></head>
<body>
    <h1>{}</h1>
</body>
</html>
    "#,
    title
);
```

**この方法の問題点：**
- コードが読みにくい
- HTMLとRustのコードが混在
- 共通部分（ヘッダー、フッター）を毎回書く必要がある
- デザイナーとの協業が難しい
- 変更時に複数のファイルを修正する必要がある

### テンプレートエンジンの利点

**Teraテンプレートエンジン**を使うと：

✅ HTMLとロジックを分離できる
✅ テンプレート継承で共通部分を再利用
✅ 変数、ループ、条件分岐が使える
✅ 自動HTMLエスケープでXSS対策
✅ デザイナーとの協業が容易

---

## 9.2 Teraテンプレートの基本

### Teraとは

**Tera**は、Jinja2（Python）やTwig（PHP）に影響を受けたRust用のテンプレートエンジンです。

**基本的な構文：**

```html
<!-- 変数の出力 -->
{{ variable_name }}

<!-- 条件分岐 -->
{% if condition %}
    <p>条件が真の場合</p>
{% else %}
    <p>条件が偽の場合</p>
{% endif %}

<!-- ループ -->
{% for item in items %}
    <li>{{ item }}</li>
{% endfor %}

<!-- コメント -->
{# これはコメントです #}
```


### 簡単な例

**テンプレートファイル（hello.html）：**
```html
<!DOCTYPE html>
<html lang="ja">
<head>
    <title>{{ title }}</title>
</head>
<body>
    <h1>こんにちは、{{ name }}さん！</h1>
    
    {% if is_admin %}
        <p>管理者権限があります</p>
    {% endif %}
    
    <ul>
    {% for item in items %}
        <li>{{ item }}</li>
    {% endfor %}
    </ul>
</body>
</html>
```

**Rustコード：**
```rust
use loco_rs::prelude::*;

pub async fn hello(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "hello.html", json!({
        "title": "ようこそ",
        "name": "太郎",
        "is_admin": true,
        "items": vec!["りんご", "バナナ", "オレンジ"]
    }))
}
```

**出力されるHTML：**
```html
<!DOCTYPE html>
<html lang="ja">
<head>
    <title>ようこそ</title>
</head>
<body>
    <h1>こんにちは、太郎さん！</h1>
    
    <p>管理者権限があります</p>
    
    <ul>
        <li>りんご</li>
        <li>バナナ</li>
        <li>オレンジ</li>
    </ul>
</body>
</html>
```

---

## 9.3 テンプレート継承

### テンプレート継承とは

**テンプレート継承**は、共通部分を親テンプレートに定義し、子テンプレートで必要な部分だけをオーバーライドする仕組みです。

### ベースレイアウトの作成

`assets/views/layout/base.html`を作成します：

```html
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Telegraf設定管理{% endblock %}</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background: #f5f5f5;
        }
        
        .site-header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 2rem;
            text-align: center;
        }
        
        .main-container {
            max-width: 1200px;
            margin: 2rem auto;
            padding: 0 2rem;
        }
        
        {% block extra_styles %}{% endblock %}
    </style>
</head>
<body>
    <header class="site-header">
        <h1>{% block header_title %}Telegraf設定管理システム{% endblock %}</h1>
    </header>
    
    {% block navigation %}{% endblock %}
    
    <main class="main-container">
        {% block content %}{% endblock %}
    </main>
    
    <footer class="site-footer">
        {% block footer %}
        <p>&copy; 2024 Telegraf設定管理システム</p>
        {% endblock %}
    </footer>
</body>
</html>
```

### ブロックの説明

**`{% block name %}`**は、子テンプレートでオーバーライド可能な領域を定義します。

- **`title`**: ページタイトル（ブラウザのタブに表示）
- **`header_title`**: ヘッダーのタイトル
- **`navigation`**: ナビゲーションバー
- **`content`**: メインコンテンツ（必須）
- **`footer`**: フッター
- **`extra_styles`**: ページ固有のCSS


### 子テンプレートの作成

ベースレイアウトを継承して、具体的なページを作成します。

`assets/views/admin/index.html`：

```html
{% extends "layout/base.html" %}

{% block title %}管理画面 - Telegraf設定管理{% endblock %}

{% block extra_styles %}
<style>
    .dashboard-stats {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 1.5rem;
    }
    
    .stat-box {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        padding: 2rem;
        border-radius: 8px;
        color: white;
    }
</style>
{% endblock %}

{% block content %}
<div class="card">
    <h2>ダッシュボード</h2>
    <div class="dashboard-stats">
        <div class="stat-box">
            <h3>監視中のURL</h3>
            <div class="value">{{ url_count }}</div>
        </div>
    </div>
</div>
{% endblock %}
```

**ポイント：**

1. **`{% extends "layout/base.html" %}`**
   - 親テンプレートを指定します
   - 必ず最初の行に書きます

2. **`{% block title %}`**
   - 親テンプレートの`title`ブロックをオーバーライド
   - ページ固有のタイトルを設定

3. **`{% block extra_styles %}`**
   - ページ固有のCSSを追加
   - 親テンプレートのCSSに追加される

4. **`{% block content %}`**
   - メインコンテンツを定義
   - これが必須のブロック

---

## 9.4 管理画面用レイアウト

### ナビゲーション付きレイアウト

認証済みユーザー向けに、ナビゲーションバー付きのレイアウトを作成します。

`assets/views/layout/admin.html`：

```html
{% extends "layout/base.html" %}

{% block navigation %}
<nav class="site-nav">
    <div class="nav-container">
        <ul class="nav-links">
            <li>
                <a href="/admin" 
                   {% if current_page == "dashboard" %}class="active"{% endif %}>
                    ダッシュボード
                </a>
            </li>
            <li>
                <a href="/admin/list" 
                   {% if current_page == "list" %}class="active"{% endif %}>
                    URL一覧
                </a>
            </li>
            <li>
                <a href="/admin/edit" 
                   {% if current_page == "edit" %}class="active"{% endif %}>
                    URL編集
                </a>
            </li>
        </ul>
        <div class="nav-actions">
            <form method="POST" action="/auth/logout">
                <button type="submit" class="btn btn-danger">ログアウト</button>
            </form>
        </div>
    </div>
</nav>
{% endblock %}
```

**ポイント：**

- **`{% extends "layout/base.html" %}`**
  - `base.html`を継承
  - さらにこのテンプレートを継承することも可能（多段継承）

- **`{% if current_page == "dashboard" %}class="active"{% endif %}`**
  - 現在のページをハイライト
  - コントローラーから`current_page`変数を渡す

### 管理画面ページの作成

`admin.html`を継承してダッシュボードを作成：

```html
{% extends "layout/admin.html" %}

{% block title %}管理画面 - Telegraf設定管理{% endblock %}

{% block content %}
<div class="card">
    <h2>ダッシュボード</h2>
    <p>監視中のURL: {{ url_count }}件</p>
</div>
{% endblock %}
```

**継承の階層：**
```
base.html (基本レイアウト)
  ↓ extends
admin.html (ナビゲーション追加)
  ↓ extends
admin/index.html (ダッシュボード)
```

---

## 9.5 コントローラーでの使用

### ViewEngineの取得

Locoでは`ViewEngine`エクストラクタを使ってテンプレートエンジンにアクセスします。

```rust
use loco_rs::prelude::*;

pub async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render().view(&v, "admin/index.html", json!({
        "url_count": 10,
        "current_page": "dashboard"
    }))
}
```

### コードの解説

**`ViewEngine(v): ViewEngine<TeraView>`**
- パターンマッチングで`TeraView`を取り出します
- `v`がテンプレートエンジンのインスタンス

**`format::render()`**
- レスポンスビルダーを作成

**`.view(&v, "admin/index.html", json!({...}))`**
- テンプレートファイルのパスを指定
- `json!({})`マクロでデータを渡す


### データの渡し方

#### 単純な値

```rust
format::render().view(&v, "template.html", json!({
    "name": "太郎",
    "age": 25,
    "is_admin": true,
    "items": vec!["A", "B", "C"]
}))
```

#### 構造体を使用

```rust
use serde::Serialize;

#[derive(Serialize)]
struct PageData {
    title: String,
    urls: Vec<String>,
    count: usize,
}

let data = PageData {
    title: "URL一覧".to_string(),
    urls: vec!["https://example.com".to_string()],
    count: 1,
};

format::render().view(&v, "template.html", data)
```

#### Optionの扱い

```rust
format::render().view(&v, "template.html", json!({
    "error": error_message,  // Option<String>
    "success": success_message  // Option<String>
}))
```

テンプレート側：
```html
{% if error %}
<div class="alert alert-error">{{ error }}</div>
{% endif %}

{% if success %}
<div class="alert alert-success">{{ success }}</div>
{% endif %}
```

---

## 9.6 Teraの高度な機能

### フィルター

Teraには便利なフィルター関数があります。

```html
<!-- 文字列操作 -->
{{ name | upper }}  <!-- 大文字変換 -->
{{ name | lower }}  <!-- 小文字変換 -->
{{ text | truncate(length=50) }}  <!-- 切り詰め -->

<!-- 配列操作 -->
{{ urls | length }}  <!-- 配列の長さ -->
{{ urls | first }}  <!-- 最初の要素 -->
{{ urls | last }}   <!-- 最後の要素 -->
{{ urls | join(sep=", ") }}  <!-- 結合 -->

<!-- 数値操作 -->
{{ price | round }}  <!-- 四捨五入 -->
```

**使用例：**

```html
<p>現在 <strong>{{ urls | length }}</strong> 件のURLが登録されています。</p>

{% if urls | length > 0 %}
    <p>最初のURL: {{ urls | first }}</p>
{% endif %}
```

### ループ変数

ループ内では特別な変数が使えます。

```html
{% for url in urls %}
<tr class="{% if loop.index is odd %}odd{% else %}even{% endif %}">
    <td>{{ loop.index }}</td>
    <td>{{ url }}</td>
    <td>
        {% if loop.first %}最初{% endif %}
        {% if loop.last %}最後{% endif %}
    </td>
</tr>
{% endfor %}
```

**利用可能なループ変数：**
- `loop.index`: 1から始まるインデックス
- `loop.index0`: 0から始まるインデックス
- `loop.first`: 最初の要素かどうか（bool）
- `loop.last`: 最後の要素かどうか（bool）
- `loop.length`: ループの総数

### 条件分岐の応用

```html
<!-- 複数条件 -->
{% if user.is_admin and user.is_active %}
    <a href="/admin">管理画面</a>
{% elif user.is_active %}
    <a href="/dashboard">ダッシュボード</a>
{% else %}
    <p>アカウントが無効です</p>
{% endif %}

<!-- 存在チェック -->
{% if urls is defined and urls | length > 0 %}
    <!-- URLリストを表示 -->
{% endif %}

<!-- 否定 -->
{% if not is_logged_in %}
    <a href="/auth/login">ログイン</a>
{% endif %}
```

---

## 9.7 エラーページの実装

### 404 ページが見つかりません

`assets/views/errors/404.html`：

```html
{% extends "layout/base.html" %}

{% block title %}404 - ページが見つかりません{% endblock %}

{% block content %}
<div class="card">
    <div class="error-container">
        <div class="error-icon">🔍</div>
        <h1 class="error-code">404</h1>
        <p class="error-message">ページが見つかりません</p>
        <div class="error-description">
            <p>お探しのページは存在しないか、移動または削除された可能性があります。</p>
        </div>
        <div class="error-actions">
            <a href="/conf" class="btn btn-primary">公開ページへ</a>
            <a href="/admin" class="btn btn-secondary">管理画面へ</a>
        </div>
    </div>
</div>
{% endblock %}

{% block extra_styles %}
<style>
    .error-container {
        text-align: center;
        padding: 3rem;
    }
    
    .error-icon {
        font-size: 5rem;
        margin-bottom: 1rem;
    }
    
    .error-code {
        font-size: 6rem;
        font-weight: bold;
        color: #667eea;
        margin: 0;
    }
    
    .error-message {
        font-size: 1.5rem;
        color: #666;
        margin: 1rem 0;
    }
    
    .error-actions {
        margin-top: 2rem;
        display: flex;
        gap: 1rem;
        justify-content: center;
    }
</style>
{% endblock %}
```

### 500 サーバーエラー

`assets/views/errors/500.html`：

```html
{% extends "layout/base.html" %}

{% block title %}500 - サーバーエラー{% endblock %}

{% block content %}
<div class="card">
    <div class="error-container">
        <div class="error-icon">⚠️</div>
        <h1 class="error-code">500</h1>
        <p class="error-message">サーバーエラーが発生しました</p>
        
        {% if error_message %}
        <div class="error-details">
            <strong>エラー詳細：</strong>
            <pre>{{ error_message }}</pre>
        </div>
        {% endif %}
        
        <div class="error-actions">
            <a href="javascript:history.back()" class="btn btn-secondary">前のページに戻る</a>
            <a href="/conf" class="btn btn-primary">公開ページへ</a>
        </div>
    </div>
</div>
{% endblock %}
```

### コントローラーでの使用

```rust
// 404エラー
pub async fn not_found(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render()
        .status(StatusCode::NOT_FOUND)
        .view(&v, "errors/404.html", json!({}))
}

// 500エラー
pub async fn handle_error(
    ViewEngine(v): ViewEngine<TeraView>,
    err: Error,
) -> Response {
    // 開発環境ではエラー詳細を表示
    let error_message = if cfg!(debug_assertions) {
        Some(format!("{:?}", err))
    } else {
        None
    };
    
    format::render()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .view(&v, "errors/500.html", json!({
            "error_message": error_message
        }))
        .unwrap_or_else(|_| {
            // テンプレートレンダリングに失敗した場合
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal Server Error".into())
                .unwrap()
        })
}
```


---

## 9.8 HTMLからテンプレートへの移行

### 移行前（第7章のコード）

```rust
pub async fn index(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

    let mut url_rows = String::new();
    for (idx, url) in urls.iter().enumerate() {
        url_rows.push_str(&format!(
            r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
            </tr>
            "#,
            idx + 1,
            html_escape::encode_text(url)
        ));
    }

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <title>URL一覧</title>
    <!-- ... 長いHTML ... -->
</head>
<body>
    <!-- ... -->
    <table>
        <tbody>
            {}
        </tbody>
    </table>
</body>
</html>
        "#,
        url_rows
    );

    Ok(Html(html))
}
```

### 移行後（テンプレート使用）

**コントローラー（`src/controllers/admin_list.rs`）：**

```rust
use loco_rs::prelude::*;
use crate::services::config_service::ConfigService;

pub async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    let urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

    format::render().view(&v, "admin/list.html", json!({
        "urls": urls,
        "current_page": "list"
    }))
}
```

**テンプレート（`assets/views/admin/list.html`）：**

```html
{% extends "layout/admin.html" %}

{% block title %}URL一覧 - Telegraf設定管理{% endblock %}

{% block content %}
<div class="card">
    <h2>監視中のURL一覧</h2>
    
    {% if urls %}
    <p>現在 <strong>{{ urls | length }}</strong> 件のURLが監視対象として登録されています。</p>
    
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
                <td><code>{{ url }}</code></td>
            </tr>
            {% endfor %}
        </tbody>
    </table>
    {% else %}
    <div class="alert alert-info">
        現在、監視対象のURLは登録されていません。
    </div>
    {% endif %}
    
    <div class="actions">
        <a href="/admin/edit" class="btn btn-primary">URLを編集する</a>
    </div>
</div>
{% endblock %}
```

### 移行のメリット

**コード量の比較：**
- 移行前: 約100行（HTML文字列生成）
- 移行後: 約15行（コントローラー） + 30行（テンプレート）

**メリット：**
1. **可読性**: HTMLがHTMLとして読める
2. **保守性**: デザイン変更がテンプレートだけで完結
3. **再利用性**: 共通レイアウトを複数ページで共有
4. **安全性**: Teraが自動的にHTMLエスケープ
5. **協業**: デザイナーがテンプレートを編集可能

---

## 9.9 セキュリティ考慮事項

### 自動HTMLエスケープ

Teraはデフォルトで自動エスケープを行います。

```html
<!-- 安全：自動エスケープ -->
{{ user_input }}

<!-- 危険：エスケープを無効化（信頼できるHTMLのみ） -->
{{ trusted_html | safe }}
```

**例：**

```rust
format::render().view(&v, "template.html", json!({
    "user_input": "<script>alert('XSS')</script>"
}))
```

テンプレート：
```html
<p>{{ user_input }}</p>
```

出力：
```html
<p>&lt;script&gt;alert('XSS')&lt;/script&gt;</p>
```

### CSRF対策

Locoは自動的にCSRF保護を提供します。

```html
<form method="POST" action="/admin/edit">
    <!-- Locoが自動的にCSRFトークンを検証 -->
    <input type="text" name="url">
    <button type="submit">送信</button>
</form>
```

---

## 9.10 ディレクトリ構造

```
assets/
└── views/
    ├── layout/
    │   ├── base.html          # 共通ベースレイアウト
    │   └── admin.html         # 管理画面レイアウト
    ├── admin/
    │   ├── index.html         # ダッシュボード
    │   ├── list.html          # URL一覧
    │   └── edit.html          # URL編集
    ├── auth/
    │   └── login.html         # ログインページ
    ├── config/
    │   └── show.html          # 公開ページ
    └── errors/
        ├── 404.html           # ページが見つかりません
        ├── 401.html           # 認証エラー
        └── 500.html           # サーバーエラー
```

---

## 9.11 トラブルシューティング

### エラー: テンプレートが見つからない

```
Error: Template 'admin/index.html' not found
```

**解決方法：**
1. ファイルパスが正しいか確認（`assets/views/`からの相対パス）
2. ファイル名の大文字小文字を確認
3. ファイルが存在するか確認

### エラー: 変数が表示されない

```html
<!-- 空白が表示される -->
<p>{{ user_name }}</p>
```

**解決方法：**
1. コントローラーで変数を渡しているか確認
2. 変数名のスペルミスを確認
3. デバッグ用に`{% if user_name %}{{ user_name }}{% else %}変数なし{% endif %}`を使用

### エラー: レイアウト継承が機能しない

```
Error: Template 'layout/base.html' not found
```

**解決方法：**
1. `{% extends %}`のパスが正しいか確認
2. 親テンプレートが存在するか確認
3. `{% block %}`の名前が一致しているか確認

---

## 9.12 まとめ

この章では、以下を学びました：

✅ Teraテンプレートエンジンの基本構文
✅ テンプレート継承とブロック
✅ 共通レイアウトの作成
✅ コントローラーでのViewEngine使用
✅ フィルターとループ変数
✅ エラーページの実装
✅ HTMLからテンプレートへの移行

**重要なポイント：**
- テンプレート継承で共通部分を再利用
- `{% block %}`で柔軟なレイアウト設計
- Teraが自動的にHTMLエスケープ
- コードとデザインの分離

**テンプレートエンジンの利点：**
```
HTMLとロジックの分離 → 保守性向上
テンプレート継承 → コードの再利用
自動エスケープ → セキュリティ向上
```

---

## 次のステップ

次の章では、**統合とデプロイ**について学びます。これまで実装した機能を統合し、本番環境へのデプロイ方法を解説します。

[第10章: 統合とデプロイ](./chapter10_deployment.md)に進みましょう！
