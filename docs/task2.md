# タスク2: 設定ファイル操作機能の実装

## 概要

このタスクでは、Telegrafの設定ファイル（TOML形式）を読み書きするための`ConfigService`を実装しました。このサービスは、`./conf/telegraf.conf`ファイルからHTTP Response InputのURL一覧を取得・更新する機能を提供します。

## 実装したファイル

- `src/services/config_service.rs` - 設定ファイル操作サービスの実装
- `src/services/mod.rs` - サービスモジュールのエクスポート

## 学習ポイント

### 1. Rustのエラーハンドリング - `thiserror`クレート

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("設定ファイルが見つかりません: {0}")]
    FileNotFound(String),
    
    #[error("設定ファイルの読み込みに失敗しました: {0}")]
    ReadError(String),
    
    #[error("設定ファイルの書き込みに失敗しました: {0}")]
    WriteError(String),
    
    #[error("TOMLのパースに失敗しました: {0}")]
    ParseError(String),
    
    #[error("http_responseセクションが見つかりません")]
    HttpResponseSectionNotFound,
}
```

**解説:**
- `thiserror`は、カスタムエラー型を簡単に定義できるクレートです
- `#[derive(Error)]`マクロで`std::error::Error`トレイトを自動実装
- `#[error("...")]`属性でエラーメッセージを定義（`{0}`は列挙型の値を埋め込む）
- `Result<T, ConfigError>`として使用することで、型安全なエラーハンドリングが可能

### 2. Serdeによるデータ構造の定義

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelegrafConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_tags: Option<toml::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<toml::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<toml::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<InputsConfig>,
}
```

**解説:**
- `Serialize`/`Deserialize`トレイトで、構造体とTOML形式の相互変換を自動化
- `#[serde(skip_serializing_if = "Option::is_none")]`は、`None`の場合にシリアライズをスキップ
  - これにより、存在しないフィールドをTOMLファイルに出力しない
- `Option<T>`を使うことで、設定ファイルに存在しない項目を柔軟に扱える
- `toml::Value`は動的な型で、構造が不明な部分を保持できる

