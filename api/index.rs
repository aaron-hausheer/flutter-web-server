use vercel_axum::VercelLayer;
use vercel_runtime::{run, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = dbconnect::build_router().layer(VercelLayer);
    run(app).await
}