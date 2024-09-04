

#[derive(Insertable, Queryable, Serialize, Deserialize)]
pub struct Roles {
    pub id: i32,
    pub user_type: UserType
}

#[derive(Insertable, Queryable, Serialize, Deserialize)]
pub struct Users {
    pub id: i32,
    pub role_id: String,
    pub user_type: UserType,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}

#[derive(Insertable, Queryable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUsers {
    pub role_id: String,
    pub user_type: UserType,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
