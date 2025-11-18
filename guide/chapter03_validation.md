# 第3章: URL検証機能の実装

## この章で学ぶこと

- バリデーション（検証）の重要性
- 外部クレートの使い方
- カスタムエラー型の設計
- 文字列処理とトリミング
- イテレータとエラー伝播
- ユニットテストの書き方

---

## 3.1 バリデーションとは

### バリデーションの重要性

**バリデーション（検証）**は、入力データが正しい形式かどうかをチェックすることです。

**なぜ必要？**
- ❌ 無効なデータがシステムに入るのを防ぐ
- 🔒 セキュリティリスクを減らす
- 🐛 バグを早期に発見
- 📝 わかりやすいエラーメッセージを提供

**例: URLの検証**
- `https://example.com` ✅ 有効
- `http://localhost:8080` ✅ 有効
- `ftp://example.com` ❌ 無効（HTTP/HTTPSのみ）
- `not-a-url` ❌ 無効（URL形式ではない）
- `   ` ❌ 無効（空）

### このシステムでの役割

```
ユーザ入力
    ↓
URL検証 ← 今回実装
    ↓
設定ファイルに保存
```

無効なURLを設定ファイルに書き込む前に、検証で弾きます。

---

## 3.2 外部クレートの追加

### urlクレートとは

**urlクレート**は、URLの解析と検証を行うライブラリです。

**特徴:**
- RFC 3986準拠（URL標準規格）
- 実績があり、広く使われている
- 自前で実装するより安全

### Cargo.tomlへの追加

`Cargo.toml`に追加（既に追加済み）：

```toml
[dependencies]
url = "2.5"
```

### 依存関係のインストール

```bash
cargo build
```

初回は、urlクレートがダウンロードされます。

---

## 3.3 ValidationErrorの定義

### エラー型の設計

`src/services/url_validation_service.rs`を作成します。

```rust
use thiserror::Error;
use url::Url;

// URL検証サービス
// URLの形式検証を担当

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

### コードの詳細解説

#### use文
```rust
use thiserror::Error;
use url::Url;
```

- `thiserror::Error`: カスタムエラー型を定義
- `url::Url`: URL解析機能

#### エラーバリアント

##### InvalidUrlFormat
```rust
#[error("無効なURL形式です: {0}")]
InvalidUrlFormat(String),
```

- URL形式が正しくない場合
- 例: `"not-a-url"`、`"example.com"`（スキームなし）
- `String`に問題のあるURLを保存

##### InvalidScheme
```rust
#[error("HTTP/HTTPSスキームが必要です: {0}")]
InvalidScheme(String),
```

- HTTP/HTTPS以外のスキームの場合
- 例: `"ftp://example.com"`、`"ws://example.com"`

##### EmptyUrl
```rust
#[error("URLが空です")]
EmptyUrl,
```

- 空文字列または空白のみの場合
- データを持たない（ユニットバリアント）

### エラー設計のポイント

**良いエラー設計:**
- エラーの種類を明確に区別
- デバッグに必要な情報を含める
- わかりやすいメッセージ

**悪い例:**
```rust
#[derive(Debug, Error)]
#[error("エラーが発生しました")]
pub struct ValidationError;
```

- 何が問題かわからない
- デバッグ情報がない

---

## 3.4 UrlValidationServiceの実装

### 構造体の定義

```rust
pub struct UrlValidationService;
```

空の構造体ですが、関連関数を実装します。

### validate_url()の実装

単一URLを検証する関数です。

```rust
impl UrlValidationService {
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
        // 空文字列チェック
        if url.trim().is_empty() {
            return Err(ValidationError::EmptyUrl);
        }

        // URL形式の検証
        let parsed_url = Url::parse(url)
            .map_err(|e| ValidationError::InvalidUrlFormat(format!("{}: {}", url, e)))?;

        // HTTP/HTTPSスキームチェック
        let scheme = parsed_url.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(ValidationError::InvalidScheme(url.to_string()));
        }

        Ok(())
    }
}
```

### コードの詳細解説

#### ドキュメントコメント
```rust
/// 単一URLの検証
/// 
/// URLの形式を検証し、HTTP/HTTPSスキームを持つことを確認する
```

- `///`で始まるコメントはドキュメントコメント
- `cargo doc`でHTMLドキュメントを生成
- IDEでホバーすると表示される

#### 空文字列チェック
```rust
if url.trim().is_empty() {
    return Err(ValidationError::EmptyUrl);
}
```

**trim()とは:**
- 文字列の前後の空白を除去
- 空白: スペース、タブ、改行など

**例:**
```rust
"  hello  ".trim()  // "hello"
"   ".trim()        // ""
```

**is_empty()とは:**
- 文字列が空かどうかをチェック
- `true`: 空文字列
- `false`: 何か文字がある

**なぜtrim()が必要？**
```rust
// ユーザが誤って空白を入力
let url = "   ";
url.is_empty()       // false（空白があるため）
url.trim().is_empty() // true（空白を除去すると空）
```

