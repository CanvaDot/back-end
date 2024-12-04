use actix_web::web::Bytes;
use actix_ws::{CloseCode, CloseReason, Session};
use uuid::Uuid;
use crate::models::user::User;


#[derive(Clone)]
pub struct WsSession {
    id: Uuid,
    session: Session,
    user: User
}

impl WsSession {
    pub fn new(session: Session, user: User) -> Self {
        Self {
            id: Uuid::new_v4(),
            session,
            user
        }
    }

    pub async fn pong(&mut self, message: &Bytes) -> bool {
        self.session.pong(message)
            .await
            .is_ok()
    }

    pub async fn text(&mut self, message: impl Into<String>) -> bool {
        self.session.text(message.into())
            .await
            .is_ok()
    }

    pub async fn close(self, message: Option<String>) -> bool {
        self.session.close(Some(CloseReason {
            code: CloseCode::Error,
            description: message
        }))
            .await
            .is_ok()
    }

    pub fn user(&self) -> User {
        self.user.clone()
    }

    pub fn user_mut(&mut self) -> &mut User {
        &mut self.user
    }
}

impl PartialEq for WsSession {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
