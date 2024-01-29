use chrono::{DateTime, Local};
use leptonic::prelude::*;
use leptos::{
    error::Result,
    html::{div, A},
    leptos_dom::logging::{console_log, console_warn},
    *,
};
use leptos_icons::BsIcon;
use leptos_router::*;
use types::{ArticleLambdaRequest, ArticleRecord, ArticleStatus, ArticleUpdateMethod};
use ulid::Ulid;

use url::Url;

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    mount_to_body(|| view! { <App /> })
}

#[derive(Debug, Clone, Copy)]
pub struct AppLayoutContext {
    pub library_drawer_closed: Signal<bool>,
    set_library_drawer_closed: WriteSignal<bool>,
    pub set_alert_display: WriteSignal<AlertDisplay>,
}

impl AppLayoutContext {
    pub fn close_library_drawer(&self) {
        self.set_library_drawer_closed.set(true);
    }

    pub fn toggle_library_drawer(&self) {
        console_log("Toggling drawer");
        let currently_closed = self.library_drawer_closed.get_untracked();
        console_log(&format!("currently_closed={}", currently_closed));
        self.set_library_drawer_closed.set(!currently_closed);
        console_log(&format!(
            "After setting: currently_closed={}",
            currently_closed
        ));
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Root default_theme=LeptonicTheme::default()>
            <Router>
                <nav></nav>
                <main></main>
            </Router>

            <Routes>
                <Route path="" view=Layout>
                    <Route path="" view=|| view! { <Queue/> }/>
                    <Route path="faq" view=|| view! { <Faq/> }/>
                </Route>
            </Routes>
        </Root>
    }
}

#[component]
pub fn Faq() -> impl IntoView {
    view! {
        <H1>FAQ</H1>
        <a href="https://vlad.roam.garden/How-do-I-read-things-on-the-internet">
            Workflow Inspiration
        </a>
        <ol>
            <li>"Ingest: Save something to this page!"</li>
            <li>
                "Skim: When looking for light reading ('podcast mode'), listen to the article at the top of the list using TTS"
            </li>
            <li>
                "[unclear] Study: At a dedicated time, read something 'recommended by Spaced Repetition'"
            </li>
            <li>"[optional] Record: Keep notes and highlights in some digital world, somehow."</li>
        </ol>

        <ul>
            <li>"Maybe these kinds of states for an article:"</li>
            <ol>
                <li>"Ingested/unread"</li>
                <li>"Queued for listening"</li>
                <li>"Queued for reading (next read date)"</li>
                <li>"Done - do not revisit"</li>
            </ol>
        </ul>

        <p>
            So I guess my record could look like
            <p>
                "id, Title, State, Summary, FullTextLink(s3), AudioLink(s3), Original link, Archive link"
            </p>
        </p>
        <p>
            <a href="https://aws.amazon.com/getting-started/hands-on/build-serverless-web-app-lambda-apigateway-s3-dynamodb-cognito/">
                Serverless AWS site
            </a>
        </p>
    }
}

#[derive(Clone)]
pub struct AlertDisplay {
    pub shown: bool,
    pub variant: AlertVariant,
    pub title: String,
    pub text: String,
}

#[component]
pub fn BinderAlert(
    #[prop(into)] alert_display: ReadSignal<AlertDisplay>,

    #[prop(into, optional)] id: Option<AttributeValue>,
    #[prop(into, optional)] class: Option<AttributeValue>,
    #[prop(into, optional)] style: Option<AttributeValue>,
) -> impl IntoView {
    let shown = Signal::derive(move || alert_display.get().shown);
    let variant = Signal::derive(move || alert_display.get().variant);
    let title = move || alert_display.get().title;
    let text = Signal::derive(move || alert_display.get().text);

    let ctx = use_context::<AppLayoutContext>().unwrap();

    // match variant.get() {
    //     AlertVariant::Success =>
    //     AlertVariant::Info =>
    //     AlertVariant::Warn =>
    //     AlertVariant::Danger =>
    // };

    let style = move || {
        if shown.get() {
            ""
        } else {
            "display: none;"
        }
    };

    view! {
        <div
            id=id
            class=class
            style=style
            on:click=move |_| {
                let default_alert = AlertDisplay {
                    shown: false,
                    variant: AlertVariant::Info,
                    title: "".to_owned(),
                    text: "".to_owned(),
                };
                ctx.set_alert_display.set(default_alert);
            }
        >

            <Alert variant=variant title=move || { title }.into_view()>
                {text}
            </Alert>
        </div>
    }
}

