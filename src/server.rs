use alloc::string::String as DynString;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use picoserve::{make_static, AppBuilder, AppRouter};

use crate::WEB_TASK_POOL_SIZE;

struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        use picoserve::{
            response::Redirect,
            routing::{get, parse_path_segment, put},
        };
        picoserve::Router::new()
            .nest_service(
                "",
                include!(concat!(env!("OUT_DIR"), "/website_directory.part.rs")),
            )
            .route("/", get(|| async move { Redirect::to("/index.html") }))
            .route(
                "/signals",
                get(|| async move {
                    r#"{ "0": { "name": "Hello", "curve": [1,0,0,0,1,0,1,0,1,0,0,0,0,0,0,0,1,1,1,1,1,1,0,0,1,1] } }"#
                })
                .post(|s: DynString| async move {
                    log::info!("Adding new signal {s}");
                    r#"{ "1": { "name": "Welt", "curve": [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1] } }"#
                }),
            )
            .route(
                ("/signals", parse_path_segment::<usize>()),
                put(|id| async move {
                    log::info!("Replaying signal {id}");
                })
                .post(|id, s: DynString| async move {
                    log::info!("Renaming signal {id} to {s}");
                })
                .delete(|id| async move {
                    log::info!("Deleting signal {id}");
                }),
            )
    }
}

pub async fn init(spawner: &Spawner, stack: Stack<'static>) {
    let app = make_static!(AppRouter<AppProps>, AppProps.build_app());

    let config = make_static!(
        picoserve::Config<Duration>,
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            read_request: Some(Duration::from_secs(1)),
            write: Some(Duration::from_secs(4)),
        })
        .keep_connection_alive()
    );

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web_task(id, stack, app, config));
    }
}

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await
}
