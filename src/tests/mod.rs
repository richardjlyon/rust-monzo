#[cfg(test)]
pub mod test {
    use crate::client::Monzo;
    use crate::model::DatabasePool;
    use crate::telemetry::{get_subscriber, init_subscriber};
    use once_cell::sync::Lazy;
    use temp_dir::TempDir;

    // Ensure that the `tracing` stack is only initialised once using `once_cell`
    static TRACING: Lazy<()> = Lazy::new(|| {
        let default_filter_level = "info".to_string();
        let subscriber_name = "test".to_string();
        // We cannot assign the output of `get_subscriber` to a variable based on the
        // value TEST_LOG` because the sink is part of the type returned by
        // `get_subscriber`, therefore they are not the same type. We could work around
        // it, but this is the most straight-forward way of moving forward.
        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            let _ = init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            let _ = init_subscriber(subscriber);
        };
    });

    /// Create ephemeral test db. Folder is deleted when the TempDir goes out of scope.
    pub async fn test_db() -> (DatabasePool, TempDir) {
        use crate::model::DatabasePool;

        Lazy::force(&TRACING);

        let dir = temp_dir::TempDir::with_prefix("monzo-test").unwrap();
        let db_path = dir.path().join("dev.db?mode=rwc");

        let pool = DatabasePool::new(db_path.to_str().unwrap(), 1)
            .await
            .unwrap();

        let _ = pool
            .seed_initial_data()
            .await
            .expect("Failed to seed initial data");

        (pool, dir)
    }

    pub fn get_client() -> Monzo {
        match Monzo::new() {
            Ok(client) => client,
            Err(e) => panic!("Error creating client: {e}"),
        }
    }
}
