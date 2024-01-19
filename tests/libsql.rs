use database_macros::LibSqlQueryable;
use serde::{Serialize, Deserialize};
use anyhow::Context;

#[derive(LibSqlQueryable, Serialize, Deserialize)]
pub struct LibSqlTest {
    #[primary]
    pub id: i64,
    pub test: String
}

#[tokio::test]
async fn get() -> anyhow::Result<()> {
    let db = libsql_client::Client::in_memory().unwrap();
    db.execute("CREATE TABLE LibSqlTest(id INT PRIMARY KEY, test TEXT NOT NULL);").await?; 
    db.execute("INSERT INTO LibSqlTest (id, test) VALUES (1, \"test\");").await?;
    let mut req = LibSqlTestRequest::default();
    req.id = Some(1);
    let test_struct = LibSqlTest::get(&db, req).await?;
    
    assert!(test_struct.id == 1);
    assert!(test_struct.test == "test");


    Ok(())
}

#[tokio::test]
async fn get_many() -> anyhow::Result<()> {
    let db = libsql_client::Client::in_memory().unwrap();
    db.execute("CREATE TABLE LibSqlTest(id INT PRIMARY KEY, test TEXT NOT NULL);").await?; 
    db.execute("INSERT INTO LibSqlTest (id, test) VALUES (1, \"test\"), (2, \"second_test\");").await?;
    let req = LibSqlTestRequest::default();
    let test_structs = LibSqlTest::get_many(&db, req).await?;
    let first = test_structs.first().context("No rows")?;
    let second = test_structs.last().context("Only one row")?;
    assert!(first.id == 1);
    assert!(first.test == "test");
    assert!(second.id == 2);
    assert!(second.test == "second_test");

    Ok(())
}


#[tokio::test]
async fn add() -> anyhow::Result<()> {
    let db = libsql_client::Client::in_memory().unwrap();
    db.execute("CREATE TABLE LibSqlTest(id INT PRIMARY KEY, test TEXT NOT NULL);").await?;
    
    let test_struct = LibSqlTest {
        id: 1,
        test: String::from("TEST")
    };

    let result = test_struct.add(&db).await?;
    assert!(result == 1);
    
    let confirm_req = LibSqlTestRequest {
        id: Some(1), test: None
    };
    let confirm_struct = LibSqlTest::get(&db, confirm_req).await?;

    assert!(confirm_struct.test == "TEST");
    Ok(())
}

#[derive(LibSqlQueryable, Serialize, Deserialize)]
struct AutoIncrementTest {
    #[primary(autoincrement)]
    pub id: usize,
    pub test: String
}


#[derive(LibSqlQueryable, Serialize, Deserialize)]
struct ComplexUpdateTest {
    #[primary]
    pub id1: usize,
    #[primary]
    pub id2: usize,
    pub test: String
}

#[tokio::test]
async fn add_autoincrement() -> anyhow::Result<()> {
    let db = libsql_client::Client::in_memory().unwrap();
    db.execute("CREATE TABLE AutoIncrementTest(id INTEGER PRIMARY KEY AUTOINCREMENT, test TEXT NOT NULL);").await?;

    let test_struct = AutoIncrementTest {
        id: 0,
        test: "test".to_string()
    };

    let result = test_struct.add(&db).await?;
    assert!(result == 1);
    let confirm_req = AutoIncrementTestRequest {
        id: Some(1),
        test: None,
    };

    let confirm_struct = AutoIncrementTest::get(&db, confirm_req).await?;

    assert!(confirm_struct.test == "test");

    Ok(())
}

#[tokio::test]
async fn update() -> anyhow::Result<()> {
    let db = libsql_client::Client::in_memory().unwrap();
    db.execute("CREATE TABLE LibSqlTest(id INT PRIMARY KEY, test TEXT NOT NULL);").await?;
    db.execute("INSERT INTO LibSqlTest(id, test) VALUES (1, \"test\");").await?;
    let test_struct = LibSqlTest {
        id: 1,
        test: String::from("THE QUICK BROWN FOX")
    };

    let result = test_struct.update(&db).await?;
    assert!(result == 1);
    
    let confirm_req = LibSqlTestRequest {
        id: Some(1), test: None
    };
    let confirm_struct = LibSqlTest::get(&db, confirm_req).await?;

    assert!(confirm_struct.test == "THE QUICK BROWN FOX");
    Ok(())
}

#[tokio::test]
async fn complex_update() -> anyhow::Result<()> {
    let db = libsql_client::Client::in_memory().unwrap();
    db.execute("CREATE TABLE ComplexUpdateTest(id1 INT, id2 INT, test TEXT NOT NULL, PRIMARY KEY(id1, id2));").await?;
    db.execute("INSERT INTO ComplexUpdateTest(id1, id2, test) VALUES (1, 1, \"test\");").await?;
    db.execute("INSERT INTO ComplexUpdateTest(id1, id2, test) VALUES (2, 2, \"test\");").await?;

    let test_struct = ComplexUpdateTest {
        id1: 1,
        id2: 1,
        test: String::from("THE QUICK BROWN FOX")
    };

    let result = test_struct.update(&db).await?;
    assert!(result == 1);
    
    let confirm_req = ComplexUpdateTestRequest {
        id1: Some(1),id2: Some(1), test: None
    };
    let confirm_struct = ComplexUpdateTest::get(&db, confirm_req).await?;

    assert!(confirm_struct.test == "THE QUICK BROWN FOX");
    Ok(())
}
