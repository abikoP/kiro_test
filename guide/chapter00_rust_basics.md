# 第0章: Rust基礎

## この章について

この章では、本テキストを進めるために必要なRustの基礎知識を学びます。すでにRustの基本を知っている方は、復習として読むか、必要に応じて参照してください。

Rustの全てを網羅するのではなく、**このテキストで使う機能に絞って**説明します。より詳しく学びたい方は、[The Rust Programming Language](https://doc.rust-lang.org/book/)（通称「The Book」）を参照してください。

---

## 0.1 変数と型

### 変数の宣言

Rustでは`let`キーワードを使って変数を宣言します。

```rust
let x = 5;
let name = "Alice";
```

### 不変性（Immutability）

Rustの変数は**デフォルトで不変**です。一度値を代入したら、変更できません。

```rust
let x = 5;
x = 6;  // ❌ エラー！xは不変
```

変更可能にするには、`mut`キーワードを使います。

```rust
let mut x = 5;
x = 6;  // ✅ OK！
```

### 基本的な型

```rust
// 整数型
let age: i32 = 25;           // 32ビット符号付き整数
let count: usize = 10;       // アーキテクチャ依存のサイズ

// 浮動小数点型
let price: f64 = 19.99;      // 64ビット浮動小数点数

// 真偽値型
let is_active: bool = true;

// 文字列型
let name: String = String::from("Alice");  // 所有権を持つ文字列
let greeting: &str = "Hello";              // 文字列スライス（参照）
```

### 型推論

Rustは型を推論できるので、多くの場合、型注釈は不要です。

```rust
let x = 5;        // i32と推論される
let name = "Bob"; // &strと推論される
```

---

## 0.2 関数

### 関数の定義

```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn add(a: i32, b: i32) -> i32 {
    a + b  // returnキーワードは不要（最後の式が戻り値）
}
```

### ポイント
- 関数名は`snake_case`（小文字とアンダースコア）
- 引数には型注釈が必須
- 戻り値の型は`->`の後に指定
- 最後の式（セミコロンなし）が戻り値

### 使用例

```rust
fn main() {
    greet("Alice");
    let sum = add(3, 5);
    println!("Sum: {}", sum);  // Sum: 8
}
```

---

## 0.3 構造体とメソッド

### 構造体の定義

```rust
struct User {
    username: String,
    email: String,
    age: u32,
}
```

### インスタンスの作成

```rust
let user = User {
    username: String::from("alice"),
    email: String::from("alice@example.com"),
    age: 25,
};

println!("Username: {}", user.username);
```

### メソッドの定義

`impl`ブロック内でメソッドを定義します。

```rust
impl User {
    // 関連関数（コンストラクタのようなもの）
    fn new(username: String, email: String, age: u32) -> Self {
        User {
            username,
            email,
            age,
        }
    }
    
    // メソッド（selfを受け取る）
    fn greet(&self) {
        println!("Hello, I'm {}!", self.username);
    }
    
    // 可変メソッド
    fn celebrate_birthday(&mut self) {
        self.age += 1;
    }
}
```

### 使用例

```rust
let mut user = User::new(
    String::from("alice"),
    String::from("alice@example.com"),
    25
);

user.greet();              // Hello, I'm alice!
user.celebrate_birthday();
println!("Age: {}", user.age);  // Age: 26
```

---

## 0.4 エラーハンドリング（Result型）

### Result型とは

Rustでは、エラーが発生する可能性のある操作に`Result`型を使います。

```rust
enum Result<T, E> {
    Ok(T),   // 成功時の値
    Err(E),  // エラー時の値
}
```

### Result型の使用例

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("ゼロで割ることはできません"))
    } else {
        Ok(a / b)
    }
}
```

### Result型の処理

#### 1. matchを使う方法

```rust
match divide(10.0, 2.0) {
    Ok(result) => println!("結果: {}", result),
    Err(error) => println!("エラー: {}", error),
}
```

#### 2. unwrapを使う方法（開発時のみ推奨）

```rust
let result = divide(10.0, 2.0).unwrap();  // エラー時はパニック
```

#### 3. ?演算子を使う方法（推奨）

```rust
fn calculate() -> Result<f64, String> {
    let result = divide(10.0, 2.0)?;  // エラー時は早期リターン
    Ok(result * 2.0)
}
```

### 本テキストでの使用例

```rust
// ConfigServiceの例
pub fn get_urls() -> Result<Vec<String>, ConfigError> {
    let config = Self::read_config()?;  // エラー時は早期リターン
    
    let urls = config
        .inputs
        .and_then(|inputs| inputs.http_response)
        .and_then(|http_responses| http_responses.first().cloned())
        .map(|http_response| http_response.urls)
        .ok_or(ConfigError::HttpResponseSectionNotFound)?;
    
    Ok(urls)
}
```

---

## 0.5 所有権の基本

### 所有権とは

Rustの最も重要な特徴の1つが**所有権システム**です。メモリ安全性をコンパイル時に保証します。

### 所有権の3つのルール

1. **各値には所有者がいる**
2. **所有者は常に1つだけ**
3. **所有者がスコープを抜けると、値は破棄される**

### 所有権の移動（ムーブ）

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1の所有権がs2に移動

// println!("{}", s1);  // ❌ エラー！s1はもう使えない
println!("{}", s2);     // ✅ OK
```

### 借用（Borrowing）

所有権を移動せずに、値を参照できます。

```rust
fn print_length(s: &String) {  // &は参照（借用）
    println!("Length: {}", s.len());
}

let s = String::from("hello");
print_length(&s);  // 所有権は移動しない
println!("{}", s); // ✅ sはまだ使える
```

