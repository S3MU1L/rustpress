#[macro_export]
macro_rules! log_err {
    // Usage: log_err!(&self.pool, data);
    ($pool:expr, $params:expr) => {{
        let pool_clone = $pool.clone();
        let location = format!("{}:{}", file!(), line!());

        let params_json = ::serde_json::to_value($params)
            .unwrap_or(::serde_json::Value::Null);

        ::tokio::spawn(async move {
            let _ = ::sqlx::query(
                r#"
                    INSERT INTO error_logs (location, parameters)
                    VALUES ($1, $2)
                    "#,
            )
            .bind(location)
            .bind(params_json)
            .execute(&pool_clone)
            .await;
        });
    }};
}
