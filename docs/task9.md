# タスク9: 共通レイアウトとスタイリング

## 概要

このタスクでは、Webアプリケーション全体で統一されたデザインとユーザー体験を提供するために、共通レイアウトテンプレートとスタイリングを実装しました。Locoフレームワークが採用しているTeraテンプレートエンジンを使用して、再利用可能なレイアウトとエラーページを作成しています。

## 実装内容

### 1. ベースレイアウトの作成

#### 1.1 共通ベースレイアウト（assets/views/layout/base.html）

すべてのページの基礎となるレイアウトテンプレートです。

```html
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Telegraf設定管理{% endblock %}</title>
    <style>
        /* 共通スタイル定義 */
        * {
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background: #f5f5f5;
            color: #333;
            line-height: 1.6;
        }
        
        /* ... その他のスタイル ... */
        
        {% block extra_styles %}{% endblock %}
    </style>
</head>
<body>
    <!-- ヘッダー -->
    <header class="site-header">
        <h1>{% block header_title %}Telegraf設定管理システム{% endblock %}</h1>
        <p>{% block header_subtitle %}HTTP監視URL設定を簡単に管理{% endblock %}</p>
    </header>
    
    <!-- ナビゲーション -->
    {% block navigation %}{% endblock %}
    
    <!-- メインコンテンツ -->
    <main class="main-container">
        {% block content %}{% endblock %}
    </main>
    
    <!-- フッター -->
    <footer class="site-footer">
        <div class="footer-container">
            {% block footer %}
            <p>&copy; 2024 Telegraf設定管理システム</p>
            <p>Powered by Loco Framework</p>
            {% endblock %}
        </div>
    </footer>
    
    {% block extra_scripts %}{% endblock %}
</body>
</html>
```

**Teraテンプレートの基本概念:**


- **`{% block name %}`**: 子テンプレートでオーバーライド可能なブロック
- **`{% extends "path" %}`**: 親テンプレートを継承
- **`{{ variable }}`**: 変数の出力
- **`{% if condition %}`**: 条件分岐
- **`{% for item in items %}`**: ループ処理

**ブロックの役割:**

1. **`title`**: ページタイトル（ブラウザタブに表示）
2. **`header_title`**: ヘッダーのメインタイトル
3. **`header_subtitle`**: ヘッダーのサブタイトル
4. **`navigation`**: ナビゲーションバー（ページごとに異なる）
5. **`content`**: メインコンテンツ（必須）
6. **`footer`**: フッター（カスタマイズ可能）
7. **`extra_styles`**: ページ固有のCSS
8. **`extra_scripts`**: ページ固有のJavaScript

**スタイリングの特徴:**

- **レスポンシブデザイン**: メディアクエリで768px以下の画面に対応
- **モダンなUI**: グラデーション、シャドウ、トランジション効果
- **ユーティリティクラス**: `.mt-1`, `.mb-2`等のマージンクラス
- **コンポーネントスタイル**: カード、ボタン、フォーム、テーブル、アラート

#### 1.2 管理画面用レイアウト（assets/views/layout/admin.html）

認証済みユーザー向けのナビゲーション付きレイアウトです。

```html
{% extends "layout/base.html" %}

{% block navigation %}
<nav class="site-nav">
    <div class="nav-container">
        <ul class="nav-links">
            <li><a href="/admin" {% if current_page == "dashboard" %}class="active"{% endif %}>ダッシュボード</a></li>
            <li><a href="/admin/list" {% if current_page == "list" %}class="active"{% endif %}>URL一覧</a></li>
            <li><a href="/admin/edit" {% if current_page == "edit" %}class="active"{% endif %}>URL編集</a></li>
        </ul>
        <div class="nav-actions">
            <form method="POST" action="/auth/logout" style="margin: 0;">
                <button type="submit" class="btn btn-danger">ログアウト</button>
            </form>
        </div>
    </div>
</nav>
{% endblock %}
```

**ポイント:**

- **テンプレート継承**: `base.html`を継承し、`navigation`ブロックのみをオーバーライド
- **アクティブページのハイライト**: `current_page`変数でナビゲーションの現在位置を表示
- **ログアウトフォーム**: POSTメソッドでCSRF対策

