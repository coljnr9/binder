use chrono::{DateTime, Duration, Local};
use leptonic::{components::prelude::*, prelude::*};
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
    mount_to_body(|| view! { <App/> })
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
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Root default_theme=LeptonicTheme::Dark>
            <Router trailing_slash=TrailingSlash::Redirect>
                <nav></nav>
                <main></main>

                <Routes>
                    <Route path="/" view=Layout>
                        <Route path="/" view=|| view! { <ReadingList/> }/>

                        <Route path="/faq/" view=|| view! { <Faq/> }/>

                        <Route path="/archive/" view=|| view! { <ArticleArchive/> }>
                        </Route>

                        <Route path="/authors/" view=|| view! { <Outlet/> }>
                            <Route path=":author" view=|| view! { <AuthorAnthology/> }/>
                            <Route path="" view=|| view! { <H1>List of Authors</H1>}/>
                        </Route>
                    </Route>
                </Routes>
            </Router>
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

            <Alert variant=variant.get()>
                <AlertTitle slot>{title}</AlertTitle>
                <AlertContent slot>{text}</AlertContent>
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
                        icon=icondata::BsList
                        on:click=move |_| ctx.toggle_library_drawer()
                    />

                    <H1>"Binder"</H1>
                </div>

                <div class="search">
                    <TextInput
                        get=article_url
                        set=set_article_url
                        placeholder="Add a new article..."
                        style="padding: 10px; width: 100%;"
                    />
                    <Button on_press=move |ev| {
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

async fn get_articles_by_next_read_date(
    start_date: Option<DateTime<Local>>,
    end_date: Option<DateTime<Local>>,
) -> Vec<ArticleRecord> {
    console_log("Getting articles in date range");
    let mut url_str = "https://api.cole.plus/articles".to_string();
    match (start_date, end_date) {
        (Some(s), Some(e)) => url_str.push_str(&format!(
            "?start={}&end={}",
            s.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            e.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        )),
        (Some(s), None) => url_str.push_str(&format!(
            "?start={}",
            s.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        )),
        (None, Some(e)) => url_str.push_str(&format!(
            "?end={}",
            e.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        )),
        (None, None) => {}
    };
    let response = match reqwasm::http::Request::get(&url_str).send().await {
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

#[component]
pub fn LargeReadingListDisplay(mut articles: Vec<ArticleRecord>) -> impl IntoView {
    let preview_articles = articles[..3]
        .iter()
        .map(|article| (article.ulid.clone(), article.clone()))
        .collect::<Vec<_>>();

    let (visible_articles, set_visible_articles) = create_signal(preview_articles);
    let (display_all, set_display_all) = create_signal(false);
    let (button_text, set_button_text) = create_signal("Show more...");

    let update_article_view = move |_| {
        let articles = articles.iter();

        set_display_all.update(move |value| *value = !*value);

        if display_all.get() {
            set_visible_articles.update(move |visible_articles| {
                for article in articles.skip(3) {
                    visible_articles.push((article.ulid.clone(), article.clone()))
                }
            });
            set_button_text.update(move |text| *text = "Show less...");
        } else {
            set_visible_articles.update(move |visible_articles| {
                visible_articles.clear();
                for article in articles.take(3) {
                    visible_articles.push((article.ulid.clone(), article.clone()))
                }
            });
            set_button_text.update(move |text| *text = "Show more...");
        }
    };

    view! {
    <H2>Next Up</H2>
    <Collapsibles default_on_open=OnOpen::CloseOthers>
        <Stack spacing=Size::Em(0.5) style="min-width: 50%">
            <For
                each=move || visible_articles.get()
                key=|a| a.0.clone()
                    children=move |(_, article)| {
                        view! { <ArticleDisplay article=article.clone()/> }
                    }
                />

            </Stack>
        </Collapsibles>
        <div style="position: sticky; bottom: 10px; margin: 10px; display: flex; flex-direction: row; justify-content: flex-end;">
            <Button
                on_press=update_article_view id="preview-toggle-button"
            >
                {move || button_text.get()}
            </Button>
        </div>
    }
}

#[component]
pub fn ArticleArchive() -> impl IntoView {
    let location = use_location();
    let hash = location.hash.get();
    view! {
        <Box id="archive">
            <Await
                future=||get_articles_by_next_read_date(None, None)
                let:articles
            >
            <H1>Article Archive</H1>
            {
                let articles = articles.clone();
                let v = view! {
                    <Stack spacing=Size::Em(0.5) style="min-width: 50%">
                        {
                            articles.into_iter()
                            .map(|a| view! { <ArticleDisplayImmutable article=a.clone()/> })
                            .collect_view()
                        }
                    </Stack>

                };
                v

            }

            </Await>
        </Box>
    }
}
#[component]
pub fn ArticleArchiveOld() -> impl IntoView {
    let articles = create_resource(
        move || (),
        move |_| async move { get_articles_by_next_read_date(None, None).await },
    );
    view! {
        <Box id="queue">
            {move || match articles.get() {
                None => {
                    view! {
                        <Stack spacing=Size::Em(0.5)>
                            <H1>Article Archive</H1>
                            <Skeleton height=Size::Em(15.0)>Loading articles...</Skeleton>
                        </Stack>
                    }
                }
                Some(v) => {
                    let v = view! {
                        <Stack spacing=Size::Em(0.5) style="min-width: 50%">
                            <H1>Article Archive</H1>
                            {v
                                .into_iter()
                                .map(|a| view! { <ArticleDisplay article=a.clone()/> })
                                .collect_view()}

                        </Stack>
                    };
                    v
                }
            }}

        </Box>
    }
}

//  The main feed of articles to read/listen to
#[component]
pub fn ReadingList() -> impl IntoView {
    let article_read_date_start = None;
    let article_read_date_end = Some(Local::now() + Duration::weeks(1));
    let articles = create_resource(
        move || (article_read_date_start, article_read_date_end),
        move |_| async move {
            get_articles_by_next_read_date(article_read_date_start, article_read_date_end).await
        },
    );

    // 1. Loading
    // 2. No articles
    // 2. 1-3 Articles
    // 3. 4+ Articles

    view! {
        <Box id="queue">
            {move || match articles.get() {
                None => view! { <Skeleton height=Size::Em(5.0)/> },
                Some(v) if v.is_empty() => view! { <H3>Reading list empty</H3> },
                Some(v) if v.len() < 3 => view! { <H3>There are few articles</H3> },
                Some(v) => view! { <LargeReadingListDisplay articles=v/> },
            }}

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
                    icon=icondata::BsList
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
pub fn AuthorAnthology() -> impl IntoView {
    let params = use_params_map();
    let author_name = move || {
        params.with(|params| {
            params
                .get("author")
                .cloned()
                .unwrap_or("Invalid author param".to_string())
        })
    };
    view! { <Box id="author-anthology">

        <H1>Works by {author_name}</H1>
    </Box>
    }
}

#[component]
pub fn ArticleDisplayImmutable(article: ArticleRecord) -> impl IntoView {
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

    let ingest_date_view = ingest_date.to_rfc2822();
    let status_view = match status.clone() {
        Some(s) => format!("{:?}", s),
        None => "Unknown".to_owned(),
    };
    let author_url = format!("/authors/{}", author.clone());
    let article_anchor = format!("#{}", ulid.clone());
    let element_id = ulid.clone();

    view! {
            <div id={ element_id } class="article-container">
            <Collapsible>
                <CollapsibleHeader slot>
                    <Stack spacing=Size::Em(0.5)>
                    <H3>
                        <Anchor href={ article_anchor } title="Anchor to this article"/>
                        {title}
                    </H3>

                    <Link href=author_url>{ author }</Link>
                    Next review: {next_read_date.to_rfc2822()}
                </Stack>
            </CollapsibleHeader>

            <CollapsibleBody slot>
                <div style="display: flex; align-items: flex-start; flex-direction: column;">
                    <LinkExt href=source_url.clone() target=LinkExtTarget::Blank>
                        Source
                    </LinkExt>

                    {
                        let html_text = move || match article_content.get() {
                            None => "Loading...".to_string(),
                            Some(d) => d,
                        };
                        view! { <div inner_html=html_text></div> }
                    }

                </div>

            </CollapsibleBody>

        </Collapsible>
        </div>
    }
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

    let ingest_date_view = ingest_date.to_rfc2822();
    let status_view = match status.clone() {
        Some(s) => format!("{:?}", s),
        None => "Unknown".to_owned(),
    };
    let author_url = format!("/authors/{}", author.clone());
    let article_anchor = format!("/archive/#{}", ulid.clone());
    let element_id = ulid.clone();
    let ulid1 = ulid.clone();
    let ulid2 = ulid.clone();
    view! {
            <div id={ element_id } class="article-container">
            <Collapsible>
                <CollapsibleHeader slot>
                    <Stack spacing=Size::Em(0.5)>
                    <H3>
                        <Anchor href={ article_anchor } title="Anchor to this article"/>
                        {title}
                    </H3>

                    <Link href=author_url>{ author }</Link>
                    Next review: {next_read_date.to_rfc2822()}
                </Stack>
            </CollapsibleHeader>

            <CollapsibleBody slot>
                <div style="display: flex; align-items: flex-start; flex-direction: column;">
                    <LinkExt href=source_url.clone() target=LinkExtTarget::Blank>
                        Source
                    </LinkExt>

                    {
                        let html_text = move || match article_content.get() {
                            None => "Loading...".to_string(),
                            Some(d) => d,
                        };
                        view! { <div inner_html=html_text></div> }
                    }

                    <ButtonGroup>
                        <Button on_press=move |_| {
                            spawn_local(
                                requeue_article(
                                    Ulid::from_string(&ulid1).expect("Invalid ULID"),
                                    status.clone(),
                                ),
                            );
                        }>"Finished Reading"</Button>

                        <Button on_press=move |_| {
                            spawn_local(archive_article(Ulid::from_string(&ulid2).expect("Invalid ULID")));
                        }>Archive</Button>
                    </ButtonGroup>

                </div>

            </CollapsibleBody>

        </Collapsible>
        </div>
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

async fn archive_article(article_ulid: Ulid) {
    console_log("Archiving article");

    update_article_status(article_ulid, ArticleStatus::Archive).await;
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
    view! {
        <Box id="library-drawer-content">
            <H2>Navigation</H2>
            <LinkButton href="/">
                <H3>Home</H3>
            </LinkButton>
            <LinkButton href="/archive/">
                <H3>Archive</H3>
            </LinkButton>
            <LinkButton href="/faq/">
                <H3>FAQ</H3>
            </LinkButton>

        </Box>
    }
}

// Source flow