#[component]
pub fn Layout() -> impl IntoView {
    let default_alert = AlertDisplay {
        shown: false,
        variant: AlertVariant::Info,
        title: "".to_owned(),
        text: "".to_owned(),
    };

    let (library_drawer_closed, set_library_drawer_closed) = create_signal(true);
    let (get_alert_display, set_alert_display) = create_signal(default_alert.clone());

    let ctx = AppLayoutContext {
        library_drawer_closed: library_drawer_closed.into(),
        set_library_drawer_closed,
        set_alert_display,
    };

    provide_context(ctx);

    view! {
        <BinderAlert alert_display=get_alert_display id="alert"/>

        <LayoutAppBar/>
        <Outlet/>
        <LibraryDrawer/>

        // Whatever page needs to be rendered
    }
}

enum UrlValidationResult {
    Unreachable,
    Empty,
    Unparsable,
    Ok(Url),
}

fn validate_article_url(url_string: &str) -> UrlValidationResult {
    if url_string.is_empty() {
        return UrlValidationResult::Empty;
    };

    let url = match Url::parse(url_string) {
        Ok(u) => u,
        Err(_) => return UrlValidationResult::Unparsable,
    };

    UrlValidationResult::Ok(url)
}

fn submit_article_handler(ctx: AppLayoutContext, article_url_string: String) {
    let alert_display = match validate_article_url(&article_url_string) {
        UrlValidationResult::Empty => AlertDisplay {
            shown: true,
            text: "Cannot process a blank URL".to_string(),
            variant: AlertVariant::Warn,
            title: "Warning".to_string(),
        },

        UrlValidationResult::Unparsable => AlertDisplay {
            shown: true,
            text: "Unable to parse URL".to_string(),
            variant: AlertVariant::Warn,
            title: "Warning".to_string(),
        },
        UrlValidationResult::Ok(url) => {
            let save_article_action = create_action(|url: &Url| save_article(url.clone()));
            save_article_action.dispatch(url);
            AlertDisplay {
                shown: true,
                variant: AlertVariant::Success,
                title: "Success".to_string(),
                text: "Article stored successfully".to_string(),
            }
        }
        _ => AlertDisplay {
            shown: true,
            variant: AlertVariant::Danger,
            title: "Error".to_string(),
            text: "Unhandled error storing article".to_string(),
        },
    };

    ctx.set_alert_display.set(alert_display);
}

#[component]
pub fn LayoutAppBar() -> impl IntoView {
    let ctx = use_context::<AppLayoutContext>().unwrap();

    let (article_url, set_article_url) = create_signal("".to_owned());

    view! {
        <AppBar id="app-bar">
         <div class="flex-layout">
            <div class="binder-left">
            <Icon
                id="library-trigger"
                class="library-icon"
                icon=BsIcon::BsList
                on:click=move |_| ctx.toggle_library_drawer()
            />

            <H1 style="margin: 0;">"Binder"</H1>
            </div>

            <div class="search">
                    <TextInput
                        get=article_url
                        set=set_article_url
                        placeholder="Add a new article..."
                        style="padding: 10px; width: 100%;"
                    />
                    <Button on_click=move |ev| {
                        let article_url_string = article_url.get().to_string().clone();
                        submit_article_handler(ctx, article_url_string);
                        set_article_url.set("".to_string());
                    }>

                        "Add"
                    </Button>
            </div>

            <ThemeToggle off=LeptonicTheme::Light on=LeptonicTheme::Dark/>
        </div>
        </AppBar>
    }
}
async fn get_articles() -> Vec<ArticleRecord> {
    console_log("Getting articles");
    let response = match reqwasm::http::Request::get("https://api.cole.plus/articles")
        .send()
        .await
    {
        Ok(response) => {
            console_log("Got response.");
            response
        }
        Err(e) => {
            console_log(&format!("Error retrieving article data: {e}"));
            panic!()
        }
    };
    let articles: Vec<ArticleRecord> = match response.json().await {
        Ok(articles) => articles,
        Err(e) => {
            console_log(&format!("Error deserializing articles: {e}"));
            panic!();
        }
    };
    console_log(&format!("Returning articles: {:#?}", articles));

    articles
}

