use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    error::DisplayErrorContext, types::AttributeValue, Client as DynamoDbClient,
};

use article_scraper::Readability;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use reqwest::Client as ReqClient;
use url::Url;

use types::{ArticleLambdaRequest, ArticleLambdaResponse, ArticleRecord};
use ulid::Ulid;

async fn function_handler(
    event: LambdaEvent<ArticleLambdaRequest>,
) -> Result<ArticleLambdaResponse, Error> {
    let ArticleLambdaRequest { article_url } = event.payload;

    let article_url = Url::parse(&article_url)?;
    let client = ReqClient::new();
    // let html = client.get(article_url.clone()).send().await?.text().await?;

    // Create article struct
    let article = ArticleRecord {
        uild: Ulid::new(),
        source_url: article_url.clone(),
        archive_url: None,
        summary: None,
        s3_archive_arn: None,
        s3_mp3_arn: None,
    };

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;

    let db_client = DynamoDbClient::new(&config);
    let db_storage_result = db_client
        .put_item()
        .table_name("BinderArticles")
        .item("ulid", AttributeValue::S(article.uild.to_string()))
        .item(
            "article_url",
            AttributeValue::S(article.source_url.to_string()),
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
