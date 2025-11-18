# 第6章: 管理画面トップページの実装

## この章で学ぶこと

- 認証が必要なページの実装
- ダッシュボードUIの設計
- エラーハンドリングの実践
- ナビゲーションバーの実装
- CSS Gridレイアウト

---

## 6.1 管理画面とは

### ダッシュボードの役割

**ダッシュボード**は、システムの概要を一目で把握できるページです。

**このシステムのダッシュボード:**
- 監視中のURL数を表示
- 各機能へのナビゲーション
- システムの説明

**認証が必要:**
- ログインしたユーザーのみアクセス可能
- 認証ミドルウェアで保護

---

## 6.2 AdminControllerの実装

### src/controllers/admin.rsの作成

```rust
use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;

use crate::services::config_service::ConfigService;

/// 管理画面トップページ
pub async fn index(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // ダッシュボード情報を取得
    let url_count = match ConfigService::get_urls() {
        Ok(urls) => urls.len(),
        Err(_) => 0,
    };
    
    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>管理画面 - Telegraf設定管理</title>
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
        .nav {{
            background: white;
            padding: 1rem 2rem;
            box-shadow: 0 2px 4px rgba(0,0,0,0.05);
        }}
        .nav a {{
            color: #667eea;
            text-decoration: none;
            margin-right: 1.5rem;
            font-weight: 500;
        }}
        .nav a:hover {{
            text-decoration: underline;
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
            margin-bottom: 1rem;
        }}
        .card h2 {{
            margin-top: 0;
            color: #333;
        }}
        .dashboard-stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin: 1.5rem 0;
        }}
        .stat-box {{
            background: #f8f9fa;
            padding: 1.5rem;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }}
        .stat-box h3 {{
            margin: 0 0 0.5rem 0;
            color: #666;
            font-size: 0.9rem;
            font-weight: 500;
        }}
        .stat-box .value {{
            font-size: 2rem;
            font-weight: 700;
            color: #333;
        }}
        .logout-form {{
            display: inline;
        }}
        .logout-btn {{
            background: #e74c3c;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 5px;
            cursor: pointer;
            font-size: 0.9rem;
        }}
        .logout-btn:hover {{
            background: #c0392b;
        }}
        .quick-actions {{
            margin-top: 1.5rem;
        }}
        .quick-actions a {{
            display: inline-block;
            background: #667eea;
            color: white;
            padding: 0.75rem 1.5rem;
            border-radius: 5px;
            text-decoration: none;
            margin-right: 1rem;
            margin-bottom: 0.5rem;
            transition: background 0.3s;
        }}
        .quick-actions a:hover {{
            background: #5568d3;
        }}
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
                    <div class="value">{}</div>
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
        "#,
        url_count
    );
    Ok(Html(html))
}
```

### コードの詳細解説

#### エラーハンドリング
```rust
let url_count = match ConfigService::get_urls() {
    Ok(urls) => urls.len(),
    Err(_) => 0,
};
```

**なぜmatchを使う？**
- エラー時もページを表示したい
- デフォルト値（0）で処理を続行

**?演算子との違い:**
```rust
// ?演算子: エラー時は500エラーページ
let urls = ConfigService::get_urls()?;
let url_count = urls.len();

// match: エラー時もページを表示
let url_count = match ConfigService::get_urls() {
    Ok(urls) => urls.len(),
    Err(_) => 0,  // デフォルト値
};
```

**使い分け:**
- **?演算子**: エラーが致命的な場合
- **match**: エラーを回復可能な場合

#### ナビゲーションバー
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

**ポイント:**
- リンク: GETリクエスト（状態を変更しない）
- ログアウト: POSTリクエスト（セッションを削除）

#### ダッシュボード統計
```html
<div class="dashboard-stats">
    <div class="stat-box">
        <h3>監視中のURL数</h3>
        <div class="value">{}</div>
    </div>
</div>
```

- 統計情報を視覚的に表示
- 数値を大きく、ラベルを小さく

#### クイックアクション
```html
<div class="quick-actions">
    <a href="/admin/list">URL一覧を見る</a>
    <a href="/admin/edit">URLを編集する</a>
</div>
```

- よく使う機能へのショートカット
- ボタン風のリンク

---

## 6.3 CSS Gridレイアウト

### グリッドの設定

```css
.dashboard-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin: 1.5rem 0;
}
```

### コードの詳細解説

#### display: grid
```css
display: grid;
```

