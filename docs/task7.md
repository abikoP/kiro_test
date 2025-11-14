# タスク7: URL一覧表示機能の実装

## 概要

このタスクでは、Telegrafの設定ファイルに登録されているHTTP監視URL一覧を表示する機能を実装しました。認証済みユーザーが`/admin/list`エンドポイントにアクセスすると、ConfigServiceから取得したURL一覧がテーブル形式で表示されます。また、編集画面へのナビゲーションリンクも提供し、ユーザーが直感的にURLを管理できるようにしています。

## 実装したファイル

- `src/controllers/admin_list.rs` - URL一覧表示コントローラー（新規作成）
- `src/controllers/mod.rs` - モジュール定義（更新）
- `src/app.rs` - ルーティング設定（更新）

## 学習ポイント

### 1. 新しいコントローラーモジュールの作成

```rust
// src/controllers/admin_list.rs
use axum::{extract::State, response::{Html, IntoResponse}};
use loco_rs::prelude::*;
use crate::services::config_service::ConfigService;

pub async fn index(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // URL一覧を取得
    let urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;
    
    // HTMLを生成して返す
    Ok(Html(html))
}
```

**解説:**
- `ConfigService::get_urls()`は`Result<Vec<String>, ConfigError>`を返す
- `map_err()`でエラー型を変換
  - `ConfigError` → `loco_rs::Error`
  - エラーメッセージをユーザーフレンドリーに変換
- `?`演算子でエラーを伝播
  - エラー時は500エラーページが表示される
  - タスク6の`match`式とは異なるアプローチ

**`map_err()`の詳細:**

```rust
// 元の型: Result<Vec<String>, ConfigError>
let result = ConfigService::get_urls();

// エラー型を変換: Result<Vec<String>, loco_rs::Error>
let urls = result.map_err(|e| {
    loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
})?;
```

**エラーハンドリングの選択肢:**

1. **`?`演算子 + `map_err()`**: エラーを伝播（今回の実装）
   - エラー時は500エラーページ
   - 設定ファイルが存在しない場合は致命的エラー

2. **`match`式**: エラーを処理（タスク6の実装）
   - エラー時もページを表示
   - デフォルト値で処理を続行

3. **`unwrap_or_default()`**: エラーを無視
   ```rust
   let urls = ConfigService::get_urls().unwrap_or_default();
   ```

### 2. 動的なテーブル生成

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

**解説:**
- `urls.iter().enumerate()`でインデックス付きイテレーション
  - `idx`: 0から始まるインデックス
  - `url`: URL文字列への参照
- `idx + 1`で1から始まる番号を表示
- `html_escape::encode_text(url)`でXSS対策
  - `<script>`などの特殊文字をエスケープ
  - `&lt;script&gt;`のように変換される
- `push_str()`で文字列を連結
  - `String::new()`で空の文字列を作成
  - ループ内で行を追加していく

**`enumerate()`の使い方:**

```rust
let urls = vec!["https://example.com", "https://test.com"];

// enumerate()なし
for url in urls.iter() {
    println!("{}", url);
}
// 出力:
// https://example.com
// https://test.com

// enumerate()あり
for (idx, url) in urls.iter().enumerate() {
    println!("{}: {}", idx + 1, url);
}
// 出力:
// 1: https://example.com
// 2: https://test.com
```

**XSSエスケープの重要性:**

```rust
// 危険: エスケープなし
let url = "<script>alert('XSS')</script>";
let html = format!("<td>{}</td>", url);
// 結果: <td><script>alert('XSS')</script></td>
// → スクリプトが実行される！

// 安全: エスケープあり
let html = format!("<td>{}</td>", html_escape::encode_text(url));
// 結果: <td>&lt;script&gt;alert('XSS')&lt;/script&gt;</td>
// → スクリプトは実行されず、テキストとして表示される
```

### 3. 条件分岐によるUI表示の切り替え

