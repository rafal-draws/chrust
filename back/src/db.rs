#[allow(unused)]
pub mod db_conn {

    use chrono::{Local, NaiveDateTime, Utc};
    use serde::{Deserialize, Serialize};
    use sqlx::{prelude::FromRow, query, query_as, Pool, Postgres};
    use tracing::info;
    use std::env;
    use uuid::Uuid;

    use crate::http::handlers::delete::DeleteStatus;
    #[allow(dead_code)]
    #[derive(Debug)]
    pub enum AuthError {
        UUIDError(String),
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    pub enum SqlError {
        UploadQueryError(String),
    }

    #[derive(FromRow, Debug, Serialize)]
    pub struct User {
        pub id: i64,
        pub username: String,
        pub uuid: String,
        pub created_at: NaiveDateTime, // pub songs: Vec<String>
    }

    impl User {
        pub async fn new(username: String) -> Self {
            let uuid = Uuid::new_v4().to_string();

            let database_user = Self::create_user(username, uuid).await;

            Self {
                id: database_user.id,
                uuid: database_user.uuid,
                username: database_user.username,
                created_at: Utc::now().naive_utc(), // songs: vec![],
            }
        }

        async fn create_user(username: String, uuid: String) -> User {
            // TODO wrap in option
            let pool = sqlx::PgPool::connect(
                &env::var("DATABASE_URL")
                    .expect("Database URL should be reachable when creating user"),
            )
            .await
            .expect("Should be valid sqlite pool");

            info!("pool is valid: {:?}", &pool);

            let mut tx = pool.begin().await.expect("should create transaction");
            
            let query = query(
                r#"INSERT INTO users (username, uuid, created_at) values ($1, $2, CURRENT_TIMESTAMP)"#
            )
            .bind(&username)
            .bind(&uuid)
            .execute(&mut *tx)
            .await
            .expect("Should execute transaction creating a user");

            info!("User added successfully: {:?}", query);

            tx.commit().await.expect("Transaction should be closed");

            
            let user_struct: User = sqlx::query_as::<_, User>(
                "SELECT id, username, uuid, created_at from users where uuid = $1",
            )
            .bind(uuid)
            .fetch_one(&pool)
            .await
            .expect("User should exist at this point");


            user_struct
        }
    }




    #[derive(FromRow, Debug, Deserialize, Serialize)]
    pub struct Upload {
        pub id: i64,
        pub user_uuid: String,
        pub upload_uuid: String,
        pub file_name: String,
        pub added: NaiveDateTime,
        pub ready: bool,
    }



    pub fn get_default_upload() -> Vec<Upload> {
        let mut default_vec = Vec::new();

        default_vec.insert(
            0,
            Upload {
                id: 0,
                upload_uuid: "insert".to_string(),
                user_uuid: "your first".to_string(),
                file_name: "song".to_string(),
                added: Local::now().naive_local(),
                ready: false,
            },
        );

        default_vec
    }

    pub async fn get_all_uploads(user_uuid: &String) -> Result<Vec<Upload>, SqlError> {
        let pool = get_pool().await;
        let mut tx = pool.begin().await.expect("should create transaction");

        let uploads_vec = sqlx::query_as::<_, Upload>(
            "SELECT DISTINCT id, user_uuid, upload_uuid, file_name, added, ready from uploads where user_uuid = $1"
        )
        .bind(user_uuid)
        .fetch_all(&mut *tx)
        .await;

        tx.commit().await.expect("Transaction should be closed");

        match uploads_vec {
            Ok(v) => {
                tracing::info!("{:?}", v);
                Ok(v)
            }
            Err(e) => Err(SqlError::UploadQueryError(format!(
                "Uploads couldn't be fetched. {} \n {}",
                user_uuid, e
            ))),
        }
    }

     pub async fn drop_all_uploads() -> Result<(), SqlError> {
        let pool = get_pool().await;
        let mut tx = pool.begin().await.expect("should create transaction");

        let uploads_vec = sqlx::query(
            "DELETE FROM uploads"
        )
        .execute(&mut *tx)
        .await;

        tx.commit().await.expect("Transaction should be closed");

        match uploads_vec {
            Ok(v) => {
                tracing::info!("Deleted {} rows from uploads table", v.rows_affected());
            Ok(())
            }
            Err(e) => Err(SqlError::UploadQueryError(format!(
                "Uploads couldn't be removed. {} \n",
                e
            ))),
        }
    }

    pub async fn drop_all_users() -> Result<(), SqlError> {
        let pool = get_pool().await;
        let mut tx = pool.begin().await.expect("should create transaction");

        let uploads_vec = sqlx::query(
            "DELETE FROM users"
        )
        .execute(&mut *tx)
        .await;

        tx.commit().await.expect("Transaction should be closed");

        match uploads_vec {
            Ok(v) => {
                tracing::info!("Deleted {} rows from users table", v.rows_affected());
            Ok(())
            }
            Err(e) => Err(SqlError::UploadQueryError(format!(
                "Uploads couldn't be removed. {} \n",
                e
            ))),
        }
    }


