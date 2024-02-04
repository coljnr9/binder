use chrono::offset::Local;
use chrono::{DateTime, Duration};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use ulid::Ulid;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArticleStatus {
    New,
    Repetition1,
    Repetition2,
    Repetition3,
    Repetition4,
    Archive,
}

impl ArticleStatus {
    pub fn repeat_duration(&self) -> chrono::Duration {
        match self {
            ArticleStatus::New => Duration::weeks(1),
            ArticleStatus::Repetition1 => Duration::weeks(2),
            ArticleStatus::Repetition2 => Duration::weeks(4),
            ArticleStatus::Repetition3 => Duration::weeks(12),
            ArticleStatus::Repetition4 => Duration::weeks(26),
            ArticleStatus::Archive => Duration::weeks(52),
        }
    }

    pub fn next_status(&self) -> ArticleStatus {
        match self {
            ArticleStatus::New => ArticleStatus::Repetition1,
            ArticleStatus::Repetition1 => ArticleStatus::Repetition2,
            ArticleStatus::Repetition2 => ArticleStatus::Repetition3,
            ArticleStatus::Repetition3 => ArticleStatus::Repetition4,
            ArticleStatus::Repetition4 => ArticleStatus::Archive,
            ArticleStatus::Archive => ArticleStatus::Archive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArticleUpdateMethod {
    Status(ArticleStatus),
    NextReadDate(DateTime<Local>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleRecord {
    #[serde(default)]
    pub ulid: String,

    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub archive_url: Option<String>,

    #[serde(default)]
    pub ingest_date: DateTime<Local>,

    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub s3_archive_arn: Option<String>,
    #[serde(default)]
    pub s3_mp3_arn: Option<String>,

    #[serde(default)]
    pub status: Option<ArticleStatus>,

    #[serde(default)]
    pub next_read_date: DateTime<Local>,
    // TODO(coljnr9) Metadata/tags
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleStatusUpdateLambdaRequest {
    pub ulid: Ulid,
    pub status: ArticleStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleLambdaRequest {
    pub article_url: String,
}

#[derive(Serialize)]
pub struct ArticleLambdaResponse {
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseArticleBody {
    pub article_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedArticle {
    pub title: Option<String>,
    pub byline: Option<String>,
    pub dir: Option<String>,
    pub lang: Option<String>,
    pub content: Option<String>,
    pub text_content: Option<String>,
    pub length: Option<i64>,
    pub excerpt: Option<String>,
    pub site_name: Option<String>,
}
