# 第2章: 設定ファイル操作機能の実装

## この章で学ぶこと

- サービス層とは何か
- TOML形式の理解
- カスタムエラー型の定義
- Serdeを使ったデータ変換
- ファイル操作とエラーハンドリング
- Option/Resultの実践的な使い方

---

## 2.1 サービス層とは

### MVCアーキテクチャの復習

Webアプリケーションは、一般的に以下の層に分かれています：

```
┌─────────────────┐
│   View (HTML)   │ ← ユーザが見る画面
└────────┬────────┘
         │
┌────────▼────────┐
│   Controller    │ ← リクエストを受け取り、レスポンスを返す
└────────┬────────┘
         │
┌────────▼────────┐
│    Service      │ ← ビジネスロジック（今回実装）
└────────┬────────┘
         │
┌────────▼────────┐
│     Model       │ ← データベース操作
└─────────────────┘
```

### サービス層の役割

**サービス層**は、ビジネスロジックを担当します。

**例: ConfigService**
- 設定ファイルの読み書き
- URL一覧の取得・更新
- データの検証

**メリット:**
- コントローラーがシンプルになる
- ロジックの再利用が容易
- テストしやすい

---

## 2.2 TOML形式の理解

### TOMLとは

**TOML**（Tom's Obvious, Minimal Language）は、設定ファイル用のフォーマットです。

**特徴:**
- 人間が読みやすい
- 型を持つ（文字列、数値、配列など）
- ネスト構造をサポート

### Telegraf設定ファイルの例

```toml
[global_tags]
  dc = "us-east-1"

[agent]
  interval = "10s"
  round_interval = true

[[inputs.http_response]]
  urls = [
    "https://example.com",
    "https://api.example.com/health"
  ]
  response_timeout = "5s"
  method = "GET"
```

**構造の説明:**
- `[global_tags]`: セクション（テーブル）
- `dc = "us-east-1"`: キーと値
- `[[inputs.http_response]]`: 配列テーブル（複数定義可能）
- `urls = [...]`: 配列

---

## 2.3 プロジェクト構造の準備

### サービスディレクトリの作成

```bash
mkdir -p src/services
```

### ファイル構成

```
src/
├── services/
│   ├── mod.rs              # モジュール定義
│   └── config_service.rs   # 設定ファイル操作サービス
```

---

## 2.4 カスタムエラー型の定義

### なぜカスタムエラー型が必要か

標準のエラー型（`std::io::Error`など）だけでは、エラーの種類を区別しにくいです。

**カスタムエラー型のメリット:**
- エラーの種類を明確に区別
- わかりやすいエラーメッセージ
- 型安全なエラーハンドリング

### thiserrorクレートの追加

`Cargo.toml`に追加（既に追加済み）：

```toml
[dependencies]
thiserror = "1.0"
```

### ConfigErrorの定義

`src/services/config_service.rs`を作成します。

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

### コードの詳細解説

#### thiserrorクレート
```rust
use thiserror::Error;
```

`thiserror`は、カスタムエラー型を簡単に定義できるクレートです。

#### #[derive(Debug, Error)]
```rust
#[derive(Debug, Error)]
pub enum ConfigError {
```

- `Debug`: デバッグ出力を可能にする
- `Error`: `std::error::Error`トレイトを自動実装

#### #[error("...")]属性
```rust
#[error("設定ファイルが見つかりません: {0}")]
FileNotFound(String),
```

- `#[error("...")]`: エラーメッセージを定義
- `{0}`: 列挙型の値（`String`）を埋め込む

**使用例:**
```rust
let error = ConfigError::FileNotFound("./conf/telegraf.conf".to_string());
println!("{}", error);
// 出力: 設定ファイルが見つかりません: ./conf/telegraf.conf
```

### Locoのエラー型への変換

Locoのコントローラーで使えるように、変換を実装します。

```rust
// LocoのErrorへの変換を実装
impl From<ConfigError> for loco_rs::Error {
    fn from(err: ConfigError) -> Self {
        loco_rs::Error::string(&err.to_string())
    }
}
```

**解説:**
- `From`トレイト: 型変換を定義
- `ConfigError`から`loco_rs::Error`への変換
- `?`演算子で自動変換される

---

## 2.5 データ構造の定義

### Serdeとは

**Serde**は、Rustのシリアライゼーション（データ変換）ライブラリです。

**できること:**
- Rust構造体 ↔ JSON
- Rust構造体 ↔ TOML
- Rust構造体 ↔ YAML

### TelegrafConfigの定義

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const DEFAULT_CONFIG_PATH: &str = "./conf/telegraf.conf";

// Telegraf設定ファイル全体の構造
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

### コードの詳細解説

#### #[derive(Debug, Serialize, Deserialize, Clone)]
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
```

- `Serialize`: Rust構造体 → TOML
- `Deserialize`: TOML → Rust構造体
- `Clone`: 構造体のコピーを作成

#### #[serde(skip_serializing_if = "Option::is_none")]
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub global_tags: Option<toml::Value>,
```

