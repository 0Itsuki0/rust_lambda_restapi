pub mod event;
pub mod event_service;
pub mod handler;
pub mod handler_params;

use aws_sdk_dynamodb::Client;
use axum::{
    routing::{get, put},
    Router,
};
use event_service::EventService;
use lambda_http::{run, tracing, Error};
use std::env::{self, set_var};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    set_var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH", "true");

    let config = aws_config::load_from_env().await;
    let db_client = Client::new(&config);
    let table_name = env::var("DYNAMO_TABLE_NAME")?;

    let event_service = EventService::new(db_client, &table_name);

    let event_api = Router::new()
        .route("/", get(handler::get_events).post(handler::post_event))
        .route(
            "/:id",
            get(handler::get_event_single).delete(handler::delete_event_single),
        )
        .route("/:id/title", put(handler::put_event_title));

    let app = Router::new()
        .nest("/events", event_api)
        .with_state(event_service);

    run(app).await
}
