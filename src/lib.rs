pub mod connectors {
    pub mod postgres_connector;
}

pub mod controllers {
    // Declare each controller within the controllers module
    pub mod register_user_controller;
    pub mod skills_controller;
    pub mod worklog_controller;
    pub mod blog_controller;
}
pub mod models {
    pub mod user_type;
}

pub mod table_models {
    pub mod users;
    pub mod skills_models;
    pub mod worklog_models;
    pub mod blog_models;
}

pub mod schemas {
    pub mod user_schema;
    pub mod skills_schema;
    pub mod worklog_schema;
    pub mod blog_schema;
}

// Re-exporting items for easier access

pub use connectors::postgres_connector::{DbPool, RealDbConnector};


pub use controllers::blog_controller::{create_post, delete_post, get_post, update_post};
pub use schemas::blog_schema::posts;
pub use table_models::blog_models::{NewPost, Post};


pub use controllers::skills_controller::{create_skill, delete_skill, get_skill, update_skill};
pub use schemas::skills_schema::skills;
pub use table_models::skills_models::{NewSkill, Skill};


pub use controllers::worklog_controller::{create_worklog, delete_all_worklog, delete_worklog, get_all_worklog, get_by_worklog_id, get_worklog};
pub use schemas::worklog_schema::worklog;
pub use table_models::worklog_models::{NewWorklog, Worklog};


pub use controllers::register_user_controller::create_user;
pub use schemas::user_schema::users;
pub use table_models::users::{NewUsers, Users};
