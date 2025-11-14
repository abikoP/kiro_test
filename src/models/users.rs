use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub use super::_entities::users::{ActiveModel, Column, Entity, Model};

impl Model {
    /// パスワードをハッシュ化してユーザを作成
    pub async fn create_with_password(
        db: &DatabaseConnection,
        username: &str,
        password: &str,
    ) -> Result<Self> {
        let password_hash = hash_password(password)?;
        
        let user = ActiveModel {
            username: Set(username.to_string()),
            password_hash: Set(password_hash),
            ..Default::default()
        };
        
        let user = user.insert(db).await?;
        Ok(user)
    }
    
    /// ユーザ名でユーザを検索
    pub async fn find_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<Self>> {
        let user = Entity::find()
            .filter(Column::Username.eq(username))
            .one(db)
            .await?;
        Ok(user)
    }
    
    /// パスワードを検証
    pub fn verify_password(&self, password: &str) -> Result<bool> {
        verify_password(password, &self.password_hash)
    }
}

/// パスワードをbcryptでハッシュ化
fn hash_password(password: &str) -> Result<String> {
    use bcrypt::{hash, DEFAULT_COST};
    hash(password, DEFAULT_COST).map_err(|e| Error::string(&format!("パスワードのハッシュ化に失敗しました: {}", e)))
}

/// パスワードを検証
fn verify_password(password: &str, hash: &str) -> Result<bool> {
    use bcrypt::verify;
    verify(password, hash).map_err(|e| Error::string(&format!("パスワードの検証に失敗しました: {}", e)))
}