    pub async fn get_upload(upload_uuid: String) -> Result<Upload, SqlError> {
        let pool = get_pool().await;
        let mut tx = pool.begin().await.expect("should create transaction");

        let upload = sqlx::query_as::<_, Upload>(
            "SELECT id, user_uuid, upload_uuid, file_name, added from uploads where upload_uuid = $1"
        )
        .bind(&upload_uuid)
        .fetch_one(&mut *tx)
        .await;

        tx.commit().await.expect("Transaction should be closed");

        match upload {
            Ok(v) => {
                tracing::info!("{:?}", v);
                Ok(v)
            }
            Err(e) => Err(SqlError::UploadQueryError(format!(
                "Uploads couldn't be fetched. {} \n {} ",
                upload_uuid, e
            ))),
        }
    }

    pub async fn insert_upload_to_db(user_uuid: &String, file_name: &String) -> Upload {
        let pool = get_pool().await;
        let upload_uuid = Uuid::new_v4().to_string();

        
        let mut tx = pool.begin().await.expect("should create transaction");
        
        info!("Creating query: Insert in (INSERT INTO uploads (user_uuid, upload_uuid, file_name, added, ready) values (?, ?, ?, CURRENT_TIMESTAMP, false)");
        info!("Provided user_uuid: {:?}", &user_uuid);
        info!("Provided upload_uuid{:?}", &upload_uuid);
        info!("Provided file_name: {:?}", &file_name);
        
        let query = query(
            r#"INSERT INTO uploads (user_uuid, upload_uuid, file_name, added, ready) values ($1, $2, $3, CURRENT_TIMESTAMP, false)"#
        )
        .bind(user_uuid)
        .bind(&upload_uuid)
        .bind(file_name)
        .execute(&mut *tx)
        .await
        .expect(&format!("Should insert record for {} {}", user_uuid, file_name));

        info!("UPLOAD INSERTED SUCCESSFULLY: {:?}", query);

        let user_struct: Upload = sqlx::query_as::<_, Upload>(
            "SELECT id, user_uuid, upload_uuid, file_name, added, ready from uploads where upload_uuid = $1"
        )
        .bind(upload_uuid)
        .fetch_one(&mut *tx)
        .await
        .expect("Insert should exist at this point");

        info!("UPLOAD DATA -> {:?}", &user_struct);

        tx.commit().await.expect("Transaction should be closed");

        user_struct
    }

    pub async fn delete_upload_db(upload_uuid: String, user_uuid: String) -> DeleteStatus {
        let pool = get_pool().await;

        let mut tx = pool.begin().await.expect("should create transaction");

        tracing::info!("DELETING UPLOAD REQUEST BEGIN");
        tracing::info!("user_uuid: {}", &user_uuid);
        tracing::info!("upload_uuid: {}", &upload_uuid);

        let row_delete =
            sqlx::query("DELETE FROM uploads where upload_uuid = $1 and user_uuid = $2")
                .bind(&upload_uuid)
                .bind(&user_uuid)
                .execute(&mut *tx)
                .await
                .expect("Delete be possible at this point");

        match tx.commit().await {
            Ok(a) => {
                tracing::info!("DELETE REQUEST FULFILLED for {:?}, upload_uuid {:?}", &user_uuid, &upload_uuid);
                tracing::info!("{:?}, {:?}", a, row_delete);
                DeleteStatus {
                    upload_uuid: upload_uuid,
                    user_uuid: user_uuid,
                    status: true,
                }
            }
            Err(e) => {
                tracing::info!("DELETE REQUEST COULDN'T BE FULFILLED for {:?}, upload_uuid {:?}", &user_uuid, &upload_uuid);
                tracing::error!("Couldn't delete a song! {}", e);
                DeleteStatus {
                upload_uuid: upload_uuid,
                user_uuid: user_uuid,
                status: false,
            }
        },
        }
    }

    pub async fn get_pool() -> Pool<Postgres> {
        let db = sqlx::PgPool::connect(
            &env::var("DATABASE_URL").expect("Database URL should be reachable when creating user"),
        )
        .await
        .expect("DB should be reachable!");
        db
    }

    pub async fn get_user_by_uuid(uuid: &String) -> Result<User, AuthError> {
        sqlx::query_as::<_, User>("select * from users where uuid = $1")
            .bind(uuid)
            .fetch_one(&get_pool().await)
            .await
            .map_err(|_| AuthError::UUIDError(uuid.to_string()))
    }


}
