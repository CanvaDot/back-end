use actix_web::{get, rt::spawn, web::Payload, Error, HttpRequest, HttpResponse};
use actix_ws::{handle, AggregatedMessage};
use futures_util::StreamExt;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use crate::{helpers::{cells::processes::{get_canvas_spec, process_written_cell}, http::{socket::{CanvaDotSession, CanvaDotSessionMessage}, socket_messages::SocketMessage}}, models::user::User};


lazy_static! {
    static ref SESSIONS: Mutex<Vec<CanvaDotSession>> = Mutex::new(Vec::new());
}

#[get("/session")]
pub async fn session(req: HttpRequest, stream: Payload, user: User) -> Result<HttpResponse, Error> {
    let (res, session, stream) = handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    let mut session = CanvaDotSession::new(
        session.clone(),
        user
    );

    {
        let mut sessions = SESSIONS
            .lock()
            .await;

        let mut session = session.clone();

        let spec = match get_canvas_spec() {
            Ok(spec) => spec,
            Err(err) => {
                session.close(Some(err)).await;
                return Ok(res);
            }
        };

        session.send(CanvaDotSessionMessage::Text(
            SocketMessage::InitConnection(&session.user(), spec)
                .into()
        ))
            .await;

        sessions.push(session);
    }

    spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let payload = SocketMessage::from(text.to_string());

                    match payload {
                        SocketMessage::WriteCell(pos, col) => {
                            if !session.user().can_consume_credit() {
                                session.send(CanvaDotSessionMessage::Text(
                                    SocketMessage::SendError(
                                        "Cannot consume a token at this moment."
                                            .into()
                                    )
                                        .into()
                                )).await;

                                continue;
                            }

                            let consumption = session
                                .user_mut()
                                .consume_credit()
                                .await;

                            if consumption.is_err() {
                                session.send(CanvaDotSessionMessage::Text(
                                    SocketMessage::SendError(
                                        "Cannot consume a token at this moment."
                                            .into()
                                    )
                                        .into()
                                )).await;

                                continue;
                            }

                            if let Err(err) = process_written_cell(pos, col) {
                                session.send(CanvaDotSessionMessage::Text(
                                    SocketMessage::SendError(err)
                                        .into()
                                ))
                                    .await;
                                continue;
                            }
                        },

                        SocketMessage::SendError(_) => {
                            session.send(CanvaDotSessionMessage::Text(
                                payload.into()
                            ))
                                .await;
                            continue;
                        },

                        _ => {}
                    }

                    let sender = payload.to_sender(session.user());

                    let sender: String = match sender {
                        SocketMessage::SendError(_) => {
                            session.send(CanvaDotSessionMessage::Text(
                                sender.into()
                            ))
                                .await;
                            continue;
                        },
                        _ => sender.into()
                    };

                    for session in SESSIONS.lock().await.iter_mut() {
                        session.send(CanvaDotSessionMessage::Text(sender.clone()))
                            .await;
                    }
                },

                Ok(AggregatedMessage::Ping(ping)) => {
                    session.send(CanvaDotSessionMessage::Pong(&ping))
                        .await;
                },

                Err(_) | Ok(AggregatedMessage::Close(_)) => {
                    let mut sessions = SESSIONS
                        .lock()
                        .await;

                    println!("Disconnected");

                    sessions.retain(|s| s == &session);
                    break;
                }

                _ => {}
            }
        }
    });

    Ok(res)
}
