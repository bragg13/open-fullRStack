use axum::{
    routing::{delete, get, post, put},
    Router,
};
use create::create_blog;
use delete::delete_blog;
use read::{get_blog, get_blogs};
use update::update_blog;
pub mod create;
pub mod delete;
pub mod read;
pub mod test_helper;
pub mod update;

pub fn routes() -> Router {
    Router::new()
        .route("/blogs/{id}", get(get_blog))
        .route("/blogs/{id}", put(update_blog))
        .route("/blogs/{id}", delete(delete_blog))
        .route("/blogs", get(get_blogs))
        .route("/blogs", post(create_blog))
}