- `None`の場合、TOMLファイルに出力しない
- 存在しないフィールドを保持できる

#### Option<toml::Value>
```rust
pub global_tags: Option<toml::Value>,
```

- `Option`: 値があるかないかを表す
- `toml::Value`: 動的な型（構造が不明な部分を保持）

**なぜOption?**
- 設定ファイルに存在しない項目があるかもしれない
- 柔軟に対応できる

### InputsConfigの定義

```rust
// Inputs設定
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_response: Option<Vec<HttpResponseInput>>,
}
```

**解説:**
- `Vec<HttpResponseInput>`: 配列（複数のhttp_response設定）
- `[[inputs.http_response]]`は配列なので`Vec`

### HttpResponseInputの定義

```rust
// HTTP Response Input設定
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
- `urls`: 必須フィールド（`Option`なし）
- その他: 任意フィールド（`Option`あり）

---

## 2.6 ConfigServiceの実装

### 構造体の定義

```rust
pub struct ConfigService;
```

空の構造体ですが、関連関数（静的メソッド）を実装します。

### read_config()の実装

設定ファイルを読み込む関数です。

```rust
impl ConfigService {
    // 設定ファイルを読み込む
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
}
```

### コードの詳細解説

#### ファイルパスの作成
```rust
let path = Path::new(DEFAULT_CONFIG_PATH);
```

- `Path::new()`: ファイルパスを作成
- `DEFAULT_CONFIG_PATH`: `"./conf/telegraf.conf"`

#### ファイルの存在確認
```rust
if !path.exists() {
    return Err(ConfigError::FileNotFound(DEFAULT_CONFIG_PATH.to_string()));
}
```

- `path.exists()`: ファイルが存在するか確認
- 存在しない場合はエラーを返す

#### ファイルの読み込み
```rust
let content = fs::read_to_string(path)
    .map_err(|e| ConfigError::ReadError(e.to_string()))?;
```

- `fs::read_to_string()`: ファイル全体を文字列として読み込む
- `map_err()`: エラー型を変換
  - `std::io::Error` → `ConfigError::ReadError`
- `?`: エラー時は早期リターン

**map_errの仕組み:**
```rust
// 元のエラー型: Result<String, std::io::Error>
fs::read_to_string(path)
    // エラーを変換: Result<String, ConfigError>
    .map_err(|e| ConfigError::ReadError(e.to_string()))?;
```

#### TOMLのパース
```rust
let config: TelegrafConfig = toml::from_str(&content)
    .map_err(|e| ConfigError::ParseError(e.to_string()))?;
```

- `toml::from_str()`: TOML文字列を構造体に変換
- Serdeが自動的に変換してくれる

**変換の流れ:**
```
TOML文字列 → toml::from_str() → TelegrafConfig構造体
```

### get_urls()の実装

URL一覧を取得する関数です。

```rust
// URL一覧を取得
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

### コードの詳細解説

#### Selfの使用
```rust
let config = Self::read_config()?;
```

- `Self`: 実装対象の型（`ConfigService`）
- `Self::read_config()` = `ConfigService::read_config()`

#### Optionチェーン
```rust
let urls = config
    .inputs                                                    // Option<InputsConfig>
    .and_then(|inputs| inputs.http_response)                  // Option<Vec<HttpResponseInput>>
    .and_then(|http_responses| http_responses.first().cloned()) // Option<HttpResponseInput>
    .map(|http_response| http_response.urls)                  // Option<Vec<String>>
    .ok_or(ConfigError::HttpResponseSectionNotFound)?;        // Result<Vec<String>, ConfigError>
```

**各メソッドの説明:**

##### and_then()
```rust
.and_then(|inputs| inputs.http_response)
```

- `Option<T>`が`Some`の場合、クロージャを実行
- `None`の場合、`None`を返す

**例:**
```rust
let x: Option<i32> = Some(5);
let y = x.and_then(|n| Some(n * 2));  // Some(10)

let x: Option<i32> = None;
let y = x.and_then(|n| Some(n * 2));  // None
```

##### first()
```rust
.and_then(|http_responses| http_responses.first().cloned())
```

- `first()`: 配列の最初の要素を取得（`Option<&T>`）
- `cloned()`: 参照から所有権のある値に変換

##### map()
```rust
.map(|http_response| http_response.urls)
```

- `Option<T>`が`Some`の場合、値を変換
- `None`の場合、`None`を返す

##### ok_or()
```rust
.ok_or(ConfigError::HttpResponseSectionNotFound)?
```

- `Option<T>`を`Result<T, E>`に変換
- `Some(value)` → `Ok(value)`
- `None` → `Err(error)`

**なぜこの書き方?**
- Null安全性を保つ
- 各階層の存在を確認
- 簡潔に書ける

### update_urls()の実装

URL一覧を更新する関数です。

