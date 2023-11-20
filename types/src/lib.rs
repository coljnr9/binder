use serde::{Deserialize, Serialize};
use ulid::Ulid;
use url::Url;

#[derive(Debug)]
pub struct ArticleRecord {
    pub uild: Ulid,

    pub source_url: Url,
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
