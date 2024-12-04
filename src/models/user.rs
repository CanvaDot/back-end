use std::{future::{ready, Ready}, ops::Add, time::{SystemTime, SystemTimeError, UNIX_EPOCH}};
use actix_web::{cookie::time::Duration, dev::Payload, error::ErrorUnauthorized, Error as ActixError, FromRequest, HttpRequest};
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use jsonwebtoken::{decode, encode, errors::Error as JwtError, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as, Error as SqlxError};
use time::OffsetDateTime;
use thiserror::Error;
use crate::{db, helpers::{database::connection::DbConnectionError, http::jwt::Claims}, jwt_hash};


#[derive(Error, Debug)]
pub enum UserError {
    #[error("{0:#}")]
    DbQuery(#[from] SqlxError),

    #[error("{0:#}")]
    DbConn(#[from] DbConnectionError),

    #[error("{0:#}")]
    Bcrypt(#[from] BcryptError),

    #[error("{0:#}")]
    Jwt(#[from] JwtError),

    #[error("{0:#}")]
    SystemTime(#[from] SystemTimeError),

    #[error("Cannot consume a token at this time.")]
    Unconsumable
}

type UserResult<R> = Result<R, UserError>;

#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    id: i32,
    email: String,
    username: String,
    password: String,
    credits: i32,
    next_free_credit: OffsetDateTime,
    activated: bool
}

impl User {
    pub async fn exists(email: &String, username: &String) -> UserResult<bool> {
        Ok(query!(
            r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM users
                    WHERE email = $1 OR username = $2
                )
            "#,
            email,
            username
        )
            .fetch_one(db!())
            .await?
            .exists
            .unwrap_or(false))
    }

    pub async fn insert(email: String, username: String, password: String) -> UserResult<Self> {
        query_as!(
            Self,
            r#"
                INSERT INTO users (email, username, password)
                VALUES ($1, $2, $3)
                RETURNING *
            "#,
            email,
            username,
            hash(&password, DEFAULT_COST)?
        )
            .fetch_one(db!())
            .await
            .map_err(|err| UserError::DbQuery(err))
    }

    pub async fn login(email: String, password: String) -> UserResult<Option<Self>> {
        query_as!(
            Self,
            r#"
                SELECT *
                FROM users
                WHERE email = $1
            "#,
            email
        )
            .fetch_optional(db!())
            .await
            .map(|mut user| user
                .take_if(|user| verify(
                    password,
                    &user.password
                )
                    .unwrap_or(false))
            )
            .map_err(|err| UserError::DbQuery(err))
    }

    pub fn from_jwt(token: String) -> Result<Self, UserError> {
        Ok(
            decode::<Claims<User>>(
                &token,
                jwt_hash!(decode),
                &Validation::default()
            )?
                .claims
                .into_inner()
        )
    }

    pub fn jwt(&self) -> Result<String, UserError> {
        let exp = SystemTime::now()
            .add(Duration::hours(12))
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        Ok(encode(
            &Header::default(),
            &Claims::new(exp as usize, self),
            jwt_hash!(encode)
        )?)
    }

    pub fn name(&self) -> &String {
        &self.username
    }

    pub async fn activate(&mut self) -> UserResult<()> {
        query!(
            r#"
                UPDATE users
                SET activated = true
                WHERE id = $1
            "#,
            self.id
        )
            .execute(db!())
            .await?;

        self.activated = true;

        Ok(())
    }

    pub async fn consume_credit(&mut self) -> UserResult<()> {
        let now = OffsetDateTime::now_utc();

        if self.next_free_credit > now {
            if self.credits <= 0 {
                return Err(UserError::Unconsumable);
            }

            query!(
                r#"
                    UPDATE users
                    SET credits = credits - 1
                    WHERE id = $1
                "#,
                self.id
            )
                .execute(db!())
                .await?;

            self.credits -= 1;

            return Ok(());
        }

        let next_time = now
            .add(Duration::hours(12));

        query!(
            r#"
                UPDATE users
                SET next_free_credit = $1
                WHERE id = $2
            "#,
            next_time,
            self.id
        )
            .execute(db!())
            .await?;

        self.next_free_credit = next_time;

        Ok(())
    }

    pub fn can_consume_credit(&self) -> bool {
        self.next_free_credit <= OffsetDateTime::now_utc() || self.credits > 0
    }
}

impl FromRequest for User {
    type Error = ActixError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        req.cookie("Session")
            .and_then(|cookie| Self::from_jwt(
                    cookie
                        .value()
                        .to_string()
                )
                    .ok()
            )
            .map_or(
                ready(Err(ErrorUnauthorized("Provide Session cookie for this endpoint."))),
                |res| ready(Ok(res))
            )
    }
}
