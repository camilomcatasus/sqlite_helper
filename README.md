# sqlite_helper
Helper macro crate for various sqlite drivers in rust

## Supported sqlite crates
- rusqlite
- libsql-client

## How to use
Adding the derive macro Queryable for rusqlite or LibSqlQueryable for libsql_client to a struct will implement get, get_many, add, and update functions for each of those struct.

### get and get_many
The rusqlite implementations take a rusqlite::Connection struct and a request struct. The name of the request struct depends on the name of the struct, which will have the format {struct_name}Request.
The libsql_client implementations take a libsql_client::Client struct, the rest should be the same as the rusqlite implementation.

The get and get_many will look for rows in the table that matches the struct name who columns match the fields that have Some() in the request struct. 

### add
Will do a simple add of the struct to the table.

### update
Assumes the id to be the first field in the struct
TODO: Have a macro attribute let you denote which field is the id field.

Will update the rest of the columns with the fields of the struct this was called from.

## Example

```rust
#[derive(Queryable)]
struct Example {
  pub id: usize,
  pub example_text: String
}

#[derive(LibSqlQueryable)]
struct Example2 {
  pub id: usize,
  pub example_text: String
}
```

## Requirements (other than rusqlite or libsql_client)

- anyhow

