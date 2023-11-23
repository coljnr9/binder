use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client as DynamoDbClient};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use types::ArticleRecord;

#[derive(Deserialize)]
struct Request {}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: LambdaEvent<Request>) -> Result<Vec<ArticleRecord>, Error> {
    let title_not_found: AttributeValue =
        aws_sdk_dynamodb::types::AttributeValue::S(String::from("TITLE NOT FOUND"));
    println!("Getting articles");
    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let db_client = DynamoDbClient::new(&config);

    let page_size = 10;
    let table_name = "BinderArticles";

    let items: Result<Vec<_>, _> = db_client
        .scan()
        .table_name(table_name)
        .limit(page_size)
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;

    let mut resp = Vec::new();
let content_s3_arn = *item.get("article_url").unwrap().as_s().unwrap().clone()
    for item in items? {
        println!("{:?}", item);
        let article_record = ArticleRecord {
            ulid: (*item.get("ulid").unwrap().as_s().unwrap()).clone(),
            source_url: (*item.get("article_url").unwrap().as_s().unwrap()).clone(),
            title: (*item
                .get("title")
                .unwrap_or(&title_not_found)
                .as_s()
                .unwrap())
            .clone(),
            author: (*item
                .get("author")
                .unwrap_or(&aws_sdk_dynamodb::types::AttributeValue::S(
                    "AUTHOR NOT FOUND".to_string(),
                ))
                .as_s()
                .unwrap())
            .clone(),
            ingest_date: Some("".to_string()),
            archive_url: Some("".to_string()),
            summary: Some("".to_string()),
            s3_archive_arn: Some("".to_string()),
            s3_mp3_arn: Some("".to_string()),
        };
        resp.push(article_record);
    }
    println!("{:?}", &resp);
    Ok(resp)
}
