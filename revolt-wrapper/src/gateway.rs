use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, Stream, StreamExt,
};
use tokio::{net::TcpStream, sync::Mutex, task::JoinHandle, time};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

use crate::models::{Event, ThreadResult};

type WsReceiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type WsSender = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub const GATEWAY_URL: &str = "wss://ws.revolt.chat";
const _GATEWAY_VERSION: &str = "1";

#[derive(Debug, Clone)]
pub struct Events {
    gateway_url: String,
    token: String,
    rx: Arc<Mutex<Option<WsReceiver>>>,
    ping: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[derive(Debug, Clone)]
pub struct GatewayClient {
    pub gateway_url: String,
    token: String,
}

impl GatewayClient {
    pub fn new(token: String) -> GatewayClient {
        GatewayClient {
            gateway_url: GATEWAY_URL.to_string(),
            token,
        }
    }

    pub fn gateway_url(mut self, url: String) -> GatewayClient {
        self.gateway_url = url;
        self
    }

    pub fn token(mut self, token: String) -> GatewayClient {
        self.token = token;
        self
    }

    pub async fn get_events(&self) -> ThreadResult<Events> {
        let mut events = Events::new(self.gateway_url.to_string(), self.token.to_string());
        events.connect().await?;
        Ok(events)
    }
}

fn start_ping(mut tx: WsSender) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let current_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            match tx
                .send(Message::Binary(
                    rmp_serde::to_vec_named(&Event::Ping {
                        data: current_timestamp,
                    })
                    .unwrap(),
                ))
                .await
            {
                Ok(_) => time::sleep(Duration::from_secs(20)).await,
                Err(err) => {
                    log::debug!("Encountered error while pinging {:?}", err);
                    break;
                }
            }
        }
    })
}

fn get_url(gateway_url: &str, token: &str) -> Url {
    Url::parse_with_params(
        gateway_url,
        &[
            ("version", _GATEWAY_VERSION),
            ("token", token),
            ("format", "msgpack"),
        ],
    )
    .unwrap()
}

impl Events {
    fn new(gateway_url: String, token: String) -> Self {
        Self {
            gateway_url,
            token,
            rx: Arc::new(Mutex::new(None)),
            ping: Arc::new(Mutex::new(None)),
        }
    }

    async fn connect(&mut self) -> ThreadResult<()> {
        log::debug!("Events connecting");
        let mut ping = self.ping.lock().await;

        if ping.is_some() {
            ping.as_mut().unwrap().abort();
        }

        let (socket, _) = connect_async(
            Url::parse_with_params(
                &self.gateway_url,
                &[
                    ("version", _GATEWAY_VERSION),
                    ("token", self.token.as_str()),
                    ("format", "msgpack"),
                ],
            )
            .unwrap(),
        )
        .await?;
        let (tx, rx) = socket.split();

        *ping = Some(start_ping(tx));
        *self.rx.lock().await = Some(rx);
        Ok(())
    }

    async fn reconect(
        waker: Waker,
        gateway_url: String,
        token: String,
        rx: Arc<Mutex<Option<WsReceiver>>>,
        ping: Arc<Mutex<Option<JoinHandle<()>>>>,
    ) {
        let mut wait = 1;
        loop {
            let mut ping = ping.lock().await;
            if ping.is_some() {
                ping.as_mut().unwrap().abort();
            }
            match connect_async(get_url(&gateway_url, &token)).await {
                Ok((socket, _)) => {
                    let (tx, new_rx) = socket.split();
                    *ping = Some(start_ping(tx));
                    *rx.lock().await = Some(new_rx);
                    log::debug!("Reconnected to websocket");
                    break;
                }
                Err(err) => {
                    log::info!(
                        "Websocket reconnection failed {}, trying again in {}s",
                        err,
                        wait
                    );
                    thread::sleep(Duration::from_secs(wait));
                    if wait < 64 {
                        wait *= 2;
                    }
                }
            }
        }
        waker.wake();
    }
}

impl Stream for Events {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let mut rx = futures::executor::block_on(async { self.rx.lock().await });
            if rx.is_none() {
                continue;
            }

            match rx.as_mut().unwrap().poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(msg))) => match msg {
                    Message::Binary(msg) => match rmp_serde::from_slice(msg.as_slice()) {
                        Ok(event) => return Poll::Ready(Some(event)),
                        Err(rmp_serde::decode::Error::Syntax(msg)) => {
                            if !msg.starts_with("unknown variant") {
                                log::debug!("Failed to parse event {:?}", msg);
                            }
                        }
                        Err(err) => {
                            log::debug!("Failed to parse event {:?}", err);
                        }
                    },
                    Message::Close(_) => {
                        log::debug!("Websocket closed, reconnecting");
                        tokio::spawn(Events::reconect(
                            cx.waker().clone(),
                            self.gateway_url.clone(),
                            self.token.clone(),
                            Arc::clone(&self.rx),
                            Arc::clone(&self.ping),
                        ));
                        return Poll::Pending;
                    }
                    _ => {
                        log::debug!("Received unknown message");
                    }
                },
                Poll::Pending => break Poll::Pending,
                Poll::Ready(None) => {
                    log::debug!("Websocket closed, reconnecting");
                    tokio::spawn(Events::reconect(
                        cx.waker().clone(),
                        self.gateway_url.clone(),
                        self.token.clone(),
                        Arc::clone(&self.rx),
                        Arc::clone(&self.ping),
                    ));
                    return Poll::Pending;
                }
                _ => {}
            }
        }
    }
}
