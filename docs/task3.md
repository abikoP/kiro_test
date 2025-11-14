# タスク3: URL検証機能の実装

## 概要

このタスクでは、URLの形式を検証する`UrlValidationService`を実装しました。このサービスは、HTTP/HTTPSスキームを持つ有効なURLかどうかをチェックし、Telegraf設定ファイルに登録する前のバリデーション機能を提供します。

## 実装したファイル

- `src/services/url_validation_service.rs` - URL検証サービスの実装

## 学習ポイント

### 1. 外部クレートの活用 - `url`クレート

```rust
use url::Url;

let parsed_url = Url::parse("https://example.com")?;
```

**解説:**
- `url`クレートは、RFC 3986に準拠したURL解析を提供する標準的なライブラリ
- `Url::parse()`は文字列をパースし、URLの各要素（スキーム、ホスト、パスなど）を抽出
- パースに失敗すると`ParseError`を返すため、`Result`型で安全にエラーハンドリング可能
- 自前でURL検証ロジックを書くよりも、実績のあるクレートを使う方が安全で保守性が高い

**Cargo.tomlへの追加:**
```toml
[dependencies]
url = "2.5"
```

### 2. カスタムエラー型の設計

```rust
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("無効なURL形式です: {0}")]
    InvalidUrlFormat(String),
    
    #[error("HTTP/HTTPSスキームが必要です: {0}")]
    InvalidScheme(String),
    
    #[error("URLが空です")]
    EmptyUrl,
}
```

**解説:**
- エラーの種類ごとに列挙型のバリアントを定義
- `InvalidUrlFormat`と`InvalidScheme`は`String`を保持し、問題のあるURLを含める
- `EmptyUrl`はデータを持たないユニットバリアント
- エラーメッセージに日本語を使用することで、エンドユーザーにも分かりやすい

**エラー設計のベストプラクティス:**
- エラーの原因を明確に区別できるようにバリアントを分ける
- デバッグに必要な情報（URL文字列など）をエラーに含める
- `#[error("...")]`属性で人間が読みやすいメッセージを提供

### 3. 文字列のトリミングと空チェック

```rust
if url.trim().is_empty() {
    return Err(ValidationError::EmptyUrl);
}
```

**解説:**
- `trim()`は文字列の前後の空白文字（スペース、タブ、改行など）を除去
- `is_empty()`で空文字列かどうかをチェック
- `"   "`のような空白のみの文字列も空として扱う
- 早期リターンパターンで、無効な入力を即座に拒否

**なぜトリミングが必要か:**
- ユーザー入力には意図しない空白が含まれることが多い
- `" https://example.com "`のような入力を正規化
- セキュリティ上も、予期しない空白による問題を防ぐ

### 4. エラー変換とコンテキストの追加

```rust
let parsed_url = Url::parse(url)
    .map_err(|e| ValidationError::InvalidUrlFormat(format!("{}: {}", url, e)))?;
```

**解説:**
- `Url::parse()`は`Result<Url, ParseError>`を返す
- `map_err()`でエラー型を`ParseError`から`ValidationError`に変換
- `format!()`マクロで、元のURL文字列とエラーメッセージを組み合わせる
- `?`演算子でエラーを伝播（エラーの場合は関数から早期リターン）

**エラーコンテキストの重要性:**
```rust
// 悪い例: どのURLが問題か分からない
Err(ValidationError::InvalidUrlFormat("relative URL without a base".to_string()))

// 良い例: 問題のあるURLが明確
Err(ValidationError::InvalidUrlFormat("example.com: relative URL without a base".to_string()))
```

### 5. URLスキームの検証

```rust
let scheme = parsed_url.scheme();
if scheme != "http" && scheme != "https" {
    return Err(ValidationError::InvalidScheme(url.to_string()));
}
```

**解説:**
- `scheme()`メソッドでURLのスキーム部分（`http`、`https`など）を取得
- 文字列比較で、許可されたスキームかどうかをチェック
- `ftp://`、`file://`、`ws://`などの他のスキームを拒否
- Telegrafの`http_response`プラグインはHTTP/HTTPSのみをサポートするため

**スキーム検証の意義:**
- セキュリティ: 意図しないプロトコルへのアクセスを防ぐ
- 互換性: Telegrafがサポートするプロトコルのみを許可
- エラーの早期発見: 設定ファイルに書き込む前に問題を検出

