use kiro_test::models::users::Model as User;
use sea_orm::{Database, DatabaseConnection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // データベースに接続
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://db.sqlite?mode=rwc".to_string());
    let db: DatabaseConnection = Database::connect(&db_url).await?;

    // テスト用ユーザを作成
    let username = "admin";
    let password = "admin123";

    // 既存のユーザをチェック
    match User::find_by_username(&db, username).await {
        Ok(Some(user)) => {
            println!("ℹ️  テストユーザ '{}' は既に存在します (ID: {})", username, user.id);
            println!("   パスワード: {}", password);
        }
        Ok(None) => {
            // ユーザを作成
            match User::create_with_password(&db, username, password).await {
                Ok(user) => {
                    println!("✅ テストユーザを作成しました:");
                    println!("   ユーザ名: {}", user.username);
                    println!("   パスワード: {}", password);
                    println!("   ID: {}", user.id);
                }
                Err(e) => {
                    println!("❌ ユーザの作成に失敗しました: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ ユーザの検索に失敗しました: {}", e);
        }
    }

    Ok(())
}
