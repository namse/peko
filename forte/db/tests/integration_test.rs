use db::turso_with_config;

const LOCAL_LIBSQL_URL: &str = "http://127.0.0.1:8080";

fn create_test_db() -> db::Database {
    turso_with_config(LOCAL_LIBSQL_URL.to_string(), String::new())
        .expect("Failed to create test database")
}

#[wstd::test]
async fn test_connection_without_auth() {
    let db = create_test_db();

    let result = db.execute_sql("SELECT 1").await;
    assert!(result.is_ok(), "Should connect without auth token");
}

#[wstd::test]
async fn test_create_table() {
    let db = create_test_db();

    let result = db
        .execute_sql(
            "CREATE TABLE IF NOT EXISTS docs (
                pk TEXT NOT NULL,
                sk TEXT NOT NULL,
                data BLOB,
                PRIMARY KEY (pk, sk)
            )",
        )
        .await;

    assert!(result.is_ok(), "Should create table: {:?}", result.err());
}

#[wstd::test]
async fn test_insert_and_get() {
    let db = create_test_db();

    // Ensure table exists
    db.execute_sql(
        "CREATE TABLE IF NOT EXISTS docs (
            pk TEXT NOT NULL,
            sk TEXT NOT NULL,
            data BLOB,
            PRIMARY KEY (pk, sk)
        )",
    )
    .await
    .expect("Failed to create table");

    let test_pk = "test_pk";
    let test_sk = "test_sk";
    let test_data = b"hello world";

    // Insert data
    db.insert(test_pk, test_sk, test_data)
        .await
        .expect("Failed to insert data");

    // Get data
    let result = db.get(test_pk, test_sk).await.expect("Failed to get data");

    assert!(result.is_some(), "Should return inserted data");
    assert_eq!(result.unwrap().as_ref(), test_data);

    // Cleanup
    db.delete(test_pk, test_sk)
        .await
        .expect("Failed to delete data");
}

#[wstd::test]
async fn test_get_nonexistent() {
    let db = create_test_db();

    // Ensure table exists
    db.execute_sql(
        "CREATE TABLE IF NOT EXISTS docs (
            pk TEXT NOT NULL,
            sk TEXT NOT NULL,
            data BLOB,
            PRIMARY KEY (pk, sk)
        )",
    )
    .await
    .expect("Failed to create table");

    let result = db
        .get("nonexistent_pk", "nonexistent_sk")
        .await
        .expect("Query should succeed");

    assert!(result.is_none(), "Should return None for nonexistent key");
}

#[wstd::test]
async fn test_update_existing() {
    let db = create_test_db();

    // Ensure table exists
    db.execute_sql(
        "CREATE TABLE IF NOT EXISTS docs (
            pk TEXT NOT NULL,
            sk TEXT NOT NULL,
            data BLOB,
            PRIMARY KEY (pk, sk)
        )",
    )
    .await
    .expect("Failed to create table");

    let test_pk = "update_test_pk";
    let test_sk = "update_test_sk";
    let initial_data = b"initial data";
    let updated_data = b"updated data";

    // Insert initial data
    db.insert(test_pk, test_sk, initial_data)
        .await
        .expect("Failed to insert initial data");

    // Update with new data
    db.insert(test_pk, test_sk, updated_data)
        .await
        .expect("Failed to update data");

    // Verify update
    let result = db.get(test_pk, test_sk).await.expect("Failed to get data");

    assert!(result.is_some(), "Should return updated data");
    assert_eq!(result.unwrap().as_ref(), updated_data);

    // Cleanup
    db.delete(test_pk, test_sk)
        .await
        .expect("Failed to delete data");
}