**コントローラーでの使用例:**

```rust
use loco_rs::prelude::*;

pub async fn dashboard(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    format::render().view(&v, "admin/index.html", json!({
        "current_page": "dashboard",
        "url_count": 10
    }))
}
```

### 2. ページテンプレートの実装

#### 2.1 ダッシュボード（assets/views/admin/index.html）

```html
{% extends "layout/admin.html" %}

{% block title %}管理画面 - Telegraf設定管理{% endblock %}

{% block extra_styles %}
<style>
    .dashboard-stats {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 1.5rem;
        margin: 1.5rem 0;
    }
    
    .stat-box {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        padding: 2rem;
        border-radius: 8px;
        color: white;
        box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
        transition: transform 0.3s;
    }
    
    .stat-box:hover {
        transform: translateY(-5px);
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
            <div class="label">アクティブな監視対象</div>
        </div>
    </div>
</div>
{% endblock %}
```

**解説:**

- **`extra_styles`ブロック**: ページ固有のCSSを追加
- **CSS Grid**: `grid-template-columns: repeat(auto-fit, minmax(250px, 1fr))`でレスポンシブなグリッドレイアウト
- **変数展開**: `{{ url_count }}`でコントローラーから渡されたデータを表示
- **ホバーエフェクト**: `transform: translateY(-5px)`で浮き上がる効果


#### 2.2 URL一覧ページ（assets/views/admin/list.html）

```html
{% extends "layout/admin.html" %}

{% block title %}URL一覧 - Telegraf設定管理{% endblock %}

{% block content %}
<div class="card">
    <h2>監視中のURL一覧</h2>
    
    {% if urls %}
    <p style="color: #666; margin-bottom: 1.5rem;">
        現在 <strong>{{ urls | length }}</strong> 件のURLが監視対象として登録されています。
    </p>
    
    <table>
        <thead>
            <tr>
                <th style="width: 80px;">No.</th>
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
</div>
{% endblock %}
```

**Teraのループと条件分岐:**

- **`{% if urls %}`**: 配列が空でない場合に実行
- **`{{ urls | length }}`**: フィルター関数で配列の長さを取得
- **`{% for url in urls %}`**: 配列をループ
- **`{{ loop.index }}`**: ループのインデックス（1から開始）
- **`{% else %}`**: 条件が偽の場合の処理

#### 2.3 URL編集ページ（assets/views/admin/edit.html）

```html
{% extends "layout/admin.html" %}

{% block title %}URL編集 - Telegraf設定管理{% endblock %}

{% block content %}
<div class="card">
    <h2>URL編集</h2>
    
    {% if success %}
    <div class="alert alert-success">
        {{ success }}
    </div>
    {% endif %}
    
    {% if errors %}
    <div class="alert alert-error">
        <strong>以下のエラーが発生しました：</strong>
        <ul style="margin: 0.5rem 0 0 1.5rem;">
            {% for err in errors %}
            <li>{{ err }}</li>
            {% endfor %}
        </ul>
    </div>
    {% endif %}
    
    <form method="POST" action="/admin/edit">
        <!-- 既存URLの削除 -->
        {% if urls and urls | length > 0 %}
        <ul class="url-list">
            {% for url in urls %}
            <li class="url-item">
                <input 
                    type="checkbox" 
                    name="delete_urls" 
                    value="{{ url }}" 
                    id="url_{{ loop.index }}"
                    onchange="this.parentElement.classList.toggle('marked-for-deletion', this.checked)"
                >
                <label for="url_{{ loop.index }}">{{ url }}</label>
            </li>
            {% endfor %}
        </ul>
        {% endif %}
        
        <!-- 新規URL追加 -->
        <div class="form-group">
            <textarea 
                name="new_urls" 
                class="form-control" 
                rows="8" 
                placeholder="https://example.com"
            >{{ new_urls }}</textarea>
        </div>
        
        <button type="submit" class="btn btn-success">保存</button>
    </form>
</div>

<script>
    // フォーム送信前の確認
    document.querySelector('form').addEventListener('submit', function(e) {
        const deleteCheckboxes = document.querySelectorAll('input[name="delete_urls"]:checked');
        if (deleteCheckboxes.length > 0) {
            if (!confirm('選択したURLを削除しますか？')) {
                e.preventDefault();
            }
        }
    });
</script>
{% endblock %}
```

