use database_macros::Queryable;
use anyhow::Context;

#[derive(Queryable)]
struct TestModel {
    pub id: usize,
    pub comments: Option<String>,
    pub test_val: String
}

fn initialize_table(conn: &rusqlite::Connection) -> anyhow::Result<()> {
    conn.execute("CREATE TABLE TestModel (id INT PRIMARY KEY, comments TEXT, test_val TEXT NOT NULL);", [])?;
    Ok(())
}

#[test]
fn get() -> anyhow::Result<()> {
    let manager = r2d2_sqlite::SqliteConnectionManager::memory();
    let pool = r2d2::Pool::new(manager)?;
    let conn: r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> = pool.get()?;

    initialize_table(&conn)?;
    conn.execute("INSERT INTO TestModel (id, comments, test_val) VALUES (1, 'hello', 'test_val'), (2, NULL, 'test2');", [])?;

    let mut single_request = TestModelRequest::default();
    single_request.id = Some(1);

    let test_model = TestModel::get(&conn, single_request)?;
    assert_eq!(test_model.id, 1);
    assert_eq!(test_model.comments, Some("hello".to_string()));
    assert_eq!(test_model.test_val, "test_val");

    return Ok(());
}

#[test]
fn get_many() -> anyhow::Result<()> {
    let manager = r2d2_sqlite::SqliteConnectionManager::memory();
    let pool = r2d2::Pool::new(manager)?;
    let conn: r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> = pool.get()?;

    initialize_table(&conn)?;
    conn.execute("INSERT INTO TestModel (id, comments, test_val) VALUES (1, 'hello', 'test_val'), (2, NULL, 'test2');", [])?;

    let single_request = TestModelRequest::default();

    let test_models = TestModel::get_many(&conn, single_request)?;
    let first_model = test_models.first().context("No first element in test_models")?;
    assert_eq!(first_model.id, 1);
    assert_eq!(first_model.comments, Some("hello".to_string()));
    assert_eq!(first_model.test_val, "test_val");
    let second_model = test_models.last().context("No second element in test_models")?;
    assert_eq!(second_model.id, 2);
    assert_eq!(second_model.comments, None);
    assert_eq!(second_model.test_val, "test2");

    return Ok(());
}

#[test]
fn add() -> anyhow::Result<()> {
    let manager = r2d2_sqlite::SqliteConnectionManager::memory();
    let pool = r2d2::Pool::new(manager)?;
    let conn: r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> = pool.get()?;
    initialize_table(&conn)?;

    let new_test_model = TestModel {
        id: 1,
        comments: Some(String::from("This is a comment")),
        test_val: String::from("test_val")
    };

    new_test_model.add(&conn)?;

    conn.query_row("SELECT * FROM TestModel", [], |row| {
        let id: usize = row.get("id")?;
        assert_eq!(id, 1);
        let comments: Option<String> = row.get("comments")?;
        assert_eq!(comments, Some("This is a comment".to_string()));
        let test_val: String = row.get("test_val")?;
        assert_eq!(test_val, "test_val".to_string());
        Ok(())
    })?;

    return Ok(());
}

#[test]
fn update() -> anyhow::Result<()> {
    let manager = r2d2_sqlite::SqliteConnectionManager::memory();
    let pool = r2d2::Pool::new(manager)?;
    let conn: r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> = pool.get()?;
    initialize_table(&conn)?;
    conn.execute("INSERT INTO TestModel (id, comments, test_val) VALUES (1, 'This is a comment', 'test_val');", [])?;

    let new_test_model = TestModel {
        id: 1,
        comments: Some(String::from("comments")),
        test_val: String::from("new_test_val")
    };

    new_test_model.update(&conn)?;


    conn.query_row("SELECT * FROM TestModel", [], |row| {
        let id: usize = row.get("id")?;
        assert_eq!(id, 1);
        let comments: Option<String> = row.get("comments")?;
        assert_eq!(comments, Some("comments".to_string()));
        let test_val: String = row.get("test_val")?;
        assert_eq!(test_val, "new_test_val".to_string());
        Ok(())
    })?;

    return Ok(());
}