```rust
let html = format!(
    r#"
    <div class="card">
        <h2>監視中のURL一覧</h2>
        {}
        <div class="actions">
            <a href="/admin/edit" class="btn">URLを編集する</a>
        </div>
    </div>
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
```

**解説:**
- `if urls.is_empty()`で空状態をチェック
- 三項演算子のような使い方
  - 条件が真: 空状態メッセージを表示
  - 条件が偽: テーブルを表示
- `format!`マクロのプレースホルダー`{}`に式を埋め込める
  - `if`式は値を返すので、そのまま埋め込める

**`if`式の値:**

```rust
// if式は値を返す
let message = if count == 0 {
    "データがありません"
} else {
    "データがあります"
};

// これは以下と同じ
let message = match count {
    0 => "データがありません",
    _ => "データがあります",
};
```

**空状態デザインのベストプラクティス:**
- **明確なメッセージ**: 「データがありません」ではなく「監視中のURLはありません」
- **次のアクション**: 「URLを追加する」ボタンを表示
- **視覚的フィードバック**: アイコンやイラストで状態を表現

### 4. テーブルスタイリング

```css
table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 1rem;
}

th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid #e0e0e0;
}

th {
    background: #f8f9fa;
    font-weight: 600;
    color: #333;
}

tr:hover {
    background: #f8f9fa;
}
```

**解説:**
- `border-collapse: collapse`: セルのボーダーを結合
  - デフォルトは`separate`（セル間に隙間）
  - `collapse`で隙間をなくし、すっきりした見た目に
- `border-bottom`のみ設定: 横線のみのシンプルなデザイン
  - 縦線は視覚的ノイズになるため省略
- `tr:hover`: ホバー時に行を強調
  - ユーザーがどの行を見ているか明確に

**`border-collapse`の違い:**

```css
/* separate（デフォルト） */
table {
    border-collapse: separate;
    border-spacing: 2px;
}
/* 結果: セル間に隙間がある */

/* collapse */
table {
    border-collapse: collapse;
}
/* 結果: セルが密着している */
```

**テーブルデザインのパターン:**

1. **ストライプテーブル**: 偶数行に背景色
   ```css
   tr:nth-child(even) {
       background: #f8f9fa;
   }
   ```

2. **ボーダーテーブル**: すべてのセルにボーダー
   ```css
   th, td {
       border: 1px solid #e0e0e0;
   }
   ```

3. **ホバーテーブル**: ホバー時に行を強調（今回の実装）
   ```css
   tr:hover {
       background: #f8f9fa;
   }
   ```

### 5. モジュールシステムとコントローラーの登録

```rust
// src/controllers/mod.rs
pub mod auth;
pub mod admin;
pub mod admin_list;  // ← 追加
pub mod config;
```

**解説:**
- `pub mod admin_list;`で新しいモジュールを公開
- Rustのモジュールシステム:
  - `mod.rs`はディレクトリのエントリーポイント
  - `pub mod`で外部から使用可能に
  - `admin_list.rs`ファイルが自動的に読み込まれる

**モジュール構造:**

```
src/
├── controllers/
│   ├── mod.rs          # pub mod admin_list;
│   ├── admin.rs        # pub async fn index()
│   ├── admin_list.rs   # pub async fn index()
│   ├── auth.rs
│   └── config.rs
└── main.rs             # use crate::controllers::admin_list;
```

**モジュールの可視性:**

```rust
// プライベートモジュール（デフォルト）
mod internal {
    pub fn helper() {}
}
// エラー: 外部から使用不可

// パブリックモジュール
pub mod public {
    pub fn api() {}
}
// OK: 外部から使用可能
```

**関数の可視性:**

```rust
pub mod admin_list {
    // パブリック関数: 外部から呼び出し可能
    pub async fn index() -> Result<impl IntoResponse> {
        // ...
    }
    
    // プライベート関数: モジュール内でのみ使用
    async fn build_html(urls: Vec<String>) -> String {
        // ...
    }
}
```

