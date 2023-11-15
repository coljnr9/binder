use leptonic::prelude::*;
use leptos::{error::Result, leptos_dom::logging::console_log, *};
use leptos_icons::BsIcon;
use leptos_router::*;

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    mount_to_body(|| view! { <App/> })
}

#[derive(Debug, Clone, Copy)]
pub struct AppLayoutContext {
    pub library_drawer_closed: Signal<bool>,
    set_library_drawer_closed: WriteSignal<bool>,
}

impl AppLayoutContext {
    pub fn close_library_drawer(&self) {
        self.set_library_drawer_closed.set(true);
    }

    pub fn toggle_library_drawer(&self) {
        console_log("Toggling library drawer");
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
        // <Box id="app">
        // <Layout/>
        // </Box>
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
#[component]
pub fn Layout() -> impl IntoView {
    let (library_drawer_closed, set_library_drawer_closed) = create_signal(true);
    let ctx = AppLayoutContext {
        library_drawer_closed: library_drawer_closed.into(),
        set_library_drawer_closed,
    };

    provide_context(ctx);

    view! {
        <LayoutAppBar/>
        <LibraryDrawer/>

        // Whatever page needs to be rendered
        <Outlet/>
    }
}

#[component]
pub fn LayoutAppBar() -> impl IntoView {
    let ctx = use_context::<AppLayoutContext>().unwrap();

    view! {
        <AppBar id="app-bar" height=Size::Em(4.5)>
            <Icon
                id="library-trigger"
                class="library-icon"
                icon=BsIcon::BsList
                on:click=move |_| ctx.toggle_library_drawer()
            />

            <H1>"Binder"</H1>

            <ThemeToggle off=LeptonicTheme::Light on=LeptonicTheme::Dark/>
        </AppBar>
    }
}

//  The main feed of articles to read/listen to
#[component]
pub fn Queue() -> impl IntoView {
    view! {
        <Box id="queue">
            <div style="display: flex; justify-content: center">
                <H3>Queue</H3>
            </div>
            <Stack spacing=Size::Em(
                0.5,
            )>{(0..50).map(|_| view! { <Skeleton height=Size::Em(35.0)/> }).collect_view()}</Stack>

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

#[component]
pub fn ArticleDisplay(article_name: String, mp3_url: String) -> impl IntoView {
    view! {

       <script
          type="module"
          crossorigin
          src="https://embed.type3.audio/player.js"
        />

        <type-3-player
          mp3-url={mp3_url}
          author="Nick Bostrom"
          title="Base Camp for Mount Ethics"
          cover-image-url="https://radiobostrom.com/images/cover-art-radio-bostrom-500x500.jpeg"
          listen-to-this-page="true"
          listen-to-this-page-text={article_name}
        />
    }
}

#[component]
pub fn LibraryDrawerContent() -> impl IntoView {
    let ctx = use_context::<AppLayoutContext>().unwrap();

    view! {
        <Box id="library-drawer-content">
            <H3>Next Up</H3>
            <ArticleDisplay
                article_name="Next Up Article".to_string()
                mp3_url="https://download.samplelib.com/mp3/sample-3s.mp3".to_string()
            />

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
                            <A href="faq"><H3>FAQ</H3></A>
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
                                view! {
                                    <ArticleDisplay
                                        article_name=format!("Article {}", n)
                                        mp3_url="https://download.samplelib.com/mp3/sample-3s.mp3"
                                            .to_string()
                                    />
                                }
                            })
                            .collect_view()}
                    </Stack>
                </CollapsibleBody>
            </Collapsible>

        </Box>
    }
}

// Source flow
