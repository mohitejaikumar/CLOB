use std::{error::Error, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use scylla::{client::{execution_profile::ExecutionProfile, session::Session, session_builder::SessionBuilder}, frame::Compression, policies::load_balancing};



pub mod schema;
pub mod scylla_tables;


pub struct ScyllaDb {
    pub session: Session
}


impl ScyllaDb {
    pub async fn create_session(uri: &str) -> Result<ScyllaDb, Box<dyn Error>> {
        let policy = Arc::new(load_balancing::DefaultPolicy::default());
        let profile = ExecutionProfile::builder().load_balancing_policy(policy).build();
        let profile_handle = profile.into_handle();


        let session = SessionBuilder::new()
            .known_node(format!("{}:{}", uri, 9042))
            .known_node(format!("{}:{}", uri, 9043))
            .known_node(format!("{}:{}", uri, 9044))
            .default_execution_profile_handle(profile_handle)
            .compression(Some(Compression::Lz4))
            .build()
            .await?;

        Ok(ScyllaDb { session })
    }

    pub async fn new_user_id(&self) -> Result<i64, Box<dyn Error>> {
        let s = r#"
        SELECT COUNT(*) FROM keyspace_1.user_table;
        "#;
        let res = self.session.query_unpaged(s, &[]).await?;
        let mut temp = res.into_rows_result().unwrap();
        let count = temp.rows::<(i64,)>().unwrap().next().transpose().unwrap().unwrap().0;
        Ok(count + 1)
    }
}


pub fn get_epoch_micros() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros()
}