- CSS Gridレイアウトを使用
- 柔軟なレイアウトが可能

#### grid-template-columns
```css
grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
```

**分解して理解:**

##### repeat()
```css
repeat(auto-fit, ...)
```

- `auto-fit`: 利用可能なスペースに自動的にフィット
- カラム数が自動調整される

##### minmax()
```css
minmax(200px, 1fr)
```

- 最小: 200px
- 最大: 1fr（均等分割）

**動作例:**

```
画面幅1200px:
[200px] [200px] [200px] [200px] [200px] [200px]
↓ 自動調整
[200px] [200px] [200px] [200px] [200px] [200px]

画面幅800px:
[200px] [200px] [200px] [200px]
↓ 自動調整
[200px] [200px] [200px] [200px]

画面幅400px:
[200px] [200px]
↓ 自動調整
[400px]
```

#### gap
```css
gap: 1rem;
```

- グリッドアイテム間の間隔
- 1rem = 16px（通常）

---

## 6.4 モジュールの設定

### src/controllers/mod.rsの更新

```rust
pub mod auth;
pub mod config;
pub mod admin;
```

---

## 6.5 ルーティングの設定

### src/app.rsのroutes()を更新

```rust
fn routes(_ctx: &AppContext) -> AppRoutes {
    use axum::routing::{get, post};
    use crate::controllers::{auth, config, admin};
    use crate::middleware::auth::require_auth;

    AppRoutes::with_default_routes()
        // 公開ルート（認証不要）
        .add("/conf", get(config::show))
        .add("/auth/login", get(auth::login_form))
        .add("/auth/login", post(auth::login))
        
        // 保護ルート（認証必須）
        .add("/admin", get(admin::index))
        .add("/auth/logout", post(auth::logout))
        .layer(axum::middleware::from_fn(require_auth))
}
```

### コードの詳細解説

#### ルートの分離
```rust
// 公開ルート
.add("/conf", get(config::show))
.add("/auth/login", get(auth::login_form))

// 保護ルート
.add("/admin", get(admin::index))
.layer(axum::middleware::from_fn(require_auth))
```

**ミドルウェアの適用:**
- `.layer()`より前のルートに適用される
- `/admin`は認証が必要
- `/conf`と`/auth/login`は認証不要

---

## 6.6 動作確認

### サーバーの起動

```bash
cargo run -- start
```

### アクセス手順

1. **ログインページにアクセス**
   ```
   http://localhost:3000/auth/login
   ```

2. **ログイン**
   - ユーザ名: `admin`
   - パスワード: `admin123`

3. **管理画面トップページ**
   ```
   http://localhost:3000/admin
   ```

### 表示される内容

- ヘッダー: 「Telegraf設定管理システム」
- ナビゲーションバー
  - ダッシュボード
  - URL一覧
  - URL編集
  - ログアウトボタン
- ダッシュボードカード
  - 監視中のURL数
- ウェルカムカード
  - システムの説明
  - クイックアクションボタン

---

## 6.7 認証の確認

### 未認証でアクセス

```bash
curl -i http://localhost:3000/admin
```

**結果:**
```
HTTP/1.1 303 See Other
Location: /auth/login
```

- 認証ミドルウェアがリダイレクト
- ログインページへ誘導

### 認証済みでアクセス

ログイン後、ブラウザで`/admin`にアクセス
- ダッシュボードが表示される
- セッションCookieが送信される

---

## 6.8 まとめ

この章では、以下を学びました：

- ✅ 認証が必要なページの実装
- ✅ ダッシュボードUIの設計
- ✅ matchによるエラーハンドリング
- ✅ ナビゲーションバーの実装
- ✅ CSS Gridレイアウト
- ✅ ルーティングの分離

**重要なポイント:**
- matchでエラーを回復可能に処理
- CSS Gridでレスポンシブデザイン
- ミドルウェアで認証を制御
- ナビゲーションでユーザビリティ向上

**MVCパターン:**
```
リクエスト
    ↓
認証ミドルウェア (require_auth)
    ↓
コントローラー (admin::index)
    ↓
サービス (ConfigService::get_urls)
    ↓
レスポンス (HTML)
```

---

## 次のステップ

次の章では、**URL一覧表示機能**を実装します。データをテーブル形式で表示する方法を学びます。

[第7章: URL一覧表示機能の実装](./chapter07_list_view.md)に進みましょう！
