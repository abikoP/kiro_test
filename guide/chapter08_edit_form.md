# 第8章: URL編集フォームの実装

## この章で学ぶこと

- Axumのフォーム処理（`Form<T>`エクストラクタ）
- 配列データの安全な削除方法
- PRG（Post-Redirect-Get）パターン
- エラー時のユーザーフィードバック
- イテレータチェーンによる文字列処理

## 8.1 URL編集機能の概要

この章では、管理画面からURL一覧を編集（追加・削除）できる機能を実装します。ユーザーは以下の操作ができます：

- 既存のURLをチェックボックスで選択して削除
- 新しいURLを複数行のテキストエリアから追加
- バリデーションエラー時は編集画面を再表示

## 8.2 データ構造体の定義

まず、HTMLフォームから送信されるデータを受け取る構造体を定義します。

`src/controllers/admin_edit.rs`を新規作成：

```rust
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::services::{
    config_service::ConfigService, 
    url_validation_service::UrlValidationService
};

/// URL更新フォームのデータ構造体
#[derive(Debug, Deserialize)]
pub struct UrlUpdateForm {
    /// 削除対象のURLインデックス（チェックボックスで選択）
    #[serde(default)]
    pub delete_indices: Vec<usize>,
    
    /// 新規追加するURL（改行区切り）
    #[serde(default)]
    pub new_urls: String,
}
```

### コードの解説

**`#[derive(Deserialize)]`**
- Serdeを使ってHTMLフォームデータを構造体に自動変換します
- Axumが内部でこの機能を使ってリクエストボディをパースします

**`#[serde(default)]`**
- フォームフィールドが送信されない場合にデフォルト値を使用します
- チェックボックスが1つも選択されていない場合、`delete_indices`は空のVecになります
- テキストエリアが空の場合、`new_urls`は空文字列になります

**`delete_indices: Vec<usize>`**
- HTMLの複数チェックボックス（同じ`name`属性）は配列として受け取ります
- 例：`<input name="delete_indices" value="0">`と`<input name="delete_indices" value="1">`が両方チェックされると、`vec![0, 1]`になります

**`new_urls: String`**
- テキストエリアの内容を文字列として受け取ります
- 後で改行で分割して個別のURLに変換します

## 8.3 編集フォームの表示

次に、編集フォームを表示する関数を実装します。

```rust
/// URL編集フォーム表示
pub async fn edit(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // ConfigServiceから現在のURL一覧を取得
    let urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

    // URL一覧をHTMLで生成
    let mut url_list_html = String::new();
    for (idx, url) in urls.iter().enumerate() {
        url_list_html.push_str(&format!(
            r#"
            <div class="url-item">
                <input type="checkbox" name="delete_indices" value="{}" id="url_{}">
                <label for="url_{}">{}</label>
            </div>
            "#,
            idx, idx, idx,
            html_escape::encode_text(url)
        ));
    }

    let html = generate_edit_page_html(&url_list_html, "", false);
    Ok(Html(html))
}
```

### コードの解説

**`State<AppContext>`**
- Locoのアプリケーションコンテキストを取得します
- 今回は使用していませんが、将来的にデータベース接続などで使用できます
- `_ctx`という名前は「使用していない」ことを明示しています

**`Result<impl IntoResponse>`**
- Locoの標準的な戻り値型です
- `impl IntoResponse`は「レスポンスに変換できる何か」を意味します
- `Html`や`Redirect`など、異なる型を返すことができます

**`map_err()`**
- `Result`のエラー型を変換します
- `ConfigError`を`loco_rs::Error`に変換しています
- これにより、`?`オペレータで統一的にエラーを伝播できます

**`enumerate()`**
- イテレータのインデックスと値のペアを取得します
- `(0, "https://example.com")`のような形式になります
- チェックボックスの`value`属性にインデックスを使用します

**`html_escape::encode_text()`**
- XSS（クロスサイトスクリプティング）攻撃を防ぐためにエスケープします
- `<script>`などの特殊文字を`&lt;script&gt;`に変換します
- **セキュリティの基本原則：すべてのユーザー入力をエスケープする**

