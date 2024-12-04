use actix_web::web::Bytes;
use actix_ws::{CloseCode, CloseReason, Session};
use uuid::Uuid;
use crate::models::user::User;


pub enum CanvaDotSessionMessage<'i> {
    Text(String),
    Pong(&'i Bytes)
}

#[derive(Clone)]
pub struct CanvaDotSession {
    id: Uuid,
    session: Session,
    user: User
}

impl CanvaDotSession {
    pub fn new(session: Session, user: User) -> Self {
        Self {
            id: Uuid::new_v4(),
            session,
            user
        }
    }

    pub async fn send<'i>(&mut self, message: CanvaDotSessionMessage<'i>) {
        match message {
            CanvaDotSessionMessage::Text(text) => {
                let _ = self.session.text(text).await;
            },

            CanvaDotSessionMessage::Pong(ping) => {
                let _ = self.session.ping(&ping).await;
            },
        }
    }

    pub async fn close<'i>(self, message: Option<String>) {
        let _ = self.session.close(Some(CloseReason {
            code: CloseCode::Error,
            description: message
        }));
    }

    pub fn id(&self) -> String {
        self.id.to_string()
    }

    pub fn user(&self) -> &User {
        &self.user
    }
}

impl PartialEq for CanvaDotSession {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