//  The main feed of articles to read/listen to
#[component]
pub fn Queue() -> impl IntoView {
    let articles = create_resource(|| (), |_| async move { get_articles().await });
    view! {
        <Box id="queue">
            <div style="display: flex; flex-direction: row; justify-content: center">
                <H2>Queue</H2>
            </div>

            <Stack spacing=Size::Em(
                0.5,
            )>
                {move || match articles.get() {

                    Some(mut articles) => {
                        // articles.sort_by(|a, b| {
                        //     let a_status = a.status.as_ref().unwrap_or(&ArticleStatus::New);
                        //     let b_status = b.status.as_ref().unwrap_or(&ArticleStatus::New);
                        //     a_status.partial_cmp(&b_status).unwrap_or(std::cmp::Ordering::Less)
                        // });
                        articles
                            .into_iter()
                            .map(|a| view! { <ArticleDisplay article=a/> })
                            .collect_view()
                    }
                    None => view! { <p>"Loading..."</p> }.into_view(),
                }}

            </Stack>

        </Box>
    }
}

#[component]
pub fn LibraryDrawer() -> impl IntoView {
    let ctx = use_context::<AppLayoutContext>().unwrap();
    view! {
        <Drawer
            id="library-drawer"
            shown=Signal::derive(move || !ctx.library_drawer_closed.get())
            side=DrawerSide::Left
        >
            <AppBar id="library-drawer-app-bar">
                <Icon
                    id="drawer-library-trigger"
                    class="library-icon"
                    icon=BsIcon::BsList
                    on:click=move |_| ctx.toggle_library_drawer()
                />
            </AppBar>
            <LibraryDrawerContent/>
        </Drawer>
    }
}

#[derive(Debug, Clone)]
pub struct Article {
    name: String,
    author: String,
    date: String,

    source_url: String,

    // Populated once ChatGPT has time to summarize stuff
    summary: Option<String>,
    // Populated when
    fulltext_uri: Option<String>,
    // Populated once an IA snapshot is triggered
    archive_url: Option<String>,
    // Populated once Amazon Polly pushes an MP3 to S3
    mp3_url: Option<String>,
}

#[component]
pub fn ArticleDisplay(article: ArticleRecord) -> impl IntoView {
    let ArticleRecord {
        ulid,
        title,
        author,
        source_url,
        archive_url,
        ingest_date,
        summary,
        s3_archive_arn,
        s3_mp3_arn,
        status,
        next_read_date,
    } = article;

    // Maybe create a resource that loads...?
    let article_content = create_resource(
        move || s3_archive_arn.clone(),
        |s3_archive_arn| async move {
            match s3_archive_arn {
                Some(arn) => {
                    let arn = arn.split("/").last().unwrap();
                    if arn.len() <= 3 {
                        return "No archive".to_string();
                    }
                    console_log(&format!("Fetching article fulltext for: {:?}", arn));

                    let response = match reqwasm::http::Request::get(&format!(
                        "https://api.cole.plus/article/{}",
                        arn
                    ))
                    .send()
                    .await
                    {
                        Ok(response) => {
                            console_log("Got article S3 response.");
                            response
                        }
                        Err(e) => {
                            console_log(&format!("Error retrieving article data: {e}"));
                            panic!()
                        }
                    };
                    return response.text().await.expect("Could not get text form body");
                }
                None => "No archive".to_string(),
            }
        },
    );

    let ingest_date_view = match ingest_date {
        Some(d) => d.to_rfc2822(),
        None => "Unknown".to_owned(),
    };

    let status_view = match status.clone() {
        Some(s) => format!("{:?}", s),
        None => "Unknown".to_owned(),
    };

    view! {
        <Collapsible>

            <CollapsibleHeader  slot>
            <Stack id="article-title-header" spacing=Size::Em(0.0)>
                <H3>{title}  - Status: {status_view}</H3>
                <div>{author} - Ingest: {ingest_date_view}</div>
                </Stack>
            </CollapsibleHeader>

            <CollapsibleBody slot >

            <Stack class="article-body" spacing=Size::Em(1.5)>

            <a href={source_url} rel="external">Source Url</a>

            {
                let html_text = move || match article_content.get() {
                    None => "Loading...".to_string(),
                    Some(d) => d
                };
                view! {<div inner_html=html_text/>}
            }
            <Button on_click=move |_| {
                spawn_local(requeue_article(Ulid::from_string(&ulid).expect("Invalid ULID"), status.clone()));
            }>"Finished Reading"</Button>
        </Stack>



            </CollapsibleBody>

        </Collapsible>
    }
}

