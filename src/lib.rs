pub mod connectors {
    // Declare each controller within the controllers module
    pub mod postgres_connector;
}


pub mod controllers {
    // Declare each controller within the controllers module
    pub mod skills_controller;
    pub mod worklog_controller;
    pub mod blog_controller;
}

pub mod models {
    // Declare each controller within the controllers module
    pub mod skills_models;
    pub mod worklog_models;
    pub mod blog_models;
}

pub mod schemas {
    // Declare each controller within the controllers module
    pub mod skills_schema;
    pub mod worklog_schema;
    pub mod blog_schema;
}

// Re-exporting items for easier access
pub use controllers::blog_controller::{create_post, get_post, update_post, delete_post};
pub use models::blog_models::{Post, NewPost};
pub use schemas::blog_schema::posts;


pub use controllers::skills_controller::{create_skill, get_skill, update_skill, delete_skill};
pub use models::skills_models::{Skill, NewSkill};
pub use schemas::skills_schema::skills;


pub use controllers::worklog_controller::{create_worklog, get_worklog, get_all_worklog, get_by_worklog_id, delete_all_worklog, delete_worklog};
pub use models::worklog_models::{Worklog, NewWorklog};
pub use schemas::worklog_schema::worklog;