```rust
// URL一覧を更新
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

### コードの詳細解説

#### 可変変数の宣言
```rust
let mut config = Self::read_config()?;
```

- `mut`: 変更可能な変数
- 設定を読み込んで、後で変更する

#### 可変参照
```rust
if let Some(ref mut inputs) = config.inputs {
```

- `ref mut`: 可変参照を取得
- 所有権を移動せずに変更できる

**ref mutの仕組み:**
```rust
// ref mutなし（所有権が移動）
if let Some(inputs) = config.inputs {
    // config.inputsは使えなくなる
}

// ref mutあり（参照を取得）
if let Some(ref mut inputs) = config.inputs {
    // config.inputsはまだ使える
}
```

#### first_mut()
```rust
if let Some(http_response) = http_responses.first_mut() {
    http_response.urls = urls;
}
```

- `first_mut()`: 配列の最初の要素への可変参照
- `urls`を新しい値で上書き

#### TOMLへのシリアライズ
```rust
let toml_string = toml::to_string_pretty(&config)
    .map_err(|e| ConfigError::WriteError(e.to_string()))?;
```

- `toml::to_string_pretty()`: 構造体を整形されたTOML文字列に変換
- `pretty`: インデントや改行を含む読みやすい形式

#### ファイルへの書き込み
```rust
fs::write(path, toml_string)
    .map_err(|e| ConfigError::WriteError(e.to_string()))?;
```

- `fs::write()`: ファイルに書き込む
- 既存ファイルは上書きされる

### get_raw_config()の実装

設定ファイル全体を文字列として取得する関数です。

```rust
// 設定ファイル全体を取得（表示用）
pub fn get_raw_config() -> Result<String, ConfigError> {
    let path = Path::new(DEFAULT_CONFIG_PATH);
    
    if !path.exists() {
        return Err(ConfigError::FileNotFound(DEFAULT_CONFIG_PATH.to_string()));
    }
    
    fs::read_to_string(path)
        .map_err(|e| ConfigError::ReadError(e.to_string()))
}
```

**用途:**
- 設定ファイルの内容をそのまま表示
- デバッグやログ出力

---

## 2.7 モジュールのエクスポート

### src/services/mod.rsの作成

```rust
pub mod config_service;
```

これで、他のモジュールから`use crate::services::config_service::ConfigService;`として使えます。

### src/lib.rsへの追加

```rust
pub mod services;
```

---

## 2.8 動作確認

### テスト用の設定ファイル作成

`conf/telegraf.conf`を作成します。

```toml
[agent]
  interval = "10s"

[[inputs.http_response]]
  urls = [
    "https://example.com",
    "https://api.example.com/health"
  ]
  response_timeout = "5s"
```

### 簡単なテストコード

`src/main.rs`に一時的にテストコードを追加します。

```rust
use kiro_test::services::config_service::ConfigService;

fn main() {
    // URL一覧を取得
    match ConfigService::get_urls() {
        Ok(urls) => {
            println!("現在のURL:");
            for url in urls {
                println!("  - {}", url);
            }
        }
        Err(e) => {
            eprintln!("エラー: {}", e);
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
現在のURL:
  - https://example.com
  - https://api.example.com/health
```

---

## 2.9 エラーハンドリングの実践

### エラーの種類

ConfigServiceは、以下のエラーを返す可能性があります：

1. **FileNotFound**: ファイルが存在しない
2. **ReadError**: ファイルの読み込みに失敗
3. **ParseError**: TOMLのパースに失敗
4. **HttpResponseSectionNotFound**: http_responseセクションがない
5. **WriteError**: ファイルの書き込みに失敗

### エラーハンドリングの例

```rust
use kiro_test::services::config_service::{ConfigService, ConfigError};

fn example() {
    match ConfigService::get_urls() {
        Ok(urls) => {
            println!("成功: {:?}", urls);
        }
        Err(ConfigError::FileNotFound(path)) => {
            eprintln!("ファイルが見つかりません: {}", path);
            // 設定ファイルを作成する処理...
        }
        Err(ConfigError::HttpResponseSectionNotFound) => {
            eprintln!("http_responseセクションがありません");
            // デフォルト設定を使う処理...
        }
        Err(e) => {
            eprintln!("その他のエラー: {}", e);
        }
    }
}
```

---

## 2.10 まとめ

この章では、以下を学びました：

- ✅ サービス層の役割と設計
- ✅ TOML形式の理解
- ✅ thiserrorを使ったカスタムエラー型の定義
- ✅ Serdeを使ったデータ構造の定義
- ✅ ファイル操作とエラーハンドリング
- ✅ Option/Resultの実践的な使い方
- ✅ 可変参照を使ったデータの更新

**重要なポイント:**
- サービス層でビジネスロジックを分離
- カスタムエラー型で型安全なエラーハンドリング
- Serdeで構造体とデータ形式を相互変換
- Optionチェーンで安全にネストした構造を辿る

---

## 次のステップ

次の章では、**URL検証機能**を実装します。入力データのバリデーションを学びます。

[第3章: URL検証機能の実装](./chapter03_validation.md)に進みましょう！
