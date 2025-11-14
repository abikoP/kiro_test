# Task 8: URL編集機能の実装

## 概要

このタスクでは、管理画面からURL一覧を編集（追加・削除）できる機能を実装しました。ユーザーは既存のURLを選択して削除したり、新しいURLを複数行のテキストエリアから追加できます。

## 実装内容

### 8.1 AdminEditControllerの実装

`src/controllers/admin_edit.rs`を新規作成し、URL編集機能のコントローラーを実装しました。

#### データ構造体の定義

```rust
#[derive(Debug, Deserialize)]
pub struct UrlUpdateForm {
    #[serde(default)]
    pub delete_indices: Vec<usize>,
    
    #[serde(default)]
    pub new_urls: String,
}
```

**学習ポイント:**
- `#[derive(Deserialize)]`: Serdeを使ってHTMLフォームデータを構造体に自動変換
- `#[serde(default)]`: フォームフィールドが送信されない場合（チェックボックスが未選択など）にデフォルト値を使用
- `delete_indices: Vec<usize>`: HTMLの複数チェックボックス（`name="delete_indices"`）は配列として受け取る
- `new_urls: String`: テキストエリアの内容を文字列として受け取り、後で改行で分割

#### edit()メソッド - 編集フォームの表示

```rust
pub async fn edit(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

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

**学習ポイント:**
- `State<AppContext>`: Locoのアプリケーションコンテキストを取得（今回は未使用だが、将来的にDB接続などで使用可能）
- `Result<impl IntoResponse>`: Locoの標準的な戻り値型。エラーハンドリングを統一
- `map_err()`: `Result`のエラー型を変換。`ConfigError`を`loco_rs::Error`に変換
- `html_escape::encode_text()`: XSS攻撃を防ぐためにユーザー入力をエスケープ（重要なセキュリティ対策）
- `enumerate()`: インデックスと値のペアを取得。チェックボックスのvalue属性に使用

### 8.2 URL更新処理の実装

#### update()メソッド - フォームデータの処理

```rust
pub async fn update(
    State(_ctx): State<AppContext>,
    Form(form): Form<UrlUpdateForm>,
) -> Result<impl IntoResponse> {
    // 現在のURL一覧を取得
    let mut current_urls = ConfigService::get_urls().map_err(|e| {
        loco_rs::Error::string(&format!("URL一覧の取得に失敗しました: {}", e))
    })?;

    // 削除対象のインデックスをソートして逆順にする
    let mut delete_indices = form.delete_indices.clone();
    delete_indices.sort_unstable();
    delete_indices.reverse();

    // 削除処理（後ろから削除）
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
        let url_list_html = generate_url_list_html(&current_urls);
        let error_message = format!("URLの検証に失敗しました: {}", e);
        let html = generate_edit_page_html(&url_list_html, &error_message, true);
        return Ok(Html(html).into_response());
    }

    // 新規URLを追加
    current_urls.extend(new_urls);

    // ConfigServiceでURL一覧を更新
    if let Err(e) = ConfigService::update_urls(current_urls.clone()) {
        let url_list_html = generate_url_list_html(&current_urls);
        let error_message = format!("設定ファイルの更新に失敗しました: {}", e);
        let html = generate_edit_page_html(&url_list_html, &error_message, true);
        return Ok(Html(html).into_response());
    }

    // 成功時は一覧ページへリダイレクト
    Ok(Redirect::to("/admin/list").into_response())
}
```

**学習ポイント:**

1. **Axumのフォーム抽出**
   ```rust
   Form(form): Form<UrlUpdateForm>
   ```
   - `Form<T>`エクストラクタがHTTPリクエストボディを自動的にパース
   - Content-Type: application/x-www-form-urlencodedを処理

2. **配列要素の安全な削除**
   ```rust
   delete_indices.sort_unstable();
   delete_indices.reverse();
   ```
   - 配列から複数要素を削除する際は、後ろから削除するのが定石
   - 前から削除するとインデックスがずれてしまう
   - `sort_unstable()`: 安定ソートが不要な場合に高速

3. **文字列の行分割と処理**
   ```rust
   form.new_urls
       .lines()
       .map(|line| line.trim().to_string())
       .filter(|line| !line.is_empty())
       .collect()
   ```
   - `lines()`: 改行で分割（`\n`、`\r\n`両方に対応）
   - `trim()`: 前後の空白を削除
   - `filter()`: 空行を除外
   - イテレータチェーンで効率的に処理

4. **エラーハンドリングとユーザーフィードバック**
   ```rust
   if let Err(e) = UrlValidationService::validate_urls(&new_urls) {
       let html = generate_edit_page_html(&url_list_html, &error_message, true);
       return Ok(Html(html).into_response());
   }
   ```
   - バリデーションエラー時は編集画面を再表示
   - エラーメッセージを表示してユーザーに修正を促す
   - PRG（Post-Redirect-Get）パターンの例外：エラー時は再表示

5. **成功時のリダイレクト**
   ```rust
   Ok(Redirect::to("/admin/list").into_response())
   ```
   - PRGパターン: POST成功後はGETページへリダイレクト
   - ブラウザの再読み込みで重複送信を防ぐ
   - `into_response()`: 異なる型を統一的なレスポンス型に変換

### 8.3 URL編集ビューの作成

#### HTMLテンプレートの生成

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

    format!(r#"
<!DOCTYPE html>
<html lang="ja">
<head>
    <!-- ... -->
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
            <h2>URL編集</h2>
            {}
            <form method="POST" action="/admin/edit">
                <div class="form-group">
                    <label>現在のURL一覧（削除する場合はチェック）</label>
                    {}
                </div>
                
                <div class="form-group">
                    <label for="new_urls">新規URL追加（1行に1つのURLを入力）</label>
                    <textarea name="new_urls" id="new_urls" 
                              placeholder="https://example.com&#10;https://example.org"></textarea>
                    <div class="help-text">
                        ※ HTTP/HTTPSスキームを持つ有効なURLを入力してください<br>
                        ※ 複数のURLを追加する場合は、改行で区切ってください
                    </div>
                </div>
                
                <div class="form-actions">
                    <button type="submit" class="btn btn-primary">保存</button>
                    <a href="/admin/list" class="btn btn-secondary">キャンセル</a>
                </div>
            </form>
        </div>
    </div>
</body>
</html>
    "#, error_html, url_list_html)
}
```

