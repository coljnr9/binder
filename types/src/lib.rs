use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleRecord {
    pub uild: String,

    pub title: String,
    pub author: String,
    pub source_url: String,
    pub archive_url: Option<String>,

    pub summary: Option<String>,
    pub s3_archive_arn: Option<String>,
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
