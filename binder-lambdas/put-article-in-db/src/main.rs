use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{
    error::DisplayErrorContext, types::AttributeValue, Client as DynamoDbClient,
};

use article_scraper::{ArticleScraper, Readability};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use reqwest::Client as ReqClient;
use url::Url;

use tracing::{info, warn};
use types::{
    ArticleLambdaRequest, ArticleLambdaResponse, ArticleRecord, ParseArticleBody, ParsedArticle,
};
use ulid::Ulid;

const PARSE_ARTICLE: &'static str = "https://api.cole.plus/parse-article";

async fn function_handler(
    event: LambdaEvent<ArticleLambdaRequest>,
) -> Result<ArticleLambdaResponse, Error> {
    info!("Starting put-article-in-db lambda");
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

    println!("parsing_response: {:#?}", parsing_response);

    let parsed_article: ParsedArticle = parsing_response.json().await?;

    // Create article struct
    let article_record = ArticleRecord {
        uild: Ulid::new().to_string(),
        title: parsed_article.title.unwrap_or("No title found".to_string()),
        author: parsed_article
            .byline
            .unwrap_or("No author found".to_string()),
        ingest_date: None,
        source_url: article_url.to_string(),
        archive_url: None,
        summary: None,
        s3_archive_arn: None,
        s3_mp3_arn: None,
    };

    // TODO(coljnr9)
    // Add Summarization with ChatGPT or likewise

    info!("ArticleRecord: {:#?}", article_record);

    let config = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;

    let db_client = DynamoDbClient::new(&config);
    let db_storage_result = db_client
        .put_item()
        .table_name("BinderArticles")
        .item("ulid", AttributeValue::S(article_record.uild))
        .item("article_url", AttributeValue::S(article_record.source_url))
        .item("title", AttributeValue::S(article_record.title))
        .item("author", AttributeValue::S(article_record.author))
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