### 3. ネストした構造体の定義

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_response: Option<Vec<HttpResponseInput>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpResponseInput {
    pub urls: Vec<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_timeout: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<bool>,
}
```

**解説:**
- TOML構造を反映したネストした構造体を定義
- `[[inputs.http_response]]`は配列なので`Vec<HttpResponseInput>`として表現
- 必須フィールド（`urls`）と任意フィールド（`response_timeout`等）を区別
- Rustの型システムにより、コンパイル時に構造の正しさを保証

### 4. ファイル操作とエラー変換

```rust
pub fn read_config() -> Result<TelegrafConfig, ConfigError> {
    let path = Path::new(DEFAULT_CONFIG_PATH);
    
    if !path.exists() {
        return Err(ConfigError::FileNotFound(DEFAULT_CONFIG_PATH.to_string()));
    }
    
    let content = fs::read_to_string(path)
        .map_err(|e| ConfigError::ReadError(e.to_string()))?;
    
    let config: TelegrafConfig = toml::from_str(&content)
        .map_err(|e| ConfigError::ParseError(e.to_string()))?;
    
    Ok(config)
}
```

**解説:**
- `Path::new()`でファイルパスを作成
- `path.exists()`でファイルの存在確認
- `fs::read_to_string()`でファイル全体を文字列として読み込み
- `map_err()`で標準エラー（`std::io::Error`）をカスタムエラー（`ConfigError`）に変換
- `?`演算子で、エラーが発生した場合は早期リターン
- `toml::from_str()`でTOML文字列を構造体にデシリアライズ

### 5. Optionチェーンとエラーハンドリング

```rust
pub fn get_urls() -> Result<Vec<String>, ConfigError> {
    let config = Self::read_config()?;
    
    let urls = config
        .inputs
        .and_then(|inputs| inputs.http_response)
        .and_then(|http_responses| http_responses.first().cloned())
        .map(|http_response| http_response.urls)
        .ok_or(ConfigError::HttpResponseSectionNotFound)?;
    
    Ok(urls)
}
```

**解説:**
- `and_then()`で`Option`をチェーンして、ネストした構造を安全に辿る
- `first()`で配列の最初の要素を取得（`Option<&T>`を返す）
- `cloned()`で参照から所有権のある値に変換
- `map()`で値を変換
- `ok_or()`で`Option`を`Result`に変換（`None`の場合はエラー）
- Null安全性を保ちながら、簡潔にデータを取得

### 6. 可変参照による構造体の更新

```rust
pub fn update_urls(urls: Vec<String>) -> Result<(), ConfigError> {
    let path = Path::new(DEFAULT_CONFIG_PATH);
    
    // 現在の設定を読み込む
    let mut config = Self::read_config()?;
    
    // http_responseセクションを更新
    if let Some(ref mut inputs) = config.inputs {
        if let Some(ref mut http_responses) = inputs.http_response {
            if let Some(http_response) = http_responses.first_mut() {
                http_response.urls = urls;
            } else {
                return Err(ConfigError::HttpResponseSectionNotFound);
            }
        } else {
            return Err(ConfigError::HttpResponseSectionNotFound);
        }
    } else {
        return Err(ConfigError::HttpResponseSectionNotFound);
    }
    
    // TOML形式にシリアライズ
    let toml_string = toml::to_string_pretty(&config)
        .map_err(|e| ConfigError::WriteError(e.to_string()))?;
    
    // ファイルに書き込む
    fs::write(path, toml_string)
        .map_err(|e| ConfigError::WriteError(e.to_string()))?;
    
    Ok(())
}
```

**解説:**
- `let mut config`で可変な変数を宣言
- `ref mut`で可変参照を取得（所有権を移動せずに変更可能）
- `first_mut()`で配列の最初の要素への可変参照を取得
- ネストした`if let`で各階層の存在を確認しながら更新
- `toml::to_string_pretty()`で構造体を整形されたTOML文字列に変換
- `fs::write()`でファイルに書き込み（既存ファイルは上書き）

### 7. 関連関数（Associated Function）の使用

```rust
impl ConfigService {
    pub fn read_config() -> Result<TelegrafConfig, ConfigError> {
        // ...
    }
    
    pub fn get_urls() -> Result<Vec<String>, ConfigError> {
        let config = Self::read_config()?;
        // ...
    }
}
```

**解説:**
- `self`を取らない関数は「関連関数」（他言語の静的メソッドに相当）
- `Self`は実装対象の型（ここでは`ConfigService`）を指す
- `ConfigService::read_config()`のように呼び出す
- 状態を持たないユーティリティ関数に適している

## Locoフレームワークとの統合

このサービスは、Locoフレームワークの一部として以下のように統合されます：

1. **サービス層の役割**: ビジネスロジックをコントローラーから分離
2. **モジュール構成**: `src/services/`ディレクトリにサービスを配置
3. **エラーハンドリング**: カスタムエラー型を定義し、コントローラー層で適切なHTTPレスポンスに変換

## 使用例

```rust
use crate::services::config_service::ConfigService;

// URL一覧を取得
let urls = ConfigService::get_urls()?;
println!("現在のURL: {:?}", urls);

// URL一覧を更新
let new_urls = vec![
    "https://example.com".to_string(),
    "https://api.example.com/health".to_string(),
];
ConfigService::update_urls(new_urls)?;

// 設定ファイル全体を取得
let raw_config = ConfigService::get_raw_config()?;
println!("設定ファイル:\n{}", raw_config);
```

## まとめ

このタスクで学んだRustの重要な概念：

1. **エラーハンドリング**: `thiserror`を使ったカスタムエラー型の定義
2. **Serde**: 構造体とデータ形式の相互変換
3. **Option/Resultの活用**: 安全なNull処理とエラー伝播
4. **所有権と参照**: 可変参照を使ったデータの更新
5. **関連関数**: 状態を持たないユーティリティ関数の実装

これらの概念は、Rustでのサービス層実装において基本となるパターンです。
