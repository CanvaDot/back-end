use crate::{helpers::cells::{color::Color, position::Position, processes::CanvasSpec}, models::user::{MaybeUser, User}};

macro_rules! or_error {
    (r, $e:expr) => {
        match $e {
            Ok(r) => r,
            Err(err) => {
                return SocketMessage::SendError(format!("{err:#}"));
            }
        }
    };

    (o, $e:expr, $err:literal) => {
        match $e {
            Some(r) => r,
            None => {
                return SocketMessage::SendError(format!("{:#}", $err));
            }
        }
    };
}

pub enum SocketMessage<'u> {
    WriteCell(Position, Color),
    MoveCursor(Position),

    WroteCell(&'u User, Position, Color),
    MovedCursor(&'u User, Position),

    SendError(String),

    InitConnection(&'u MaybeUser, CanvasSpec)
}

impl<'u> SocketMessage<'u> {
    pub fn to_sender(self, user: &'u User) -> Self {
        match self {
            Self::WriteCell(position, color) =>
                Self::WroteCell(user, position, color),

            Self::MoveCursor(position) =>
                Self::MovedCursor(user, position),

            _ => Self::SendError(
                "Internal message conversion error."
                    .into()
            )
        }
    }
}

impl<'u> From<String> for SocketMessage<'u> {
    fn from(value: String) -> Self {
        let (op, params) = or_error!(
            o,
            value.split_once(";"),
            "Invalid message format."
        );

        match or_error!(r, op.parse::<i32>()) {
            1 => {
                let params = params.splitn(3, ',')
                    .collect::<Vec<_>>();

                if params.len() != 3 {
                    return Self::SendError("Invalid parameter length".into());
                }

                let left = format!("{},{}", params[0], params[1]);
                let right = params[2].to_string();

                Self::WriteCell(
                    or_error!(r, left.try_into()),
                    or_error!(r, right.try_into())
                )
            },

            2 => {
                Self::MoveCursor(
                    or_error!(r, params.to_string().try_into())
                )
            },

            _ => {
                Self::SendError("Invalid OP code.".into())
            }
        }
    }
}

impl<'u> Into<String> for SocketMessage<'u> {
    fn into(self) -> String {
        match self {
            Self::WriteCell(pos, col)
                => format!("1;{},{}", pos.to_string(), col.to_string()),

            Self::MoveCursor(pos)
                => format!("2;{}", pos.to_string()),

            Self::WroteCell(user, pos, col)
                => format!("3;{},{},{}", user.name(), pos.to_string(), col.to_string()),

            Self::MovedCursor(user, pos)
                => format!("4;{},{}", user.name(), pos.to_string()),

            Self::SendError(err)
                => format!("5;{err}"),

            Self::InitConnection(user, spec)
                => format!(
                    "6;{},{}",
                    match user {
                        MaybeUser::Authorized(user) => user.name(),
                        MaybeUser::Unauthorized => "null"
                    },
                    spec.to_string()
                )
        }
    }
}