## 8.4 URL更新処理の実装

フォームから送信されたデータを処理する関数を実装します。

```rust
/// URL更新処理
pub async fn update(
    State(_ctx): State<AppContext>,
    Form(form): Form<UrlUpdateForm>,
) -> Result<impl IntoResponse> {
    // 現在のURL一覧を取得
    let mut current_urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

    // 削除対象のインデックスをソートして逆順にする（後ろから削除）
    let mut delete_indices = form.delete_indices.clone();
    delete_indices.sort_unstable();
    delete_indices.reverse();

    // 削除処理
    for idx in delete_indices {
        if idx < current_urls.len() {
            current_urls.remove(idx);
        }
    }

    // 新規URL追加処理
    let new_urls: Vec<String> = form
        .new_urls
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    // 新規URLの検証
    if let Err(e) = UrlValidationService::validate_urls(&new_urls) {
        // バリデーションエラー時は編集画面を再表示
        let url_list_html = generate_url_list_html(&current_urls);
        let error_message = format!("URLの検証に失敗しました: {}", e);
        let html = generate_edit_page_html(&url_list_html, &error_message, true);
        return Ok(Html(html).into_response());
    }

    // 新規URLを追加
    current_urls.extend(new_urls);

    // ConfigServiceでURL一覧を更新
    if let Err(e) = ConfigService::update_urls(current_urls.clone()) {
        // 更新エラー時は編集画面を再表示
        let url_list_html = generate_url_list_html(&current_urls);
        let error_message = format!("設定ファイルの更新に失敗しました: {}", e);
        let html = generate_edit_page_html(&url_list_html, &error_message, true);
        return Ok(Html(html).into_response());
    }

    // 成功時は一覧ページへリダイレクト
    Ok(Redirect::to("/admin/list").into_response())
}
```

### コードの解説

#### 1. Axumのフォーム抽出

```rust
Form(form): Form<UrlUpdateForm>
```

- `Form<T>`エクストラクタがHTTPリクエストボディを自動的にパースします
- Content-Type: `application/x-www-form-urlencoded`を処理します
- `UrlUpdateForm`構造体に自動的にマッピングされます

#### 2. 配列要素の安全な削除

```rust
delete_indices.sort_unstable();
delete_indices.reverse();
```

**なぜ後ろから削除するのか？**

配列から複数要素を削除する際は、後ろから削除するのが定石です。理由を見てみましょう：

```rust
// 悪い例：前から削除
let mut urls = vec!["A", "B", "C", "D"];
// インデックス1と3を削除したい
urls.remove(1); // "B"を削除 → ["A", "C", "D"]
urls.remove(3); // エラー！インデックス3は存在しない
```

前から削除すると、削除するたびにインデックスがずれてしまいます。

```rust
// 良い例：後ろから削除
let mut urls = vec!["A", "B", "C", "D"];
// インデックス1と3を削除したい → ソートして逆順 [3, 1]
urls.remove(3); // "D"を削除 → ["A", "B", "C"]
urls.remove(1); // "B"を削除 → ["A", "C"]
```

後ろから削除すれば、まだ削除していない要素のインデックスは変わりません。

**`sort_unstable()`**
- 安定ソートが不要な場合に高速です
- 同じ値の順序を保証しません（今回は数値なので問題なし）

#### 3. 文字列の行分割と処理

```rust
let new_urls: Vec<String> = form
    .new_urls
    .lines()
    .map(|line| line.trim().to_string())
    .filter(|line| !line.is_empty())
    .collect();
```

これはRustのイテレータチェーンの典型的な使い方です。ステップごとに見てみましょう：

**ステップ1: `lines()`**
```rust
"https://example.com\nhttps://example.org\n"
// ↓
["https://example.com", "https://example.org", ""]
```
- 改行で分割します（`\n`、`\r\n`両方に対応）