### 6. ルーティングへの追加

```rust
// src/app.rs
use crate::controllers::{admin, admin_list, auth, config};

fn routes(ctx: &AppContext) -> AppRoutes {
    // ...
    
    let protected_routes = Routes::new()
        .add("/admin", get(admin::index))
        .add("/admin/list", get(admin_list::index))  // ← 追加
        .add("/auth/logout", post(auth::logout))
        .layer(axum::middleware::from_fn_with_state(
            ctx.clone(),
            require_auth,
        ));
    
    // ...
}
```

**解説:**
- `use`文で`admin_list`モジュールをインポート
- `.add("/admin/list", get(admin_list::index))`でルートを追加
  - パス: `/admin/list`
  - HTTPメソッド: `GET`
  - ハンドラー: `admin_list::index`
- `protected_routes`に追加することで認証が必須に
  - `require_auth`ミドルウェアが自動的に適用される

**ルーティングの順序:**

```rust
// 順序が重要！
let routes = Routes::new()
    .add("/admin/list", get(admin_list::index))  // より具体的
    .add("/admin/:id", get(admin::show))         // パラメータ付き
    .add("/admin", get(admin::index));           // より一般的

// 間違った順序
let routes = Routes::new()
    .add("/admin", get(admin::index))            // これが先にマッチ
    .add("/admin/list", get(admin_list::index)); // 到達不可能！
```

**HTTPメソッドの使い分け:**

```rust
use axum::routing::{get, post, put, delete};

let routes = Routes::new()
    .add("/admin/list", get(admin_list::index))      // 一覧表示
    .add("/admin/edit", get(admin_edit::form))       // 編集フォーム表示
    .add("/admin/edit", post(admin_edit::update))    // 編集内容を保存
    .add("/admin/delete/:id", delete(admin::delete)) // 削除
    .add("/admin/create", post(admin::create));      // 新規作成
```

**RESTful APIの設計:**

| HTTPメソッド | パス | 用途 |
|------------|------|------|
| GET | `/admin/list` | 一覧表示 |
| GET | `/admin/:id` | 詳細表示 |
| GET | `/admin/new` | 新規作成フォーム |
| POST | `/admin` | 新規作成 |
| GET | `/admin/:id/edit` | 編集フォーム |
| PUT/PATCH | `/admin/:id` | 更新 |
| DELETE | `/admin/:id` | 削除 |

### 7. 認証ミドルウェアの適用

```rust
let protected_routes = Routes::new()
    .add("/admin", get(admin::index))
    .add("/admin/list", get(admin_list::index))
    .add("/auth/logout", post(auth::logout))
    .layer(axum::middleware::from_fn_with_state(
        ctx.clone(),
        require_auth,
    ));
```

**解説:**
- `.layer()`でミドルウェアを適用
  - すべての`protected_routes`に適用される
  - ルートごとに個別に設定する必要がない
- `from_fn_with_state()`で状態を渡す
  - `ctx.clone()`: AppContextをミドルウェアに渡す
  - `require_auth`: ミドルウェア関数

**ミドルウェアの実行順序:**

```
1. リクエスト受信: GET /admin/list
2. ミドルウェア実行: require_auth()
   ├─ セッションからユーザーIDを取得
   ├─ ユーザーIDが存在するかチェック
   └─ 認証成功 or 失敗
3a. 認証成功: admin_list::index()を実行
3b. 認証失敗: /auth/loginにリダイレクト
4. レスポンス返却
```

**複数のミドルウェアを適用:**

```rust
let protected_routes = Routes::new()
    .add("/admin/list", get(admin_list::index))
    .layer(axum::middleware::from_fn_with_state(ctx.clone(), require_auth))
    .layer(axum::middleware::from_fn(log_request))
    .layer(axum::middleware::from_fn(rate_limit));

// 実行順序（外側から内側）:
// 1. rate_limit
// 2. log_request
// 3. require_auth
// 4. admin_list::index
```