#### URL形式の検証
```rust
let parsed_url = Url::parse(url)
    .map_err(|e| ValidationError::InvalidUrlFormat(format!("{}: {}", url, e)))?;
```

**Url::parse()とは:**
- URL文字列を解析
- 成功: `Ok(Url)`
- 失敗: `Err(ParseError)`

**例:**
```rust
// 成功
Url::parse("https://example.com")  // Ok(Url { ... })

// 失敗
Url::parse("not-a-url")            // Err(ParseError)
Url::parse("example.com")          // Err(ParseError) - スキームなし
```

**map_err()とは:**
- エラー型を変換する
- `ParseError` → `ValidationError`

**format!()マクロ:**
```rust
format!("{}: {}", url, e)
```

- 文字列を組み立てる
- `{}`: プレースホルダー
- 例: `"not-a-url: relative URL without a base"`

**なぜエラーにURLを含める？**
```rust
// 悪い例: どのURLが問題かわからない
Err(ValidationError::InvalidUrlFormat("relative URL without a base".to_string()))

// 良い例: 問題のあるURLが明確
Err(ValidationError::InvalidUrlFormat("not-a-url: relative URL without a base".to_string()))
```

#### スキームのチェック
```rust
let scheme = parsed_url.scheme();
if scheme != "http" && scheme != "https" {
    return Err(ValidationError::InvalidScheme(url.to_string()));
}
```

**scheme()とは:**
- URLのスキーム部分を取得
- `https://example.com` → `"https"`
- `ftp://example.com` → `"ftp"`

**なぜHTTP/HTTPSのみ？**
- Telegrafの`http_response`プラグインはHTTP/HTTPSのみサポート
- セキュリティ: 意図しないプロトコルへのアクセスを防ぐ

**例:**
```rust
// OK
"https://example.com"  // scheme = "https"
"http://localhost"     // scheme = "http"

// NG
"ftp://example.com"    // scheme = "ftp"
"ws://example.com"     // scheme = "ws"
"file:///path/to/file" // scheme = "file"
```

#### 成功時
```rust
Ok(())
```

- `()`: ユニット型（値なし）
- 検証は成功/失敗のみで、値を返さない

---

## 3.5 複数URL検証の実装

### validate_urls()の実装

```rust
/// 複数URLの検証
/// 
/// URL一覧を検証し、すべてが有効であることを確認する
/// 
/// # Arguments
/// * `urls` - 検証するURL文字列のスライス
/// 
/// # Returns
/// * `Ok(())` - すべてのURLが有効な場合
/// * `Err(ValidationError)` - いずれかのURLが無効な場合（最初のエラーを返す）
pub fn validate_urls(urls: &[String]) -> Result<(), ValidationError> {
    for url in urls {
        Self::validate_url(url)?;
    }
    Ok(())
}
```

### コードの詳細解説

#### 引数の型
```rust
urls: &[String]
```

- `&[String]`: 文字列のスライス（参照）
- 配列やベクタを受け取れる

**例:**
```rust
let vec = vec!["https://example.com".to_string()];
validate_urls(&vec);  // ベクタを渡す

let array = ["https://example.com".to_string()];
validate_urls(&array); // 配列を渡す
```

#### forループ
```rust
for url in urls {
    Self::validate_url(url)?;
}
```

- `urls`の各要素を順番に処理
- `Self::validate_url()`: 単一URL検証を呼び出す
- `?`: エラー時は即座にリターン（フェイルファスト）

**フェイルファストとは:**
```rust
let urls = vec![
    "https://example.com".to_string(),
    "invalid-url".to_string(),      // ← ここでエラー
    "https://another.com".to_string(), // ← 実行されない
];

validate_urls(&urls); // Err(InvalidUrlFormat(...))
```

最初のエラーで即座に関数から返ります。

#### 成功時
```rust
Ok(())
```

すべてのURLが有効な場合のみ、`Ok(())`を返します。

---

## 3.6 モジュールへの追加

### src/services/mod.rsの更新

```rust
pub mod config_service;
pub mod url_validation_service;
```

これで、他のモジュールから使えるようになります。

```rust
use crate::services::url_validation_service::UrlValidationService;
```

---

## 3.7 動作確認

### テストコードの作成

`src/main.rs`に一時的にテストコードを追加します。

```rust
use kiro_test::services::url_validation_service::UrlValidationService;

fn main() {
    // テストケース
    let test_urls = vec![
        "https://example.com",
        "http://localhost:8080",
        "ftp://example.com",  // エラーになる
        "not-a-url",          // エラーになる
        "   ",                // エラーになる
    ];

    for url in test_urls {
        match UrlValidationService::validate_url(url) {
            Ok(_) => println!("✅ OK: {}", url),
            Err(e) => println!("❌ NG: {} - {}", url, e),
        }
    }
}
```

### 実行

