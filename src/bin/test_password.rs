use kiro_test::models::users::Model as User;
use sea_orm::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // データベースに接続
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://db.sqlite?mode=rwc".to_string());
    let db = Database::connect(&db_url).await?;

    // ユーザを検索
    let username = "admin";
    let password = "admin123";
    
    match User::find_by_username(&db, username).await {
        Ok(Some(user)) => {
            println!("✅ ユーザが見つかりました:");
            println!("   ID: {}", user.id);
            println!("   ユーザ名: {}", user.username);
            println!("   パスワードハッシュ: {}", user.password_hash);
            
            // パスワードを検証
            match user.verify_password(password) {
                Ok(true) => {
                    println!("✅ パスワード検証成功！");
                }
                Ok(false) => {
                    println!("❌ パスワードが一致しません");
                }
                Err(e) => {
                    println!("❌ パスワード検証エラー: {}", e);
                }
            }
        }
        Ok(None) => {
            println!("❌ ユーザが見つかりません");
        }
        Err(e) => {
            println!("❌ エラー: {}", e);
        }
    }

    Ok(())
}