**学習ポイント:**

1. **HTMLエスケープの重要性**
   - すべてのユーザー入力を`html_escape::encode_text()`でエスケープ
   - XSS（クロスサイトスクリプティング）攻撃を防ぐ
   - セキュリティの基本原則

2. **フォームの設計**
   ```html
   <input type="checkbox" name="delete_indices" value="0">
   <input type="checkbox" name="delete_indices" value="1">
   ```
   - 同じname属性を持つチェックボックスは配列として送信される
   - Rustの`Vec<usize>`に自動マッピング

3. **テキストエリアの活用**
   ```html
   <textarea name="new_urls" placeholder="https://example.com&#10;https://example.org"></textarea>
   ```
   - `&#10;`はHTMLエンティティで改行を表現
   - 複数行入力に適したUI

4. **条件付きレンダリング**
   ```rust
   if url_list_html.is_empty() {
       r#"<div class="empty-state">現在、監視中のURLはありません。</div>"#
   } else {
       url_list_html
   }
   ```
   - データがない場合の適切なフィードバック

### 8.4 ルーティングの設定

`src/app.rs`にルートを追加しました。

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

**学習ポイント:**

1. **同じパスに複数のHTTPメソッド**
   ```rust
   .add("/admin/edit", get(admin_edit::edit))
   .add("/admin/edit", post(admin_edit::update))
   ```
   - GETリクエスト: フォーム表示
   - POSTリクエスト: フォーム送信処理
   - RESTfulな設計パターン

2. **ミドルウェアの適用**
   ```rust
   .layer(axum::middleware::from_fn_with_state(
       ctx.clone(),
       require_auth,
   ))
   ```
   - `layer()`で認証ミドルウェアを適用
   - `protected_routes`に追加されたすべてのルートに認証が必要
   - 認証ロジックを一箇所で管理

3. **ルートの階層化**
   - `public_routes`: 認証不要（ログイン画面など）
   - `protected_routes`: 認証必要（管理画面）
   - セキュリティの分離が明確

## Locoフレームワークの特徴

### 1. Axumベースのルーティング

Locoは内部でAxumを使用しています。Axumの特徴：
- 型安全なエクストラクタ（`State`, `Form`, `Json`など）
- コンパイル時の型チェック
- 高いパフォーマンス

### 2. エラーハンドリング

```rust
Result<impl IntoResponse>
```
- Locoの標準的なエラーハンドリングパターン
- `?`オペレータで簡潔にエラー伝播
- `map_err()`でエラー型を統一

### 3. レスポンスの柔軟性

```rust
Ok(Html(html).into_response())
Ok(Redirect::to("/admin/list").into_response())
```
- `IntoResponse`トレイトで異なる型を統一
- 同じ関数から異なるレスポンスを返せる

## セキュリティのベストプラクティス

### 1. XSS対策

```rust
html_escape::encode_text(url)
```
- すべてのユーザー入力をエスケープ
- HTMLインジェクション攻撃を防ぐ

### 2. 認証の強制

```rust
.layer(axum::middleware::from_fn_with_state(ctx.clone(), require_auth))
```
- ミドルウェアで認証を強制
- 認証なしでは管理画面にアクセス不可

### 3. バリデーション

```rust
UrlValidationService::validate_urls(&new_urls)
```
- 入力データを必ず検証
- 不正なデータの保存を防ぐ

## まとめ

このタスクで学んだ主要な概念：

1. **Axumのフォーム処理**: `Form<T>`エクストラクタでフォームデータを構造体に自動変換
2. **PRGパターン**: POST後のリダイレクトで重複送信を防ぐ
3. **エラーハンドリング**: ユーザーフレンドリーなエラー表示
4. **セキュリティ**: HTMLエスケープと認証ミドルウェア
5. **イテレータチェーン**: Rustの関数型プログラミングスタイル
6. **RESTfulルーティング**: 同じパスに複数のHTTPメソッド

これらの技術を組み合わせることで、安全で使いやすいWeb管理画面を実装できました。
