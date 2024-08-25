use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use crate::schemas::skills_schema::skills;  // Make sure this is the correct import for your schema

#[derive(Queryable, Serialize, Deserialize)]
pub struct Skill {
    pub id: i32,
    pub skill_id: String,
    pub skill_name: String,
    pub body: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = skills)]
pub struct NewSkill {
    pub skill_id: String,
    pub skill_name: String,
    pub body: String,
}