**ステップ2: `map(|line| line.trim().to_string())`**
```rust
["  https://example.com  ", "https://example.org", ""]
// ↓
["https://example.com", "https://example.org", ""]
```
- 各行の前後の空白を削除します

**ステップ3: `filter(|line| !line.is_empty())`**
```rust
["https://example.com", "https://example.org", ""]
// ↓
["https://example.com", "https://example.org"]
```
- 空行を除外します

**ステップ4: `collect()`**
- イテレータを`Vec<String>`に変換します

#### 4. エラーハンドリングとユーザーフィードバック

```rust
if let Err(e) = UrlValidationService::validate_urls(&new_urls) {
    let html = generate_edit_page_html(&url_list_html, &error_message, true);
    return Ok(Html(html).into_response());
}
```

- バリデーションエラー時は編集画面を再表示します
- エラーメッセージを表示してユーザーに修正を促します
- これはPRGパターンの例外です（エラー時は再表示）

#### 5. 成功時のリダイレクト（PRGパターン）

```rust
Ok(Redirect::to("/admin/list").into_response())
```

**PRG（Post-Redirect-Get）パターン**とは：

1. ユーザーがフォームを送信（POST）
2. サーバーがデータを処理
3. 成功したら別のページへリダイレクト（Redirect）
4. ブラウザが新しいページを取得（GET）

**なぜPRGパターンを使うのか？**

PRGパターンを使わない場合：
```
ユーザー → POST /admin/edit → 成功画面を表示
ユーザーがF5キーで再読み込み → POSTが再送信される！
→ 同じデータが二重に送信される
```

PRGパターンを使う場合：
```
ユーザー → POST /admin/edit → リダイレクト → GET /admin/list
ユーザーがF5キーで再読み込み → GETが再送信される
→ 一覧ページが再表示されるだけ（安全）
```

**`into_response()`**
- 異なる型を統一的なレスポンス型に変換します
- `Html`と`Redirect`は異なる型ですが、両方とも`IntoResponse`トレイトを実装しています
- これにより、同じ関数から異なるレスポンスを返せます

## 8.5 HTML生成関数

編集ページのHTMLを生成する関数を実装します。

```rust
/// URL一覧のHTML生成（エラー時の再表示用）
fn generate_url_list_html(urls: &[String]) -> String {
    let mut html = String::new();
    for (idx, url) in urls.iter().enumerate() {
        html.push_str(&format!(
            r#"
            <div class="url-item">
                <input type="checkbox" name="delete_indices" value="{}" id="url_{}">
                <label for="url_{}">{}</label>
            </div>
            "#,
            idx, idx, idx,
            html_escape::encode_text(url)
        ));
    }
    html
}
```

### フォームの設計

HTMLフォームは以下のように設計されています：

```html
<input type="checkbox" name="delete_indices" value="0" id="url_0">
<input type="checkbox" name="delete_indices" value="1" id="url_1">
<input type="checkbox" name="delete_indices" value="2" id="url_2">
```

**同じ`name`属性を持つチェックボックス**
- 複数のチェックボックスが同じ`name`属性を持つと、配列として送信されます
- チェックされたものだけが送信されます
- Rustの`Vec<usize>`に自動的にマッピングされます

例：
- チェックボックス0と2がチェックされている場合
- フォームデータ：`delete_indices=0&delete_indices=2`
- Rustの構造体：`delete_indices: vec![0, 2]`

### エラーメッセージの表示

```rust
fn generate_edit_page_html(url_list_html: &str, error_message: &str, is_error: bool) -> String {
    let error_html = if is_error && !error_message.is_empty() {
        format!(
            r#"
            <div class="error-message">
                <strong>エラー:</strong> {}
            </div>
            "#,
            html_escape::encode_text(error_message)
        )
    } else if !is_error && !error_message.is_empty() {
        format!(
            r#"
            <div class="success-message">
                {}
            </div>
            "#,
            html_escape::encode_text(error_message)
        )
    } else {
        String::new()
    };
    
    // ... HTMLの生成
}
```