### 可変借用

```rust
fn add_exclamation(s: &mut String) {
    s.push_str("!");
}

let mut s = String::from("hello");
add_exclamation(&mut s);
println!("{}", s);  // hello!
```

### 本テキストでの使用例

```rust
// 不変借用の例
pub async fn show(State(_ctx): State<AppContext>) -> Result<impl IntoResponse> {
    let urls = ConfigService::get_urls()?;  // 所有権を取得
    // urlsを使ってHTMLを生成...
    Ok(Html(html))
}

// 可変借用の例
pub fn update_urls(urls: Vec<String>) -> Result<(), ConfigError> {
    let mut config = Self::read_config()?;
    // configを変更...
    Ok(())
}
```

---

## 0.6 モジュールシステム

### モジュールとは

Rustでは、コードを**モジュール**に分割して整理します。

### ファイル構成

```
src/
├── main.rs
├── lib.rs
├── controllers/
│   ├── mod.rs
│   ├── config.rs
│   └── admin.rs
└── services/
    ├── mod.rs
    └── config_service.rs
```

### mod.rsでのエクスポート

```rust
// src/controllers/mod.rs
pub mod config;
pub mod admin;
```

### 他のモジュールの使用

```rust
// src/main.rs
use crate::controllers::config;
use crate::services::config_service::ConfigService;

// または
use crate::controllers::*;
```

### 公開と非公開

- `pub`: 外部から使用可能
- `pub(crate)`: クレート内でのみ使用可能
- 何もつけない: モジュール内でのみ使用可能

```rust
pub struct User {
    pub username: String,      // 公開
    email: String,             // 非公開
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        User { username, email }
    }
    
    fn validate_email(&self) -> bool {  // 非公開メソッド
        self.email.contains('@')
    }
}
```

---

## 0.7 よく使うマクロ

### println!

```rust
println!("Hello, world!");
println!("x = {}", x);
println!("x = {}, y = {}", x, y);
```

### format!

```rust
let s = format!("Hello, {}!", name);
```

### vec!

```rust
let numbers = vec![1, 2, 3, 4, 5];
```

---

## 0.8 コレクション

### Vec（ベクタ）

動的配列です。

```rust
let mut numbers = Vec::new();
numbers.push(1);
numbers.push(2);

// または
let numbers = vec![1, 2, 3];

// イテレーション
for num in &numbers {
    println!("{}", num);
}
```

### String

```rust
let mut s = String::new();
s.push_str("hello");
s.push(' ');
s.push_str("world");

println!("{}", s);  // hello world
```

---

## 0.9 Option型

値が存在するかしないかを表す型です。

```rust
enum Option<T> {
    Some(T),  // 値がある
    None,     // 値がない
}
```

### 使用例

```rust
fn find_user(id: u32) -> Option<User> {
    if id == 1 {
        Some(User::new(String::from("alice"), String::from("alice@example.com"), 25))
    } else {
        None
    }
}

match find_user(1) {
    Some(user) => println!("Found: {}", user.username),
    None => println!("User not found"),
}
```

### 便利なメソッド

```rust
let user = find_user(1);

// unwrap_or: Noneの場合はデフォルト値
let username = user.map(|u| u.username).unwrap_or(String::from("guest"));

// and_then: Someの場合のみ処理を続ける
let email = find_user(1)
    .and_then(|u| Some(u.email));
```

---

## 0.10 非同期プログラミングの基礎

### async/await

Locoでは非同期プログラミングを使います。

```rust
// 非同期関数の定義
async fn fetch_data() -> Result<String, Error> {
    // 非同期処理...
    Ok(String::from("data"))
}

// 非同期関数の呼び出し
async fn process() {
    let data = fetch_data().await;  // .awaitで待機
}
```

### 本テキストでの使用例

```rust
pub async fn show(State(ctx): State<AppContext>) -> Result<impl IntoResponse> {
    // データベースアクセスなどの非同期処理
    let urls = ConfigService::get_urls()?;
    Ok(Html(html))
}
```

---

## 0.11 トレイト（Trait）

### トレイトとは

トレイトは、型が実装すべきメソッドを定義します（他の言語のインターフェースに似ています）。

```rust
trait Greet {
    fn greet(&self) -> String;
}

struct User {
    name: String,
}

impl Greet for User {
    fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name)
    }
}
```

### 派生トレイト

よく使うトレイトは`#[derive]`で自動実装できます。

```rust
#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
}

let user = User {
    name: String::from("Alice"),
    age: 25,
};

println!("{:?}", user);  // Debug: User { name: "Alice", age: 25 }
let user2 = user.clone(); // Clone: userのコピーを作成
```

### 本テキストでよく使うトレイト

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelegrafConfig {
    pub global_tags: Option<toml::Value>,
    pub agent: Option<toml::Value>,
}
```

---

## 0.12 まとめ

この章では、以下のRustの基礎を学びました：

- ✅ 変数と型
- ✅ 関数
- ✅ 構造体とメソッド
- ✅ エラーハンドリング（Result型）
- ✅ 所有権の基本
- ✅ モジュールシステム
- ✅ よく使うマクロ
- ✅ コレクション
- ✅ Option型
- ✅ 非同期プログラミングの基礎
- ✅ トレイト

これらの知識があれば、本テキストを進めることができます。

わからない部分があっても心配しないでください。実際にコードを書きながら、徐々に理解が深まっていきます。

---

## 次のステップ

準備ができたら、[第1章: Locoプロジェクトのセットアップ](./chapter01_loco_setup.md)に進みましょう！

実際にプロジェクトを作りながら、Rustの知識を実践していきます。