**フォーム処理のポイント:**

- **複数値の送信**: `name="delete_urls"`で複数のチェックボックスの値を配列として送信
- **動的なID生成**: `id="url_{{ loop.index }}"`でユニークなIDを生成
- **JavaScriptとの連携**: `onchange`イベントでCSSクラスを動的に変更
- **`extra_scripts`ブロック**: ページ固有のJavaScriptを追加可能

**コントローラーでのフォームデータ受信:**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct EditForm {
    #[serde(default)]
    pub delete_urls: Vec<String>,
    #[serde(default)]
    pub new_urls: String,
}

pub async fn edit_post(
    State(ctx): State<AppContext>,
    Form(form): Form<EditForm>,
) -> Result<Response> {
    // delete_urlsは選択されたURLの配列
    // new_urlsは改行区切りのテキスト
    
    let new_url_list: Vec<String> = form.new_urls
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    
    // 処理...
}
```


#### 2.4 ログインページ（assets/views/auth/login.html）

```html
{% extends "layout/base.html" %}

{% block title %}ログイン - Telegraf設定管理{% endblock %}

{% block navigation %}{% endblock %}

{% block content %}
<div class="login-container">
    <div class="login-card">
        <div class="login-icon">🔐</div>
        <h2>管理者ログイン</h2>
        
        {% if error %}
        <div class="alert alert-error">
            {{ error }}
        </div>
        {% endif %}
        
        <form method="POST" action="/auth/login">
            <div class="form-group">
                <label for="username">ユーザー名</label>
                <input 
                    type="text" 
                    id="username" 
                    name="username" 
                    class="form-control" 
                    required 
                    autofocus
                >
            </div>
            
            <div class="form-group">
                <label for="password">パスワード</label>
                <input 
                    type="password" 
                    id="password" 
                    name="password" 
                    class="form-control" 
                    required
                >
            </div>
            
            <button type="submit" class="btn btn-primary" style="width: 100%;">
                ログイン
            </button>
        </form>
    </div>
</div>
{% endblock %}
```

**ポイント:**

- **`{% block navigation %}{% endblock %}`**: ナビゲーションを空にして非表示
- **HTMLフォーム属性**:
  - `required`: HTML5のバリデーション
  - `autofocus`: ページ読み込み時に自動フォーカス
  - `type="password"`: パスワード入力フィールド

#### 2.5 公開ページ（assets/views/config/show.html）

```html
{% extends "layout/base.html" %}

{% block title %}Telegraf設定情報{% endblock %}

{% block header_title %}Telegraf設定情報{% endblock %}
{% block header_subtitle %}現在監視中のURL一覧{% endblock %}

{% block content %}
<div class="card">
    <h2>監視中のURL一覧</h2>
    {% if urls %}
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
    {% else %}
    <p class="text-center">監視中のURLはありません</p>
    {% endif %}
</div>

<div class="text-center mt-3">
    <a href="/admin" class="btn btn-primary">管理画面へ</a>
</div>
{% endblock %}
```

### 3. エラーページの実装

#### 3.1 401 認証エラー（assets/views/errors/401.html）

```html
{% extends "layout/base.html" %}

{% block title %}401 - 認証が必要です{% endblock %}

{% block navigation %}{% endblock %}

{% block content %}
<div class="card">
    <div class="error-container">
        <div class="error-icon">🔒</div>
        <h1 class="error-code">401</h1>
        <p class="error-message">認証が必要です</p>
        <div class="error-description">
            <p>このページにアクセスするには、ログインが必要です。</p>
        </div>
        <div class="error-actions">
            <a href="/auth/login" class="btn btn-primary">ログインページへ</a>
            <a href="/conf" class="btn btn-secondary">公開ページへ</a>
        </div>
    </div>
</div>
{% endblock %}
```

**コントローラーでの使用:**

```rust
use loco_rs::prelude::*;

