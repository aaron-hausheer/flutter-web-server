use axum::Router;
use vercel_axum::VercelLayer;
use vercel_runtime::{run, Error};

fn app() -> Router {
    dbconnect::build_router()
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = app().layer(VercelLayer);
    run(app).await
}