**ルートグループごとのミドルウェア:**

```rust
// 公開ルート: 認証不要
let public_routes = Routes::new()
    .add("/conf", get(config::show))
    .add("/auth/login", get(auth::login_form));

// 管理者ルート: 認証 + 管理者権限チェック
let admin_routes = Routes::new()
    .add("/admin/users", get(admin::users))
    .layer(axum::middleware::from_fn_with_state(ctx.clone(), require_admin));

// 一般ユーザールート: 認証のみ
let user_routes = Routes::new()
    .add("/dashboard", get(user::dashboard))
    .layer(axum::middleware::from_fn_with_state(ctx.clone(), require_auth));
```

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

3. **URL一覧ページにアクセス**
   ```
   http://localhost:3000/admin/list
   ```

### 期待される表示

**URLが登録されている場合:**
- ヘッダー: 「Telegraf設定管理システム」
- ナビゲーションバー: ダッシュボード、URL一覧、URL編集、ログアウト
- カードタイトル: 「監視中のURL一覧」
- テーブル:
  - ヘッダー: No、URL
  - データ行: 1、https://example.com など
- アクションボタン: 「URLを編集する」

**URLが登録されていない場合:**
- 空状態メッセージ: 「現在、監視中のURLはありません。」
- アクションボタン: 「URLを編集する」

### curlでの確認

```bash
# 認証なしでアクセス（リダイレクトされる）
curl -i http://localhost:3000/admin/list

# 期待される結果: 302 Found
# Location: /auth/login

# ログインしてセッションCookieを取得
curl -i -c cookies.txt -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=admin&password=password"

# セッション付きでアクセス
curl -i -b cookies.txt http://localhost:3000/admin/list

# 期待される結果: 200 OK
# Content-Type: text/html; charset=utf-8
# <table>...</table>
```

### ブラウザの開発者ツールでの確認

1. **ネットワークタブ**
   - リクエストURL: `http://localhost:3000/admin/list`
   - リクエストメソッド: `GET`
   - ステータスコード: `200 OK`
   - レスポンスヘッダー: `Content-Type: text/html; charset=utf-8`

2. **Cookieの確認**
   - 名前: `session` または `tower.sid`
   - 値: セッションID（ランダムな文字列）
   - HttpOnly: `true`
   - Secure: `false`（開発環境）

3. **コンソールタブ**
   - エラーがないことを確認
   - JavaScriptエラーがないことを確認

## セキュリティ上の考慮事項

### 1. XSS（クロスサイトスクリプティング）対策

```rust
use html_escape::encode_text;

let html = format!(
    r#"<td>{}</td>"#,
    encode_text(url)  // HTMLエスケープ
);
```

**攻撃シナリオ:**

```rust
// 悪意のあるURL
let url = r#"<script>alert('XSS')</script>"#;

// エスケープなし（危険）
let html = format!("<td>{}</td>", url);
// 結果: <td><script>alert('XSS')</script></td>
// → スクリプトが実行される！

// エスケープあり（安全）
let html = format!("<td>{}</td>", encode_text(url));
// 結果: <td>&lt;script&gt;alert('XSS')&lt;/script&gt;</td>
// → スクリプトは実行されず、テキストとして表示
```

**エスケープが必要な文字:**

| 文字 | エスケープ後 | 説明 |
|-----|------------|------|
| `<` | `&lt;` | タグの開始 |
| `>` | `&gt;` | タグの終了 |
| `&` | `&amp;` | エンティティの開始 |
| `"` | `&quot;` | 属性値の区切り |
| `'` | `&#x27;` | 属性値の区切り |

### 2. 認証の必須化

```rust
let protected_routes = Routes::new()
    .add("/admin/list", get(admin_list::index))
    .layer(axum::middleware::from_fn_with_state(
        ctx.clone(),
        require_auth,
    ));
```

