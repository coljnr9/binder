use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    operation::update_item::builders::UpdateItemFluentBuilder, types::AttributeValue,
    Client as DynamoDbClient,
};

use chrono::{format::SecondsFormat, DateTime, Local};
use lambda_http::{run, service_fn, Body, Request, RequestExt, Response};
use lambda_runtime::{Error, LambdaEvent};
use tracing::info;
use types::{ArticleStatus, ArticleStatusUpdateLambdaRequest, ArticleUpdateMethod};
use ulid::Ulid;

const BINDER_TABLE_NAME: &'static str = "BinderDb";
async fn function_handler(request: Request) -> Result<Response<String>, Error> {
    // Extract some useful information from the request
    info!("In update article function handler");

    let payload: ArticleUpdateMethod = match request.body() {
        Body::Text(t) => serde_json::from_str(t)?,
        _ => panic!("Invalid request body"),
    };

    info!("Got ArticleUpdateMethod: {:?}", payload);

    let path_params = request.path_parameters();
    let ulid_str = path_params.first("ulid").expect("No ulid specified!");
    let ulid = Ulid::from_string(ulid_str)?;

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let db_client = DynamoDbClient::new(&config);

    match payload {
        ArticleUpdateMethod::Status(status) => update_status(&db_client, ulid, &status).await?,
        ArticleUpdateMethod::NextReadDate(date) => {
            update_next_read_date(&db_client, ulid, date).await?
        }
    }

    let msg = String::new();
    let response = Response::builder()
        .status(200)
        .header("Access-Control-Allow-Headers", "Content-Type")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "OPTIONS,POST,GET")
        .body(msg)
        .unwrap();

    Ok(response)
}

async fn update_status(
    client: &DynamoDbClient,
    ulid: Ulid,
    new_status: &ArticleStatus,
) -> Result<(), Error> {
    info!("Updating article {:?} with status={:?}", ulid, new_status);

    // First get the article by ulid
    let article_item: Result<Vec<_>, _> = client
        .query()
        .table_name(BINDER_TABLE_NAME)
        .index_name("Ulid-SK-index")
        .key_condition_expression("Ulid = :ulid")
        // .expression_attribute_names(":ulid_attr", "Ulid")
        .expression_attribute_values(":ulid", AttributeValue::S(ulid.to_string()))
        .into_paginator()
        .items()
        .send()
        .collect()
        .await;

    let article_item = article_item?.pop().expect("Returned article not found");
    let pk = article_item.get("PK").expect("Unable to extract PK");
    let sk = article_item.get("SK").expect("Unable to extract SK");
    let status_str = article_item
        .get("Status")
        .expect("Unable to extract Status")
        .as_s()
        .expect("Unable to turn status to str");
    let status: ArticleStatus =
        serde_json::from_str(&status_str).expect("Unable to DE serialize status");
    let next_status = status.next_status();
    let status_value = AttributeValue::S(serde_json::to_string(&next_status)?);

    let next_read_date = Local::now() + new_status.repeat_duration();
    let new_sk = AttributeValue::S(next_read_date.to_rfc3339_opts(SecondsFormat::Millis, true));

    // Then, use the article sort key to perform an update
    info!("pk: {:#?}", &pk);
    info!("sk: {:#?}", &sk);

    client
        .put_item()
        .table_name(BINDER_TABLE_NAME)
        .item("PK", pk.clone())
        .item("SK", new_sk)
        .item(
            "Ulid",
            article_item
                .get("Ulid")
                .expect("Unable to retrieve Ulid")
                .clone(),
        )
        .item(
            "Author",
            article_item
                .get("Author")
                .expect("Unable to retrieve Author")
                .clone(),
        )
        .item(
            "IngestDate",
            article_item
                .get("IngestDate")
                .expect("Unable to retrieve IngestDate")
                .clone(),
        )
        .item(
            "S3Arn",
            article_item
                .get("S3Arn")
                .expect("Unable to retrieve S3Arn")
                .clone(),
        )
        .item("Status", status_value)
        .item(
            "Title",
            article_item
                .get("Title")
                .expect("Unable to retrieve Title")
                .clone(),
        )
        .item(
            "Url",
            article_item
                .get("Url")
                .expect("Unable to retrieve Url")
                .clone(),
        )
        .send()
        .await?;

    client
        .delete_item()
        .table_name(BINDER_TABLE_NAME)
        .key("PK", pk.clone())
        .key("SK", sk.clone())
        .send()
        .await?;

    Ok(())
}

async fn update_next_read_date(
    client: &DynamoDbClient,
    ulid: Ulid,
    date: DateTime<Local>,
) -> Result<(), Error> {
    info!("Updating article {:?} with next_read_date={:?}", ulid, date);

    client
        .update_item()
        .table_name(BINDER_TABLE_NAME)
        .key("ulid", AttributeValue::S(ulid.to_string()))
        .expression_attribute_names("#D", "next_read_date")
        .update_expression("SET #D = :next_read_date")
        .expression_attribute_values(
            ":next_read_date",
            AttributeValue::S(date.to_rfc3339_opts(SecondsFormat::Millis, true)),
        )
        .send()
        .await?;

    Ok(())
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

    run(service_fn(function_handler)).await
}
