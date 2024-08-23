use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Insertable, Queryable, Serialize, Deserialize)]
pub struct Skill {
    pub id: i32,
    pub skill_id: String,
    pub skill_name: String,
    pub body: String,
}