### 6. イテレータとエラー伝播

```rust
pub fn validate_urls(urls: &[String]) -> Result<(), ValidationError> {
    for url in urls {
        Self::validate_url(url)?;
    }
    Ok(())
}
```

**解説:**
- `&[String]`はスライス型で、配列やベクタの参照を受け取れる
- `for`ループで各URLを順番に検証
- `?`演算子により、最初のエラーで即座に関数から返る（フェイルファスト）
- すべてのURLが有効な場合のみ`Ok(())`を返す

**代替実装 - イテレータメソッドを使った関数型スタイル:**
```rust
pub fn validate_urls(urls: &[String]) -> Result<(), ValidationError> {
    urls.iter()
        .try_for_each(|url| Self::validate_url(url))
}
```
- `try_for_each()`は、すべての要素に対して処理を実行し、最初のエラーで停止
- より関数型プログラミング的なスタイル
- 今回の実装では可読性を優先して`for`ループを使用

### 7. ドキュメントコメント

```rust
/// 単一URLの検証
/// 
/// URLの形式を検証し、HTTP/HTTPSスキームを持つことを確認する
/// 
/// # Arguments
/// * `url` - 検証するURL文字列
/// 
/// # Returns
/// * `Ok(())` - URLが有効な場合
/// * `Err(ValidationError)` - URLが無効な場合
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    // ...
}
```

**解説:**
- `///`で始まるコメントはドキュメントコメント（Doc Comments）
- `cargo doc`コマンドでHTMLドキュメントを自動生成
- `# Arguments`、`# Returns`などのセクションで構造化
- IDEでホバーすると、このドキュメントが表示される

**ドキュメントのベストプラクティス:**
- 関数の目的を簡潔に説明
- 引数と戻り値の意味を明記
- エラーケースを説明
- 使用例を`# Examples`セクションに記載（今回は省略）

## Rustの型システムによる安全性

### 借用チェッカーの活用

```rust
pub fn validate_url(url: &str) -> Result<(), ValidationError>
```

- `&str`は文字列スライスの不変参照
- 所有権を移動せず、読み取り専用でアクセス
- 呼び出し側は引数の所有権を保持し続ける

```rust
let my_url = "https://example.com".to_string();
UrlValidationService::validate_url(&my_url)?;
// my_urlはまだ使用可能
println!("検証したURL: {}", my_url);
```

### Result型による明示的なエラーハンドリング

```rust
Result<(), ValidationError>
```

- 成功時は`()`（ユニット型）を返す - 検証は副作用のみで値を返さない
- 失敗時は`ValidationError`を返す
- コンパイラが`Result`の処理を強制するため、エラーの見落としを防ぐ

## 実践的な使用例

### 基本的な使用

```rust
use crate::services::url_validation_service::UrlValidationService;

// 単一URLの検証
match UrlValidationService::validate_url("https://example.com") {
    Ok(_) => println!("URLは有効です"),
    Err(e) => eprintln!("エラー: {}", e),
}

// 複数URLの検証
let urls = vec![
    "https://api.example.com/health".to_string(),
    "http://localhost:8080/metrics".to_string(),
];

if let Err(e) = UrlValidationService::validate_urls(&urls) {
    eprintln!("URL検証エラー: {}", e);
}
```

### ConfigServiceとの統合

```rust
use crate::services::config_service::ConfigService;
use crate::services::url_validation_service::UrlValidationService;

pub fn update_telegraf_urls(new_urls: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // 1. URLを検証
    UrlValidationService::validate_urls(&new_urls)?;
    
    // 2. 検証が成功したら設定ファイルを更新
    ConfigService::update_urls(new_urls)?;
    
    Ok(())
}
```

**解説:**
- 検証を先に行うことで、無効なURLが設定ファイルに書き込まれるのを防ぐ
- エラーハンドリングを分離することで、各サービスの責務が明確になる
- `Box<dyn std::error::Error>`で異なるエラー型を統一的に扱う

### Webコントローラーでの使用（Locoフレームワーク）