```bash
cargo run
```

**出力例:**
```
✅ OK: https://example.com
✅ OK: http://localhost:8080
❌ NG: ftp://example.com - HTTP/HTTPSスキームが必要です: ftp://example.com
❌ NG: not-a-url - 無効なURL形式です: not-a-url: relative URL without a base
❌ NG:     - URLが空です
```

---

## 3.8 ユニットテストの作成

### テストの重要性

**なぜテストが必要？**
- 🐛 バグを早期に発見
- 🔒 リファクタリング時の安全性
- 📝 仕様のドキュメント化
- 🚀 自信を持ってコードを変更

### テストコードの追加

`src/services/url_validation_service.rs`の最後に追加します。

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

### テストコードの詳細解説

#### #[cfg(test)]
```rust
#[cfg(test)]
mod tests {
```

- `#[cfg(test)]`: テスト時のみコンパイル
- 本番ビルドには含まれない

#### use super::*
```rust
use super::*;
```

- 親モジュールのすべてをインポート
- `UrlValidationService`や`ValidationError`が使える

#### #[test]
```rust
#[test]
fn test_valid_https_url() {
```

- `#[test]`: テスト関数であることを示す
- `cargo test`で実行される

#### assert!()
```rust
assert!(result.is_ok());
```

- 条件が`true`であることを確認
- `false`の場合、テスト失敗

#### matches!()
```rust
assert!(matches!(result, Err(ValidationError::EmptyUrl)));
```

- パターンマッチングを確認
- エラーバリアントが正しいかチェック

**例:**
```rust
let result = Err(ValidationError::EmptyUrl);

// OK
assert!(matches!(result, Err(ValidationError::EmptyUrl)));

// NG
assert!(matches!(result, Err(ValidationError::InvalidScheme(_))));
```

#### アンダースコア（_）
```rust
Err(ValidationError::InvalidScheme(_))
```

- 値を無視
- 型だけチェック

### テストの実行

```bash
cargo test
```

**出力例:**
```
running 8 tests
test services::url_validation_service::tests::test_empty_url ... ok
test services::url_validation_service::tests::test_invalid_format ... ok
test services::url_validation_service::tests::test_invalid_scheme ... ok
test services::url_validation_service::tests::test_valid_http_url ... ok
test services::url_validation_service::tests::test_valid_https_url ... ok
test services::url_validation_service::tests::test_validate_multiple_urls ... ok
test services::url_validation_service::tests::test_validate_multiple_urls_with_invalid ... ok
test services::url_validation_service::tests::test_whitespace_only_url ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 特定のテストのみ実行

```bash
# 名前でフィルタ
cargo test test_valid

# 出力を表示
cargo test -- --nocapture
```

---

## 3.9 ConfigServiceとの統合

### 統合の例

ConfigServiceでURL更新時に検証を行います。

```rust
use crate::services::url_validation_service::UrlValidationService;

pub fn update_urls_with_validation(urls: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // 1. URLを検証
    UrlValidationService::validate_urls(&urls)?;
    
    // 2. 検証が成功したら設定ファイルを更新
    ConfigService::update_urls(urls)?;
    
    Ok(())
}
```

**メリット:**
- 無効なURLが設定ファイルに書き込まれない
- エラーを早期に発見
- 責務の分離（検証と保存を分ける）

---

## 3.10 エラーハンドリングのパターン

### パターン1: match式
```rust
match UrlValidationService::validate_url(url) {
    Ok(_) => println!("URLは有効です"),
    Err(ValidationError::EmptyUrl) => println!("URLが空です"),
    Err(ValidationError::InvalidScheme(url)) => {
        println!("無効なスキーム: {}", url)
    }
    Err(ValidationError::InvalidUrlFormat(msg)) => {
        println!("無効な形式: {}", msg)
    }
}
```

### パターン2: if let
```rust
if let Err(e) = UrlValidationService::validate_url(url) {
    eprintln!("エラー: {}", e);
}
```

### パターン3: ?演算子
```rust
fn process_url(url: &str) -> Result<(), ValidationError> {
    UrlValidationService::validate_url(url)?;
    // 検証成功後の処理...
    Ok(())
}
```

---

## 3.11 まとめ

この章では、以下を学びました：

- ✅ バリデーションの重要性
- ✅ 外部クレート（url）の使い方
- ✅ カスタムエラー型の設計
- ✅ 文字列処理（trim、is_empty）
- ✅ エラー変換（map_err）
- ✅ イテレータとエラー伝播
- ✅ ユニットテストの書き方
- ✅ ドキュメントコメント

**重要なポイント:**
- 入力データは必ず検証する
- エラーメッセージは具体的に
- テストで動作を保証
- 外部クレートを活用して車輪の再発明を避ける

---

## 次のステップ

次の章では、**認証機能**を実装します。ユーザ認証とセッション管理を学びます。

[第4章: 認証機能の実装](./chapter04_authentication.md)に進みましょう！
