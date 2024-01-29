use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    error::DisplayErrorContext, types::AttributeValue, Client as DynamoDbClient,
};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{config::Region, meta::PKG_VERSION, Client as S3Client, Error as S3Error};
use aws_types::sdk_config::SdkConfig;
use bytes::Bytes;

use chrono::{offset::Local, DateTime};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use reqwest::Client as ReqClient;
use url::Url;

use tracing::{info, warn};
use types::{
    ArticleLambdaRequest, ArticleLambdaResponse, ArticleRecord, ArticleStatus, ParseArticleBody,
    ParsedArticle,
};
use ulid::Ulid;

const PARSE_ARTICLE: &'static str = "https://api.cole.plus/parse-article";
const BINDER_CONTENT_BUCKET: &'static str = "binder-content";

async fn upload_to_s3(
    client: &S3Client,
    bucket: &str,
    key: &str,
    data: &String,
) -> Result<(), S3Error> {
    info!("Upload to S3");
    let test_content = ByteStream::from(data.clone().into_bytes());
    let resp = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(test_content)
        .send()
        .await?;

    info!("Upload success (resp: {:#?} key = {})!", &resp, &key);

    Ok(())
}

async fn function_handler(
    event: LambdaEvent<ArticleLambdaRequest>,
) -> Result<ArticleLambdaResponse, Error> {
    info!("Starting put-article-in-db lambda");
    let ulid = Ulid::new().to_string();
    let content_object_name = format!("{}-content", &ulid);
    let content_s3_arn = format!("{}/{}", BINDER_CONTENT_BUCKET, &content_object_name);

    let mp3_object_name = format!("{}-mp3", &ulid);
    let mp3_s3_arn = format!("{}/{}", BINDER_CONTENT_BUCKET, &mp3_object_name);

    let ArticleLambdaRequest { article_url } = event.payload;

    let parsed_article_url = Url::parse(&article_url)?;
    let client = ReqClient::new();

    let body = ParseArticleBody {
        article_url: article_url.clone(),
    };

    println!("ParsedArticleBody: {:?}", &body);

    let parsing_response = client
        .post(Url::parse(PARSE_ARTICLE)?)
        .json(&body)
        .send()
        .await?;

    let parsed_article: ParsedArticle = parsing_response.json().await?;
    // Create article struct
    let ingest_date = Local::now();
    let article_status = ArticleStatus::New;

    let article_record = ArticleRecord {
        ulid,
        title: parsed_article.title.unwrap_or("No title found".to_string()),
        author: parsed_article
            .byline
            .unwrap_or("No author found".to_string()),
        source_url: article_url.to_string(),
        archive_url: None,
        summary: None,
        s3_archive_arn: Some(content_s3_arn),
        s3_mp3_arn: None,
        ingest_date: Some(ingest_date),
        status: Some(article_status.clone()),
        next_read_date: None,
    };

    info!("Building s3 client");
    let shared_config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let s3_client = S3Client::new(&shared_config);

    info!("Got s3_client, starting upload");
    upload_to_s3(
        &s3_client,
        &BINDER_CONTENT_BUCKET,
        &content_object_name,
        &parsed_article
            .content
            .unwrap_or("No parsed content".to_string()),
    )
    .await?;

    // TODO(coljnr9)
    // Add Summarization with ChatGPT or likewise

    info!("ArticleRecord: {:#?}", article_record);

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let db_client = DynamoDbClient::new(&config);
    let db_storage_result = db_client
        .put_item()
        .table_name("BinderArticles")
        .item("ulid", AttributeValue::S(article_record.ulid))
        .item("article_url", AttributeValue::S(article_record.source_url))
        .item("title", AttributeValue::S(article_record.title))
        .item("author", AttributeValue::S(article_record.author))
        .item(
            "ingest_date",
            AttributeValue::S(serde_json::to_string(&ingest_date)?),
        )
        .item(
            "status",
            AttributeValue::S(serde_json::to_string(&article_status)?),
        )
        .item(
            "s3_archive_arn",
            AttributeValue::S(
                article_record
                    .s3_archive_arn
                    .unwrap_or("No s3 archive".to_string()),
            ),
        )
        .item(
            "next_read_date",
            AttributeValue::S(serde_json::to_string(&article_record.next_read_date)?),
        )
        .send()
        .await;

    println!("Storage result -> {:?}", db_storage_result);
    let message = format!("Successfully stored {article_url} into DB");

    let resp = ArticleLambdaResponse { message };

    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    lambda_runtime::run(service_fn(function_handler)).await
}

fn base_url(mut url: Url) -> Url {
    match url.clone().path_segments_mut() {
        Ok(mut path) => {
            path.clear();
        }
        Err(_) => {
            warn!("Could not get base url");
            return url;
        }
    }

    url.set_query(None);

    url
}