```rust
use loco_rs::prelude::*;
use crate::services::url_validation_service::{UrlValidationService, ValidationError};

pub async fn validate_url_endpoint(
    State(_ctx): State<AppContext>,
    Json(payload): Json<UrlRequest>,
) -> Result<Response> {
    match UrlValidationService::validate_url(&payload.url) {
        Ok(_) => {
            format::json(serde_json::json!({
                "status": "valid",
                "url": payload.url
            }))
        }
        Err(ValidationError::EmptyUrl) => {
            format::json(serde_json::json!({
                "status": "error",
                "message": "URLが空です"
            }))
        }
        Err(ValidationError::InvalidUrlFormat(msg)) => {
            format::json(serde_json::json!({
                "status": "error",
                "message": msg
            }))
        }
        Err(ValidationError::InvalidScheme(url)) => {
            format::json(serde_json::json!({
                "status": "error",
                "message": format!("HTTP/HTTPSスキームが必要です: {}", url)
            }))
        }
    }
}
```

## テスト戦略

### ユニットテストの例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url() {
        let result = UrlValidationService::validate_url("https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_http_url() {
        let result = UrlValidationService::validate_url("http://localhost:8080");
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_url() {
        let result = UrlValidationService::validate_url("");
        assert!(matches!(result, Err(ValidationError::EmptyUrl)));
    }

    #[test]
    fn test_whitespace_only_url() {
        let result = UrlValidationService::validate_url("   ");
        assert!(matches!(result, Err(ValidationError::EmptyUrl)));
    }

    #[test]
    fn test_invalid_scheme() {
        let result = UrlValidationService::validate_url("ftp://example.com");
        assert!(matches!(result, Err(ValidationError::InvalidScheme(_))));
    }

    #[test]
    fn test_invalid_format() {
        let result = UrlValidationService::validate_url("not-a-url");
        assert!(matches!(result, Err(ValidationError::InvalidUrlFormat(_))));
    }

    #[test]
    fn test_validate_multiple_urls() {
        let urls = vec![
            "https://example.com".to_string(),
            "http://localhost:8080".to_string(),
        ];
        let result = UrlValidationService::validate_urls(&urls);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_multiple_urls_with_invalid() {
        let urls = vec![
            "https://example.com".to_string(),
            "ftp://invalid.com".to_string(),
        ];
        let result = UrlValidationService::validate_urls(&urls);
        assert!(result.is_err());
    }
}
```

**テストのポイント:**
- 正常系と異常系の両方をテスト
- `matches!`マクロでエラーバリアントを検証
- エッジケース（空白のみの文字列など）もカバー
- 複数URL検証のテストも含める

## Locoフレームワークとの統合

### サービス層のアーキテクチャ

```
src/
├── controllers/     # HTTPリクエストを処理
│   └── urls.rs      # URL管理のエンドポイント
├── services/        # ビジネスロジック
│   ├── config_service.rs          # 設定ファイル操作
│   └── url_validation_service.rs  # URL検証
└── models/          # データモデル
```

**責務の分離:**
- **Controller**: HTTPリクエスト/レスポンスの処理、認証・認可
- **Service**: ビジネスロジック、バリデーション、外部システムとの連携
- **Model**: データ構造、データベース操作

### サービスの特徴

1. **状態を持たない**: `UrlValidationService`は構造体だが、フィールドを持たない
2. **関連関数のみ**: すべてのメソッドが`self`を取らない静的メソッド
3. **純粋関数的**: 同じ入力に対して常に同じ出力を返す
4. **副作用なし**: 検証のみを行い、ファイルやデータベースを変更しない

## まとめ

このタスクで学んだRustの重要な概念：

1. **外部クレートの活用**: `url`クレートによる標準準拠のURL解析
2. **エラー設計**: 明確なエラーバリアントとコンテキスト情報の提供
3. **文字列処理**: トリミングと空チェックによる堅牢な入力検証
4. **エラー変換**: `map_err()`による異なるエラー型の変換
5. **イテレータ**: `for`ループと`?`演算子によるエラー伝播
6. **ドキュメント**: Doc Commentsによる自動ドキュメント生成
7. **型安全性**: `Result`型による明示的なエラーハンドリング

### 次のステップ

- タスク4: コントローラーの実装で、このサービスをHTTP APIとして公開
- タスク5: フロントエンドの実装で、ユーザーインターフェースを構築
- タスク6: 統合テストで、全体の動作を検証

このURL検証サービスは、システム全体のデータ品質を保証する重要な役割を果たします。
