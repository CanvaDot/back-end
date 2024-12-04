use actix_web::{get, rt::spawn, web::Payload, Error, HttpRequest, HttpResponse};
use actix_ws::{handle, AggregatedMessage};
use futures_util::StreamExt;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use crate::{helpers::{cells::processes::{get_canvas_spec, process_written_cell}, http::{socket_messages::SocketMessage, socket_session::WsSession}}, models::user::User};


lazy_static! {
    static ref SESSIONS: Mutex<Vec<WsSession>> = Mutex::new(Vec::new());
}

macro_rules! send_text {
    ($session:expr, $value:expr) => {
        if !$session.text($value).await {
            SESSIONS
                .lock()
                .await
                .retain(|s| s != &$session);
        }
    };
}

#[get("/session")]
pub async fn session(req: HttpRequest, stream: Payload, user: User) -> Result<HttpResponse, Error> {
    let (res, session, stream) = handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let mut session = WsSession::new(
        session.clone(),
        user
    );

    {
        let mut sessions = SESSIONS
            .lock()
            .await;

        let mut session = session.clone();

        let sent_init = session.text(
            SocketMessage::InitConnection(
                &session.user(),
                match get_canvas_spec() {
                    Ok(spec) => spec,
                    Err(err) => {
                        session.close(Some(err)).await;
                        return Ok(res);
                    }
                }
            )
        )
            .await;

        if sent_init {
            sessions.push(session);
        }
    }

    spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let payload = SocketMessage::from(text.to_string());

                    match payload {
                        SocketMessage::WriteCell(pos, col) => {
                            if !session.user().can_consume_credit() {
                                send_text!(
                                    session,
                                    SocketMessage::SendError(
                                        "Cannot consume a token at this moment."
                                            .into()
                                    )
                                );

                                continue;
                            }

                            let consumption = session
                                .user_mut()
                                .consume_credit()
                                .await;

                            if consumption.is_err() {
                                send_text!(
                                    session,
                                    SocketMessage::SendError(
                                        "Cannot consume a token at this moment.".into()
                                    )
                                );

                                continue;
                            }

                            if let Err(err) = process_written_cell(&session.user(), pos, col) {
                                send_text!(session, SocketMessage::SendError(err));

                                continue;
                            }
                        },

                        SocketMessage::SendError(_) => {
                            send_text!(session, payload);

                            continue;
                        },

                        _ => {}
                    }

                    let user = session.user();
                    let sender = payload.to_sender(&user);

                    let sender: String = match sender {
                        SocketMessage::SendError(_) => {
                            send_text!(session, sender);

                            continue;
                        },
                        _ => sender.into()
                    };

                    for session in SESSIONS.lock().await.iter_mut() {
                        send_text!(*session, &sender);
                    }
                },

                Ok(AggregatedMessage::Ping(ping)) => {
                    session.pong(&ping)
                        .await;
                },

                Err(_) | Ok(AggregatedMessage::Close(_)) => {
                    SESSIONS
                        .lock()
                        .await
                        .retain(|s| s == &session);

                    break;
                }

                _ => {}
            }
        }
    });

    Ok(res)
}
