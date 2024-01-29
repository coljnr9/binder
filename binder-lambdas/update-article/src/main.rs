use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    operation::update_item::builders::UpdateItemFluentBuilder, types::AttributeValue,
    Client as DynamoDbClient,
};

use chrono::{DateTime, Local};
use lambda_http::{run, service_fn, Body, Request, RequestExt, Response};
use lambda_runtime::{Error, LambdaEvent};
use tracing::info;
use types::{ArticleStatus, ArticleStatusUpdateLambdaRequest, ArticleUpdateMethod};
use ulid::Ulid;

const BINDER_TABLE_NAME: &'static str = "BinderArticles";
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

    client
        .update_item()
        .table_name(BINDER_TABLE_NAME)
        .key("ulid", AttributeValue::S(ulid.to_string()))
        .expression_attribute_names("#S", "status")
        .update_expression("SET #S= :new_status")
        .expression_attribute_values(
            ":new_status",
            AttributeValue::S(serde_json::to_string(&new_status)?),
        )
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
        .update_expression("SET #D= :next_read_date")
        .expression_attribute_values(
            ":next_read_date",
            AttributeValue::S(serde_json::to_string(&date)?),
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
