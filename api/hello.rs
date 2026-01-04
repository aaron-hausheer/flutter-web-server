use vercel_runtime::{run, Body, Error, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    Ok(Response::builder().status(200).body(Body::Text("ok".into()))?)
}