**保護の仕組み:**
- すべての`/admin/*`エンドポイントに認証が必要
- セッションにユーザーIDが存在しない場合、ログインページにリダイレクト
- ミドルウェアがハンドラーの前に実行される

**認証バイパスの防止:**

```rust
// 間違った実装（危険）
let routes = Routes::new()
    .add("/admin", get(admin::index))
    .layer(require_auth)
    .add("/admin/list", get(admin_list::index));  // 認証なし！

// 正しい実装（安全）
let routes = Routes::new()
    .add("/admin", get(admin::index))
    .add("/admin/list", get(admin_list::index))
    .layer(require_auth);  // すべてのルートに適用
```

### 3. エラーメッセージの情報漏洩防止

```rust
let urls = ConfigService::get_urls().map_err(|e| {
    // 詳細なエラーメッセージ（開発環境）
    loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    
    // 本番環境では一般的なメッセージに
    // loco_rs::Error::string("データの取得に失敗しました")
})?;
```

**情報漏洩の例:**

```rust
// 危険: ファイルパスが漏洩
Err(format!("設定ファイルが見つかりません: /etc/telegraf/telegraf.conf"))

// 安全: 一般的なメッセージ
Err("設定ファイルが見つかりません".to_string())
```

### 4. セッション管理

```rust
use tower_sessions::{Session, MemoryStore};

let session_layer = SessionLayer::new(MemoryStore::default())
    .with_secure(true)       // HTTPS必須
    .with_http_only(true)    // JavaScriptからアクセス不可
    .with_same_site(SameSite::Strict);  // CSRF対策
```

**セキュリティ設定の説明:**

- **Secure**: HTTPS接続でのみCookieを送信
  - HTTP接続では送信されない
  - 中間者攻撃（MITM）を防ぐ

- **HttpOnly**: JavaScriptからCookieにアクセス不可
  - `document.cookie`で読み取れない
  - XSS攻撃によるセッション盗難を防ぐ

- **SameSite**: クロスサイトリクエストでCookieを送信しない
  - `Strict`: すべてのクロスサイトリクエストで送信しない
  - `Lax`: トップレベルナビゲーションのみ送信
  - `None`: すべてのリクエストで送信（非推奨）

## パフォーマンス最適化

### 1. 文字列連結の最適化

```rust
// 非効率: 毎回新しいStringを作成
let mut html = String::new();
for url in urls.iter() {
    html = html + &format!("<tr><td>{}</td></tr>", url);  // 遅い
}

// 効率的: push_str()で既存のStringに追加
let mut html = String::new();
for url in urls.iter() {
    html.push_str(&format!("<tr><td>{}</td></tr>", url));  // 速い
}

// さらに効率的: 容量を事前に確保
let mut html = String::with_capacity(urls.len() * 100);  // 推定サイズ
for url in urls.iter() {
    html.push_str(&format!("<tr><td>{}</td></tr>", url));
}
```

**`String::with_capacity()`の効果:**

```rust
// 容量なし: 再割り当てが発生
let mut s = String::new();  // 容量: 0
s.push_str("hello");        // 容量: 8（再割り当て）
s.push_str(" world");       // 容量: 16（再割り当て）

// 容量あり: 再割り当てが発生しない
let mut s = String::with_capacity(20);  // 容量: 20
s.push_str("hello");                    // 容量: 20（そのまま）
s.push_str(" world");                   // 容量: 20（そのまま）
```

### 2. イテレーターの活用

```rust
// 現在の実装
let mut url_rows = String::new();
for (idx, url) in urls.iter().enumerate() {
    url_rows.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", idx + 1, url));
}

// イテレーターを使った実装
let url_rows = urls
    .iter()
    .enumerate()
    .map(|(idx, url)| {
        format!("<tr><td>{}</td><td>{}</td></tr>", idx + 1, encode_text(url))
    })
    .collect::<Vec<_>>()
    .join("");

// さらに効率的: fold()を使用
let url_rows = urls
    .iter()
    .enumerate()
    .fold(String::with_capacity(urls.len() * 100), |mut acc, (idx, url)| {
        acc.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", idx + 1, encode_text(url)));
        acc
    });
```

