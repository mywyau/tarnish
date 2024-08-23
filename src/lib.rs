pub mod blog_controller;
pub mod blog_models;
pub mod schema;

pub use crate::blog_controller::{create_post, get_post, update_post, delete_post, establish_connection, DbPool};
pub use crate::blog_models::{Post, NewPost};
pub use crate::schema::posts;

pub mod worklog_controller;

pub use crate::worklog_controller::{create_post, get_post, update_post, delete_post, establish_connection, DbPool};
pub use crate::blog_models::{Post, NewPost};
pub use crate::schema::posts;