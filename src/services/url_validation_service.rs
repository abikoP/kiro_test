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

pub struct UrlValidationService;

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
}