### 3. HTMLの最小化

```rust
// 開発環境: 読みやすいHTML
let html = format!(
    r#"
    <table>
        <thead>
            <tr>
                <th>No</th>
                <th>URL</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
    "#,
    url_rows
);

// 本番環境: 最小化されたHTML
let html = format!(
    r#"<table><thead><tr><th>No</th><th>URL</th></tr></thead><tbody>{}</tbody></table>"#,
    url_rows
);
```

**最小化のメリット:**
- ファイルサイズの削減（約30-40%）
- 転送時間の短縮
- 帯域幅の節約

**最小化ツール:**

```rust
use minify_html::{Cfg, minify};

let html = format!(r#"<!DOCTYPE html>..."#, url_rows);
let minified = minify(html.as_bytes(), &Cfg::default());
Ok(Html(String::from_utf8(minified)?))
```

### 4. キャッシング戦略

```rust
use axum::response::Response;
use http::header::{CACHE_CONTROL, HeaderValue};

pub async fn index(State(_ctx): State<AppContext>) -> Result<Response> {
    let urls = ConfigService::get_urls()?;
    let html = build_html(urls);
    
    let mut response = Html(html).into_response();
    
    // キャッシュヘッダーを設定
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=60"),
    );
    
    Ok(response)
}
```

**キャッシュ戦略:**

- **`private`**: ブラウザのみキャッシュ（プロキシはキャッシュしない）
- **`public`**: すべてのキャッシュが可能
- **`max-age=60`**: 60秒間キャッシュ
- **`no-cache`**: 毎回サーバーに確認
- **`no-store`**: キャッシュしない

## テストの実装

### 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_url_escaping() {
        let url = "<script>alert('XSS')</script>";
        let escaped = html_escape::encode_text(url);
        assert_eq!(escaped, "&lt;script&gt;alert('XSS')&lt;/script&gt;");
    }
    
    #[test]
    fn test_empty_urls() {
        let urls: Vec<String> = vec![];
        assert!(urls.is_empty());
    }
    
    #[test]
    fn test_url_enumeration() {
        let urls = vec!["https://example.com".to_string()];
        let result: Vec<(usize, &String)> = urls.iter().enumerate().collect();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[0].1, "https://example.com");
    }
}
```

### 統合テスト

```rust
#[cfg(test)]
mod integration_tests {
    use axum::http::StatusCode;
    use tower::ServiceExt;
    
