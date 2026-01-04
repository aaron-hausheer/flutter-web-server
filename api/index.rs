use tower::ServiceExt;
use http_body_util::BodyExt;
use vercel_runtime::{run, Error, Request, Body, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = dbconnect::build_router();

    run(move |req: Request| {
        let app = app.clone();
        async move {
            let res = app
                .oneshot(req)
                .await
                .map_err(|e| Error::from(e.to_string()))?;

            let (parts, body) = res.into_parts();

            let bytes = body
                .collect()
                .await
                .map_err(|e| Error::from(e.to_string()))?
                .to_bytes();

            Ok::<Response<Body>, Error>(
                Response::from_parts(parts, Body::Binary(bytes.to_vec()))
            )
        }
    })
    .await
}