const NEW_ARTICLE_ENDPOINT: &'static str = "https://api.cole.plus/article";

async fn requeue_article(article_ulid: Ulid, current_status: Option<ArticleStatus>) {
    console_log("Requeuing article");

    let next_status = match current_status {
        Some(status) => status.next_status(),
        None => ArticleStatus::New,
    };

    let next_read_date = Local::now() + next_status.repeat_duration();

    update_article_next_read_date(article_ulid, next_read_date).await;
    update_article_status(article_ulid, next_status).await;
}

async fn update_article_next_read_date(article_ulid: Ulid, next_read_date: DateTime<Local>) {
    // TODO(coljnr9) Actually hit the endpoint
    console_log(&format!(
        "Updating article {:?} with new read date {:?}",
        &article_ulid, next_read_date
    ));
    let endpoint = format!("https://api.cole.plus/article/{}", article_ulid.to_string());

    let body = ArticleUpdateMethod::NextReadDate(next_read_date);
    let response = reqwasm::http::Request::put(&endpoint)
        .body(serde_json::to_string(&body).expect("Unable to serialize new date"))
        .send()
        .await
        .expect("Error sending update request");
}

async fn update_article_status(article_ulid: Ulid, new_status: ArticleStatus) {
    // TODO(coljnr9) Actually hit the endpoint
    console_log(&format!(
        "Updating article {:?} with new status {:?}",
        &article_ulid, &new_status
    ));
    let endpoint = format!("https://api.cole.plus/article/{}", article_ulid.to_string());
    let body = ArticleUpdateMethod::Status(new_status);
    let response = reqwasm::http::Request::put(&endpoint)
        .body(serde_json::to_string(&body).expect("Unable to serialize new status"))
        .send()
        .await
        .expect("Error sending update request");
}

async fn save_article(article_url: Url) {
    console_log(&format!("Saving {}", article_url));

    let article_request = ArticleLambdaRequest {
        article_url: article_url.to_string(),
    };

    let mut request = reqwasm::http::Request::post(NEW_ARTICLE_ENDPOINT)
        .body(serde_json::to_string(&article_request).unwrap())
        .send()
        .await
        .unwrap();
}

#[component]
pub fn LibraryDrawerContent() -> impl IntoView {
    let ctx = use_context::<AppLayoutContext>().unwrap();

    let article = ArticleRecord {
        title: "Test Article".to_string(),
        author: "Cole Rogers".to_string(),
        source_url: "cole.plus".to_string(),
        summary: None,
        archive_url: None,
        ingest_date: None,
        ulid: "1".to_string(),
        s3_archive_arn: None,
        s3_mp3_arn: None,
        status: None,
        next_read_date: None,
    };

    view! {
        <Box id="library-drawer-content">
            <H3>Next Up</H3>
            <ArticleDisplay article=article.clone()/>

            <Collapsible>
                <CollapsibleHeader slot>
                    <H3>Navigation</H3>
                </CollapsibleHeader>
                <CollapsibleBody class="my-body" slot>
                    <Stack spacing=Size::Em(0.5)>
                        <Skeleton>
                            <A href="">
                                <H3>Home</H3>
                            </A>
                        </Skeleton>
                        <Skeleton>
                            <A href="faq">
                                <H3>FAQ</H3>
                            </A>
                        </Skeleton>
                    </Stack>
                </CollapsibleBody>
            </Collapsible>

            <Collapsible>
                <CollapsibleHeader slot>
                    <H3>Linked</H3>
                </CollapsibleHeader>
                <CollapsibleBody class="my-body" slot>
                    <Stack spacing=Size::Em(
                        0.5,
                    )>
                        {(0..15)
                            .map(|n| {
                                view! { <ArticleDisplay article=article.clone()/> }
                            })
                            .collect_view()}
                    </Stack>
                </CollapsibleBody>
            </Collapsible>

        </Box>
    }
}

// Source flow