    #[tokio::test]
    async fn test_admin_list_requires_auth() {
        let app = create_test_app().await;
        
        // 認証なしでアクセス
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/list")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // リダイレクトされることを確認
        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().get("Location").unwrap(),
            "/auth/login"
        );
    }
    
    #[tokio::test]
    async fn test_admin_list_with_auth() {
        let app = create_test_app().await;
        let session_cookie = login_as_admin(&app).await;
        
        // 認証付きでアクセス
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/list")
                    .header("Cookie", session_cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // 成功することを確認
        assert_eq!(response.status(), StatusCode::OK);
        
        // HTMLが返されることを確認
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("監視中のURL一覧"));
    }
}
```

## UIの改善案

### 1. ページネーション

```rust
pub async fn index(
    State(_ctx): State<AppContext>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse> {
    let page = params.page.unwrap_or(1);
    let per_page = 20;
    
    let urls = ConfigService::get_urls()?;
    let total = urls.len();
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(total);
    let page_urls = &urls[start..end];
    
    // ページネーションHTMLを生成
    let pagination = build_pagination(page, total, per_page);
    
    // ...
}

#[derive(Deserialize)]
struct ListParams {
    page: Option<usize>,
}
```

### 2. 検索機能

```rust
pub async fn index(
    State(_ctx): State<AppContext>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse> {
    let urls = ConfigService::get_urls()?;
    
    // 検索フィルター
    let filtered_urls = if let Some(query) = params.search {
        urls.into_iter()
            .filter(|url| url.contains(&query))
            .collect()
    } else {
        urls
    };
    
    // ...
}

#[derive(Deserialize)]
struct ListParams {
    search: Option<String>,
}
```

### 3. ソート機能

```rust
pub async fn index(
    State(_ctx): State<AppContext>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse> {
    let mut urls = ConfigService::get_urls()?;
    
    // ソート
    match params.sort.as_deref() {
        Some("asc") => urls.sort(),
        Some("desc") => urls.sort_by(|a, b| b.cmp(a)),
        _ => {}
    }
    
    // ...
}

#[derive(Deserialize)]
struct ListParams {
    sort: Option<String>,
}
```

### 4. アクションボタン

```html
<table>
    <thead>
        <tr>
            <th>No</th>
            <th>URL</th>
            <th>アクション</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>1</td>
            <td>https://example.com</td>
            <td>
                <a href="/admin/edit?url=https://example.com" class="btn-small">編集</a>
                <form method="POST" action="/admin/delete" style="display:inline">
                    <input type="hidden" name="url" value="https://example.com">
                    <button type="submit" class="btn-small btn-danger">削除</button>
                </form>
            </td>
        </tr>
    </tbody>
</table>
```

## まとめ

このタスクで学んだRustとLocoの重要な概念：

1. **エラーハンドリング**: `map_err()`によるエラー型の変換
2. **動的HTML生成**: `enumerate()`とループによるテーブル生成
3. **XSS対策**: `html_escape::encode_text()`による安全な文字列埋め込み
4. **条件分岐**: `if`式による空状態の処理
5. **モジュールシステム**: 新しいコントローラーの追加と登録
6. **ルーティング**: 認証付きエンドポイントの追加
7. **ミドルウェア**: 認証チェックの自動適用

これらの概念は、データ一覧表示機能を実装する際の基本パターンです。特に、XSS対策とエラーハンドリングは、セキュアなWebアプリケーションを構築する上で不可欠な要素です。

## タスク6との比較

| 項目 | タスク6（ダッシュボード） | タスク7（URL一覧） |
|-----|----------------------|------------------|
| エラー処理 | `match`式（デフォルト値） | `map_err()` + `?`（エラー伝播） |
| データ表示 | 統計情報（数値） | テーブル（文字列） |
| 動的生成 | 単純な値埋め込み | ループによる行生成 |
| XSS対策 | 不要（数値のみ） | 必須（文字列） |
| 空状態 | URL数0を表示 | 空状態メッセージ |

**使い分けの指針:**

- **ダッシュボード**: 概要情報、統計、グラフ
  - エラーは回復可能（デフォルト値を使用）
  - 数値データが中心

- **一覧ページ**: 詳細データ、テーブル、検索
  - エラーは致命的（エラーページを表示）
  - 文字列データが中心（XSS対策必須）

## 次のステップ

今後の改善案：

1. **ページネーション**: 大量のURLを効率的に表示
2. **検索機能**: URLをキーワードで絞り込み
3. **ソート機能**: URLをアルファベット順に並び替え
4. **個別削除**: 各URLに削除ボタンを追加
5. **一括操作**: 複数のURLを一度に削除
6. **エクスポート**: URL一覧をCSV/JSONでダウンロード
7. **インポート**: CSVファイルからURLを一括登録
8. **バリデーション**: URL形式のチェック
9. **重複チェック**: 同じURLの登録を防ぐ
10. **履歴管理**: URL変更履歴を記録

## 参考リンク

- [Axum Routing](https://docs.rs/axum/latest/axum/routing/index.html)
- [Rust Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [HTML Escaping](https://docs.rs/html-escape/latest/html_escape/)
- [OWASP XSS Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
- [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Axum Middleware](https://docs.rs/axum/latest/axum/middleware/index.html)
