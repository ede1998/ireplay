use core::cell::RefCell;

use alloc::rc::Rc;
use alloc::vec;
use alloc::{borrow::ToOwned, string::String as DynString, vec::Vec as DynVec};
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use hashbrown::HashMap as DynHashMap;
use picoserve::{
    extract::{Json, State},
    make_static, AppRouter, AppWithStateBuilder,
};
use serde::Serialize;

use crate::WEB_TASK_POOL_SIZE;

#[derive(Clone, Debug)]
struct SignalDatabase {
    signals: Rc<RefCell<DynHashMap<usize, Signal, nohash_hasher::BuildNoHashHasher<usize>>>>,
}

#[derive(Clone, Debug, Serialize)]
struct Signal {
    name: DynString,
    curve: DynVec<u8>,
}

struct AppProps;

impl AppWithStateBuilder for AppProps {
    type PathRouter = impl picoserve::routing::PathRouter<Self::State>;
    type State = SignalDatabase;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, Self::State> {
        use picoserve::{
            response::{Redirect, StatusCode},
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
                get(|State::<SignalDatabase>(state)| async move {
                    Json(state.signals.borrow().clone())
                })
                .post(
                    |State::<SignalDatabase>(state), name: DynString| async move {
                        log::info!("Adding new signal {name}");

                        let signal = Signal {
                            name,
                            curve: vec![0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1],
                        };
                        let mut signals = state.signals.borrow_mut();
                        let next_id = signals.len();
                        signals.insert(next_id, signal.clone());

                        Json([(next_id, signal)])
                    },
                ),
            )
            .route(
                ("/signals", parse_path_segment::<usize>()),
                put(|id, State::<SignalDatabase>(state)| async move {
                    match state.signals.borrow_mut().get(&id) {
                        Some(signal) => {
                            log::info!("Replaying signal {id} with name {}", signal.name);
                            StatusCode::OK
                        }
                        None => {
                            log::warn!("Failed to rename signal {id}. Does not exist");
                            StatusCode::NOT_FOUND
                        }
                    }
                })
                .post(
                    |id, State::<SignalDatabase>(state), name: DynString| async move {
                        match state.signals.borrow_mut().get_mut(&id) {
                            Some(signal) => {
                                log::info!("Renaming signal {id} to {name}");
                                signal.name = name;
                                StatusCode::OK
                            }
                            None => {
                                log::warn!("Failed to rename signal {id}. Does not exist");
                                StatusCode::NOT_FOUND
                            }
                        }
                    },
                )
                .delete(|id, State::<SignalDatabase>(state)| async move {
                    if state.signals.borrow_mut().remove(&id).is_some() {
                        log::info!("Deleted signal {id}");
                        StatusCode::OK
                    } else {
                        log::warn!("Failed to delete signal {id}. Does not exist");
                        StatusCode::NOT_FOUND
                    }
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

    let state = &*make_static!(
        SignalDatabase,
        SignalDatabase {
            signals: Rc::new(RefCell::new(
                [(
                    0,
                    Signal {
                        name: "Hello".to_owned(),
                        curve: vec![
                            1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0,
                            1, 1,
                        ],
                    },
                )]
                .into_iter()
                .collect(),
            )),
        }
    );

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web_task(id, stack, app, config, state));
    }
}

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
    state: &'static SignalDatabase,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve_with_state(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
        state,
    )
    .await
}
