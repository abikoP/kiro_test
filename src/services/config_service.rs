use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

// 設定ファイル操作サービス
// telegraf.confファイルの読み書きを担当

const DEFAULT_CONFIG_PATH: &str = "./conf/telegraf.conf";

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

// LocoのErrorへの変換を実装
impl From<ConfigError> for loco_rs::Error {
    fn from(err: ConfigError) -> Self {
        loco_rs::Error::string(&err.to_string())
    }
}

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

// Inputs設定
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_response: Option<Vec<HttpResponseInput>>,
}

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

pub struct ConfigService;

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
    
    // 設定ファイル全体を取得（表示用）
    pub fn get_raw_config() -> Result<String, ConfigError> {
        let path = Path::new(DEFAULT_CONFIG_PATH);
        
        if !path.exists() {
            return Err(ConfigError::FileNotFound(DEFAULT_CONFIG_PATH.to_string()));
        }
        
        fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(e.to_string()))
    }
}
