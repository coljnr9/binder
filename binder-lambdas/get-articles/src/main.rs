use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client as DynamoDbClient};
use chrono::{DateTime, Duration, FixedOffset, Local};
use lambda_http::{run, service_fn, Body, Request, RequestExt, Response};
use lambda_runtime::Error;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use types::ArticleRecord;

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
    run(func).await?;
    Ok(())
}

const TABLE_NAME: &'static str = "BinderDb";

pub(crate) async fn my_handler(request: Request) -> Result<Response<String>, Error> {
    let title_not_found: AttributeValue =
        aws_sdk_dynamodb::types::AttributeValue::S(String::from("TITLE NOT FOUND"));
    println!("Getting articles!!");
    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let db_client = DynamoDbClient::new(&config);

    let page_size = 10;

    let query_string_params = request.query_string_parameters();
    let start_param = query_string_params.first("start");
    let end_param = query_string_params.first("end");

    info!("start_param: {:?}, end_param: {:?}", start_param, end_param);

    let mut db_request = db_client.query().table_name(TABLE_NAME);

    db_request = match (start_param, end_param) {
        (Some(s), Some(e)) => db_request
            .key_condition_expression("PK = :articles AND SK BETWEEN :start AND :end")
            .expression_attribute_values(":start", AttributeValue::S(s.to_string()))
            .expression_attribute_values(":end", AttributeValue::S(e.to_string())),
        (Some(s), None) => db_request
            .key_condition_expression("PK = :articles AND SK >= :start")
            .expression_attribute_values(":start", AttributeValue::S(s.to_string())),
        (None, Some(e)) => db_request
            .key_condition_expression("PK = :articles AND SK <= :end")
            .expression_attribute_values(":end", AttributeValue::S(e.to_string())),
        (None, None) => db_request.key_condition_expression("PK = :articles"),
    };

    db_request = db_request
        .expression_attribute_values(":articles", AttributeValue::S("Articles".to_string()));

    let key_condition_expression = db_request.get_key_condition_expression();
    info!("Key condition expression: {:?}", key_condition_expression);

    let items: Result<Vec<_>, _> = db_request
        .select(aws_sdk_dynamodb::types::Select::AllAttributes)
        .limit(page_size)
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;

    info!("Items -> {:?}", items);

    let mut article_records = Vec::new();
    for item in items? {
        let db_arn_item = item.get("S3Arn").clone();
        let content_s3_arn = match db_arn_item {
            Some(data) => data
                .as_s()
                .expect("Could not turn arn field into a string")
                .clone(),
            None => "".to_string(),
        };

        println!("{:?}", item);
        let status = match item.get("Status") {
            Some(s) => {
                let s = s.as_s().expect("Unable to create string value");
                let v = serde_json::from_str(s).expect("Unable to deserialize status");
                Some(v)
            }
            None => None,
        };

        let ingest_date = match item.get("IngestDate") {
            Some(s) => {
                let s = s.as_s().expect("Unable to create string value");
                let v = serde_json::from_str(s).expect("Unable to deserialize ingest_date");
                v
            }
            None => panic!("No IngestDate available in article read"),
        };

        let next_read_date = match item.get("SK") {
            Some(s) => {
                let s = s.as_s().expect("Unable to create string value");
                let v = DateTime::parse_from_rfc3339(s)
                    .expect(&format!("Unable to parse ingest_date from: {}", &s))
                    .with_timezone(&Local);
                v
            }
            None => panic!("Error handling Sort Key (SK)"),
        };
        let article_record = ArticleRecord {
            ulid: (*item.get("Ulid").unwrap().as_s().unwrap()).clone(),
            source_url: (*item.get("Url").unwrap().as_s().unwrap()).clone(),
            title: (*item
                .get("Title")
                .unwrap_or(&title_not_found)
                .as_s()
                .unwrap())
            .clone(),
            author: (*item
                .get("Author")
                .unwrap_or(&aws_sdk_dynamodb::types::AttributeValue::S(
                    "AUTHOR NOT FOUND".to_string(),
                ))
                .as_s()
                .unwrap())
            .clone(),
            ingest_date,
            archive_url: Some(content_s3_arn.to_string()),
            summary: Some("".to_string()),
            s3_archive_arn: Some(content_s3_arn),
            s3_mp3_arn: Some("".to_string()),
            status,
            next_read_date,
        };
        article_records.push(article_record);
    }

    let response = Response::builder()
        .status(200)
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "OPTIONS,POST,GET")
        .body(serde_json::to_string(&article_records)?)
        .unwrap();

    Ok(response)
}
