use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    operation::update_item::builders::UpdateItemFluentBuilder, types::AttributeValue,
    Client as DynamoDbClient,
};

use lambda_http::{run, service_fn, Body, Request, RequestExt, Response};
use lambda_runtime::{Error, LambdaEvent};
use tracing::info;
use types::{ArticleStatus, ArticleStatusUpdateLambdaRequest};
use ulid::Ulid;

const BINDER_TABLE_NAME: &'static str = "BinderArticles";
async fn function_handler(request: Request) -> Result<String, Error> {
    // Extract some useful information from the request

    let status: ArticleStatus = match request.body() {
        Body::Text(t) => serde_json::from_str(t)?,
        _ => panic!("Invalid request body"),
    };
    let path_params = request.path_parameters();
    let ulid_str = path_params.first("ulid").expect("No ulid specified!");
    let ulid = Ulid::from_string(ulid_str)?;

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    let db_client = DynamoDbClient::new(&config);

    update_status(&db_client, ulid, &status).await?;

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = String::new();
    Ok(resp)
}

async fn update_status(
    client: &DynamoDbClient,
    ulid: Ulid,
    new_status: &ArticleStatus,
) -> Result<(), Error> {
    info!("Updating article {:?} with status {:?}", ulid, new_status);

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