pub async fn require_auth(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    // 認証チェック
    if !is_authenticated() {
        return format::render()
            .status(StatusCode::UNAUTHORIZED)
            .view(&v, "errors/401.html", json!({}));
    }
    
    // 認証済みの処理
    Ok(())
}
```

#### 3.2 403 アクセス拒否（assets/views/errors/403.html）

```html
{% extends "layout/base.html" %}

{% block title %}403 - アクセス拒否{% endblock %}

{% block content %}
<div class="card">
    <div class="error-container">
        <div class="error-icon">🚫</div>
        <h1 class="error-code">403</h1>
        <p class="error-message">アクセスが拒否されました</p>
        <div class="error-description">
            <p>このリソースへのアクセス権限がありません。</p>
        </div>
    </div>
</div>
{% endblock %}
```

**使用例:**

```rust
// 権限チェック
if !user.has_permission("admin") {
    return format::render()
        .status(StatusCode::FORBIDDEN)
        .view(&v, "errors/403.html", json!({}));
}
```


#### 3.3 500 サーバーエラー（assets/views/errors/500.html）

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

**エラーハンドリングの実装:**

```rust
use loco_rs::prelude::*;

// カスタムエラーハンドラー
pub async fn handle_error(
    State(ctx): State<AppContext>,
    ViewEngine(v): ViewEngine<TeraView>,
    err: Error,
) -> Response {
    tracing::error!("Application error: {:?}", err);
    
    // 開発環境ではエラー詳細を表示
    let error_message = if ctx.environment == Environment::Development {
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
            // テンプレートレンダリングに失敗した場合のフォールバック
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal Server Error".into())
                .unwrap()
        })
}
```

#### 3.4 404 ページが見つかりません（assets/views/errors/404.html）

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
    </div>
</div>
{% endblock %}
```

**Locoでの404ハンドリング:**

```rust
// app.rsでカスタム404ハンドラーを設定
impl Hooks for App {
    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::config::routes())
            .add_route(controllers::auth::routes())
            .add_route(controllers::admin::routes())
            // 404ハンドラー（すべてのルートの最後に配置）
            .add_route(controllers::errors::routes())
    }
}

// controllers/errors.rs
pub fn routes() -> Routes {
    Routes::new()
        .add("/*path", get(not_found))
}

pub async fn not_found(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render()
        .status(StatusCode::NOT_FOUND)
        .view(&v, "errors/404.html", json!({}))
}
```

## Locoにおけるビューレンダリング

### ViewEngineの使用

Locoでは`ViewEngine`エクストラクターを使用してテンプレートエンジンにアクセスします。

```rust
use loco_rs::prelude::*;

pub async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    // テンプレートをレンダリング
    format::render().view(&v, "admin/index.html", json!({
        "url_count": 10,
        "current_page": "dashboard"
    }))
}
```

**ポイント:**

- **`ViewEngine(v)`**: パターンマッチングでTeraViewを取り出す
- **`format::render()`**: レスポンスビルダーを作成
- **`.view()`**: テンプレートとデータを指定
- **`json!({})`**: `serde_json`マクロでJSONオブジェクトを作成

### データの渡し方

#### 単純な値

```rust
format::render().view(&v, "template.html", json!({
    "name": "太郎",
    "age": 25,
    "is_admin": true
}))
```

#### 構造体を使用

```rust
use serde::Serialize;

#[derive(Serialize)]
struct PageData {
    title: String,
    items: Vec<String>,
    count: usize,
}

let data = PageData {
    title: "URL一覧".to_string(),
    items: vec!["https://example.com".to_string()],
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

テンプレート側では：

```html
{% if error %}
<div class="alert alert-error">{{ error }}</div>
{% endif %}
```


## CSSスタイリングのベストプラクティス

### 1. BEM命名規則の簡易版

```css
/* Block */
.site-header { }

/* Element */
.site-header h1 { }

/* Modifier */
.btn-primary { }
.btn-secondary { }
```

### 2. レスポンシブデザイン

```css
/* モバイルファースト */
.container {
    padding: 1rem;
}

/* タブレット以上 */
@media (min-width: 768px) {
    .container {
        padding: 2rem;
    }
}

