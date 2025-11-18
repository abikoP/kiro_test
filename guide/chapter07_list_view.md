# 第7章: URL一覧表示機能の実装

## この章で学ぶこと

- データ一覧の表示
- テーブルの動的生成
- HTMLエスケープ（XSS対策）
- 空状態の処理
- エラーハンドリングの実践

---

## 7.1 URL一覧ページとは

### 機能の概要

**URL一覧ページ**は、登録されているURLをテーブル形式で表示します。

**このページの役割:**
- 監視中のURLを一覧表示
- 編集画面へのナビゲーション
- 空状態の表示

**第6章との違い:**
- 第6章: 統計情報（数値）を表示
- 第7章: 詳細データ（文字列）を表示

---

## 7.2 AdminListControllerの実装

### src/controllers/admin_list.rsの作成

```rust
use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;

use crate::services::config_service::ConfigService;

/// URL一覧表示ページ
pub async fn index(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // ConfigServiceからURL一覧を取得
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
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>URL一覧 - Telegraf設定管理</title>
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
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
        }}
        th, td {{
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid #e0e0e0;
        }}
        th {{
            background: #f8f9fa;
            font-weight: 600;
            color: #333;
        }}
        tr:hover {{
            background: #f8f9fa;
        }}
        .actions {{
            margin-top: 1.5rem;
        }}
        .btn {{
            display: inline-block;
            background: #667eea;
            color: white;
            padding: 0.75rem 1.5rem;
            border-radius: 5px;
            text-decoration: none;
            transition: background 0.3s;
        }}
        .btn:hover {{
            background: #5568d3;
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
        .empty-state {{
            text-align: center;
            padding: 3rem;
            color: #666;
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
            <h2>監視中のURL一覧</h2>
            {}
            <div class="actions">
                <a href="/admin/edit" class="btn">URLを編集する</a>
            </div>
        </div>
    </div>
</body>
</html>
        "#,
        if urls.is_empty() {
            r#"<div class="empty-state">
                <p>現在、監視中のURLはありません。</p>
            </div>"#.to_string()
        } else {
            format!(
                r#"<table>
                <thead>
                    <tr>
                        <th>No</th>
                        <th>URL</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>"#,
                url_rows
            )
        }
    );

    Ok(Html(html))
}
```

### コードの詳細解説

#### エラーハンドリング
```rust
let urls = ConfigService::get_urls().map_err(|e| {
    loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
})?;
```

**map_err()とは:**
- エラー型を変換する
- `ConfigError` → `loco_rs::Error`

**第6章との違い:**
```rust
// 第6章: matchでデフォルト値
let url_count = match ConfigService::get_urls() {
    Ok(urls) => urls.len(),
    Err(_) => 0,  // エラー時も表示
};

// 第7章: map_err()でエラー伝播
let urls = ConfigService::get_urls().map_err(|e| {
    loco_rs::Error::string(&format!("...: {}", e))
})?;  // エラー時は500エラーページ
```

**使い分け:**
- ダッシュボード: エラーでもページを表示（デフォルト値）
- 一覧ページ: エラーは致命的（エラーページ）

#### テーブル行の生成
```rust
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
```

**enumerate()とは:**
- インデックス付きイテレーター
- `(0, "url1")`, `(1, "url2")`, ...

**例:**
```rust
let urls = vec!["https://example.com", "https://test.com"];

for (idx, url) in urls.iter().enumerate() {
    println!("{}: {}", idx + 1, url);
}
// 出力:
// 1: https://example.com
// 2: https://test.com
```

#### HTMLエスケープ（XSS対策）
```rust
html_escape::encode_text(url)
```

**なぜ必要？**
```rust
// 危険: エスケープなし
let url = "<script>alert('XSS')</script>";
let html = format!("<td>{}</td>", url);
// 結果: <td><script>alert('XSS')</script></td>
// → スクリプトが実行される！

// 安全: エスケープあり
let html = format!("<td>{}</td>", html_escape::encode_text(url));
// 結果: <td>&lt;script&gt;alert('XSS')&lt;/script&gt;</td>
// → テキストとして表示される
```

#### 空状態の処理
```rust
if urls.is_empty() {
    r#"<div class="empty-state">
        <p>現在、監視中のURLはありません。</p>
    </div>"#.to_string()
} else {
    format!(r#"<table>...</table>"#, url_rows)
}
```

- URLがない場合: 空状態メッセージ
- URLがある場合: テーブル表示

---

## 7.3 モジュールとルーティングの設定

### src/controllers/mod.rsの更新

```rust
pub mod auth;
pub mod config;
pub mod admin;
pub mod admin_list;  // 追加
```

### src/app.rsのroutes()を更新

```rust
fn routes(_ctx: &AppContext) -> AppRoutes {
    use axum::routing::{get, post};
    use crate::controllers::{auth, config, admin, admin_list};
    use crate::middleware::auth::require_auth;

    AppRoutes::with_default_routes()
        // 公開ルート
        .add("/conf", get(config::show))
        .add("/auth/login", get(auth::login_form))
        .add("/auth/login", post(auth::login))
        
        // 保護ルート
        .add("/admin", get(admin::index))
        .add("/admin/list", get(admin_list::index))  // 追加
        .add("/auth/logout", post(auth::logout))
        .layer(axum::middleware::from_fn(require_auth))
}
```

---

## 7.4 動作確認

### サーバーの起動

```bash
cargo run -- start
```

### アクセス手順

1. ログイン: `http://localhost:3000/auth/login`
2. URL一覧: `http://localhost:3000/admin/list`

### 表示される内容

**URLがある場合:**
- テーブル
  - No列: 1, 2, 3, ...
  - URL列: https://example.com, ...
- 「URLを編集する」ボタン

**URLがない場合:**
- 「現在、監視中のURLはありません。」
- 「URLを編集する」ボタン

---

## 7.5 第6章との比較

| 項目 | 第6章（ダッシュボード） | 第7章（URL一覧） |
|-----|----------------------|------------------|
| **データ** | 統計情報（数値） | 詳細データ（文字列） |
| **エラー処理** | match（デフォルト値） | map_err() + ? |
| **表示形式** | 統計ボックス | テーブル |
| **XSS対策** | 不要（数値のみ） | 必須（文字列） |
| **空状態** | 0を表示 | メッセージ表示 |

---

## 7.6 まとめ

この章では、以下を学びました：

- ✅ データ一覧の表示
- ✅ enumerate()によるインデックス付きループ
- ✅ HTMLエスケープ（XSS対策）
- ✅ 空状態の処理
- ✅ map_err()によるエラー型変換

**重要なポイント:**
- 文字列データは必ずHTMLエスケープ
- enumerate()でインデックスを取得
- 空状態は明確なメッセージを表示
- エラーハンドリングは用途に応じて選択

**セキュリティ:**
```
ユーザ入力 → HTMLエスケープ → 安全な表示
```

---

## 次のステップ

次の章では、**URL編集機能**を実装します。フォーム処理とデータの更新を学びます。

[第8章: URL編集フォームの実装](./chapter08_edit_form.md)に進みましょう！
