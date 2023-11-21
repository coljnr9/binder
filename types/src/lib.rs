use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleRecord {
    #[serde(default)]
    pub uild: String,

    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub archive_url: Option<String>,

    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub s3_archive_arn: Option<String>,
    #[serde(default)]
    pub s3_mp3_arn: Option<String>, // TODO
                                    // next_repetition_date
                                    // ingest_timestamp
                                    // tags/metadata
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleLambdaRequest {
    pub article_url: String,
}

#[derive(Serialize)]
pub struct ArticleLambdaResponse {
    pub message: String,
}