/* デスクトップ */
@media (min-width: 1024px) {
    .container {
        max-width: 1200px;
        margin: 0 auto;
    }
}
```

### 3. CSS変数の活用（将来的な改善案）

```css
:root {
    --primary-color: #667eea;
    --secondary-color: #764ba2;
    --danger-color: #e74c3c;
    --success-color: #27ae60;
    
    --spacing-sm: 0.5rem;
    --spacing-md: 1rem;
    --spacing-lg: 2rem;
    
    --border-radius: 8px;
    --box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.btn-primary {
    background: var(--primary-color);
    padding: var(--spacing-md) var(--spacing-lg);
    border-radius: var(--border-radius);
}
```

### 4. トランジション効果

```css
.btn {
    transition: all 0.3s ease;
}

.btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
}

.card {
    transition: transform 0.3s ease;
}

.card:hover {
    transform: scale(1.02);
}
```

## Teraテンプレートの高度な機能

### 1. フィルター

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
{{ number | filesizeformat }}  <!-- ファイルサイズ形式 -->

<!-- 日付操作 -->
{{ created_at | date(format="%Y-%m-%d") }}
```

### 2. マクロ

再利用可能なテンプレート部品を定義できます。

```html
<!-- macros.html -->
{% macro alert(type, message) %}
<div class="alert alert-{{ type }}">
    {{ message }}
</div>
{% endmacro %}

{% macro button(text, url, style="primary") %}
<a href="{{ url }}" class="btn btn-{{ style }}">{{ text }}</a>
{% endmacro %}

<!-- 使用例 -->
{% import "macros.html" as macros %}

{{ macros::alert(type="success", message="保存しました") }}
{{ macros::button(text="編集", url="/admin/edit", style="primary") }}
```

### 3. インクルード

```html
<!-- 共通パーツを別ファイルに分離 -->
{% include "partials/header.html" %}
{% include "partials/navigation.html" %}

<main>
    {% block content %}{% endblock %}
</main>

{% include "partials/footer.html" %}
```

### 4. 条件分岐の応用

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

<!-- 型チェック -->
{% if value is string %}
    <p>{{ value }}</p>
{% elif value is number %}
    <p>{{ value | round }}</p>
{% endif %}
```

### 5. ループの高度な使用

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

<!-- ループ変数 -->
<!-- loop.index: 1から始まるインデックス -->
<!-- loop.index0: 0から始まるインデックス -->
<!-- loop.first: 最初の要素かどうか -->
<!-- loop.last: 最後の要素かどうか -->
<!-- loop.length: ループの総数 -->
```

## セキュリティ考慮事項

### 1. XSS対策

Teraはデフォルトで自動エスケープを行います。

```html
<!-- 自動エスケープ（安全） -->
{{ user_input }}

<!-- エスケープを無効化（危険！信頼できるHTMLのみ） -->
{{ trusted_html | safe }}
```

### 2. CSRF対策

フォームにCSRFトークンを含める（Locoが自動的に処理）。

```html
<form method="POST" action="/admin/edit">
    <!-- Locoが自動的にCSRFトークンを検証 -->
    <input type="text" name="url">
    <button type="submit">送信</button>
</form>
```

### 3. SQLインジェクション対策

テンプレートではなく、コントローラー側でSeaORMを使用して対策。

```rust
// 安全（パラメータ化クエリ）
let user = Users::find()
    .filter(users::Column::Username.eq(username))
    .one(&ctx.db)
    .await?;

// 危険（生のSQL）
// 使用しないこと
```


## パフォーマンス最適化

### 1. テンプレートのキャッシュ

Locoは本番環境で自動的にテンプレートをキャッシュします。

```yaml
# config/production.yaml
server:
  # テンプレートキャッシュが有効
  
# config/development.yaml
server:
  # 開発環境では変更が即座に反映される
```

### 2. CSSの最適化

本番環境では、CSSを外部ファイルに分離し、minifyすることを推奨。

```html
<!-- 開発環境: インラインCSS -->
<style>
    /* スタイル */
</style>

<!-- 本番環境: 外部CSS -->
<link rel="stylesheet" href="/static/css/main.min.css">
```

