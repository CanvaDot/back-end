
#[macro_export]
macro_rules! grv {
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => {
                return actix_web::HttpResponse::InternalServerError()
                    .body(format!("{e:#}"));
            }
        }
    }};
}

#[macro_export]
macro_rules! gro {
    ($e:expr, $err:literal) => {{
        match $e {
            Some(v) => v,
            None => {
                return actix_web::HttpResponse::InternalServerError()
                    .body(format!("{:#}", $err));
            }
        }
    }};
}