**条件付きレンダリング**
- エラーがある場合は赤い背景のメッセージを表示
- 成功メッセージがある場合は緑の背景のメッセージを表示
- どちらもない場合は何も表示しない

## 8.6 ルーティングの設定

`src/app.rs`にルートを追加します。

```rust
use crate::controllers::{admin, admin_edit, admin_list, auth, config};

// ...

fn routes(ctx: &AppContext) -> AppRoutes {
    use loco_rs::controller::Routes;
    
    // 公開ルート
    let public_routes = Routes::new()
        .add("/conf", get(config::show))
        .add("/auth/login", get(auth::login_form))
        .add("/auth/login", post(auth::login));

    // 認証が必要なルート
    let protected_routes = Routes::new()
        .add("/admin", get(admin::index))
        .add("/admin/list", get(admin_list::index))
        .add("/admin/edit", get(admin_edit::edit))      // 追加
        .add("/admin/edit", post(admin_edit::update))   // 追加
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

### 同じパスに複数のHTTPメソッド

```rust
.add("/admin/edit", get(admin_edit::edit))
.add("/admin/edit", post(admin_edit::update))
```

- **GETリクエスト**: フォーム表示（`edit`関数）
- **POSTリクエスト**: フォーム送信処理（`update`関数）
- これはRESTfulな設計パターンです

**HTTPメソッドの使い分け**
- `GET`: データの取得（副作用なし、何度実行しても同じ結果）
- `POST`: データの変更（副作用あり、実行するたびに状態が変わる）

## 8.7 セキュリティのベストプラクティス

### 1. XSS対策

```rust
html_escape::encode_text(url)
```

**XSS（クロスサイトスクリプティング）攻撃とは？**

悪意のあるユーザーが以下のようなURLを入力したとします：
```
<script>alert('攻撃！')</script>
```

エスケープしない場合：
```html
<label>
```

エスケープする場合：
```html
<label>&lt;script&gt;alert('攻撃！')&lt;/script&gt;</label>
```

**重要な原則：すべてのユーザー入力をエスケープする**

### 2. 認証の強制

```rust
.layer(axum::middleware::from_fn_with_state(ctx.clone(), require_auth))
```

- ミドルウェアで認証を強制します
- 認証なしでは管理画面にアクセスできません
- すべての保護されたルートに自動的に適用されます

### 3. バリデーション

```rust
UrlValidationService::validate_urls(&new_urls)
```

- 入力データを必ず検証します
- 不正なデータの保存を防ぎます
- セキュリティとデータ整合性の両方を保護します

## 8.8 動作確認

実装が完了したら、動作を確認してみましょう。

```bash
# アプリケーションを起動
cargo loco start

# ブラウザで以下にアクセス
# http://localhost:5150/auth/login
```

1. ログインします
2. ダッシュボードから「URL編集」をクリック
3. 既存のURLをチェックして削除してみます
4. 新しいURLをテキストエリアに入力して追加してみます
5. 保存ボタンをクリック
6. URL一覧ページにリダイレクトされることを確認

**エラーケースも試してみましょう：**
- 無効なURL（`invalid-url`など）を入力
- エラーメッセージが表示されることを確認
- フォームの内容が保持されていることを確認

## まとめ

この章で学んだ主要な概念：

1. **Axumのフォーム処理**: `Form<T>`エクストラクタでフォームデータを構造体に自動変換
2. **配列要素の安全な削除**: 後ろから削除してインデックスのずれを防ぐ
3. **イテレータチェーン**: `lines()`, `map()`, `filter()`, `collect()`で効率的に処理
4. **PRGパターン**: POST後のリダイレクトで重複送信を防ぐ
5. **エラーハンドリング**: ユーザーフレンドリーなエラー表示
6. **セキュリティ**: HTMLエスケープ、認証ミドルウェア、バリデーション

次の章では、テンプレートエンジンを使ってコードをより保守しやすくします。

[第9章: テンプレートエンジンとレイアウト](./chapter09_templates.md)に進みましょう！