### 3. 画像とアセットの最適化

```rust
// static/ディレクトリの設定
impl Hooks for App {
    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(
                Routes::new()
                    .prefix("/static")
                    .add("/", get(serve_static))
            )
    }
}
```

## テスト

### 1. テンプレートの存在確認

```rust
#[tokio::test]
async fn test_template_exists() {
    let tera = Tera::new("assets/views/**/*.html").unwrap();
    
    assert!(tera.get_template_names().any(|name| name == "layout/base.html"));
    assert!(tera.get_template_names().any(|name| name == "admin/index.html"));
    assert!(tera.get_template_names().any(|name| name == "errors/404.html"));
}
```

### 2. レンダリングテスト

```rust
#[tokio::test]
async fn test_render_dashboard() {
    let tera = Tera::new("assets/views/**/*.html").unwrap();
    
    let context = json!({
        "url_count": 5,
        "current_page": "dashboard"
    });
    
    let rendered = tera.render("admin/index.html", &context).unwrap();
    
    assert!(rendered.contains("5"));
    assert!(rendered.contains("ダッシュボード"));
}
```

### 3. 統合テスト

```rust
#[tokio::test]
async fn test_dashboard_page() {
    let app = create_test_app().await;
    
    let response = app
        .get("/admin")
        .add_header("Cookie", "session=test_session")
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.contains("ダッシュボード"));
}
```

## トラブルシューティング

### 1. テンプレートが見つからない

```
Error: Template 'admin/index.html' not found
```

**解決方法:**
- ファイルパスが正しいか確認（`assets/views/`からの相対パス）
- ファイル名の大文字小文字を確認
- Teraのグロブパターンが正しいか確認

### 2. 変数が表示されない

```html
<!-- 空白が表示される -->
<p>{{ user_name }}</p>
```

**解決方法:**
- コントローラーで変数を渡しているか確認
- 変数名のスペルミスを確認
- `{% if user_name %}{{ user_name }}{% endif %}`でデバッグ

### 3. CSSが適用されない

**解決方法:**
- ブラウザのキャッシュをクリア
- 開発者ツールでCSSが読み込まれているか確認
- セレクターの詳細度を確認

### 4. レイアウト継承が機能しない

```
Error: Template 'layout/base.html' not found
```

**解決方法:**
- `{% extends %}`のパスが正しいか確認
- 親テンプレートが存在するか確認
- `{% block %}`の名前が一致しているか確認

## ディレクトリ構造

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
        ├── 401.html           # 認証エラー
        ├── 403.html           # アクセス拒否
        ├── 404.html           # ページが見つかりません
        └── 500.html           # サーバーエラー
```

## まとめ

このタスクで実装した内容：

1. **共通レイアウト**: すべてのページで統一されたデザイン
2. **テンプレート継承**: コードの重複を避け、保守性を向上
3. **レスポンシブデザイン**: モバイルからデスクトップまで対応
4. **エラーページ**: ユーザーフレンドリーなエラー表示
5. **フォーム処理**: 安全で使いやすいフォーム
6. **スタイリング**: モダンで洗練されたUI

**Locoフレームワークの利点:**

- **Teraテンプレートエンジン**: Jinja2ライクな構文で使いやすい
- **自動エスケープ**: XSS対策が標準で有効
- **ホットリロード**: 開発環境でテンプレート変更が即座に反映
- **型安全**: Rustの型システムとの統合

**次のステップ:**

- タスク10以降で、これらのテンプレートを使用するコントローラーを実装
- JavaScriptを追加してインタラクティブな機能を実装
- CSSを外部ファイルに分離して最適化
- アクセシビリティの改善（ARIA属性、キーボードナビゲーション）

## 参考リンク

- [Tera公式ドキュメント](https://keats.github.io/tera/)
- [Loco Views](https://loco.rs/docs/the-app/views/)
- [MDN Web Docs - HTML](https://developer.mozilla.org/ja/docs/Web/HTML)
- [MDN Web Docs - CSS](https://developer.mozilla.org/ja/docs/Web/CSS)
- [Web Content Accessibility Guidelines (WCAG)](https://www.w3.org/WAI/WCAG21/quickref/)
