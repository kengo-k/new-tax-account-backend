use std::process::Command;

use self::models::*;
use diesel::{insert_into, prelude::*};
use new_tax_account_backend::*;

fn init() {
    Command::new("diesel")
        .arg("database")
        .arg("reset")
        .output()
        .expect("failed to execute process");
}

fn get_connection() -> SqliteConnection {
    establish_connection()
}

fn insert_post(
    connection: &mut SqliteConnection,
    title: &str,
    body: &str,
    published: bool,
) -> QueryResult<usize> {
    use self::schema::posts::dsl as posts;
    let result = insert_into(posts::posts)
        .values((
            posts::title.eq(title),
            posts::body.eq(body),
            posts::published.eq(published),
        ))
        .execute(connection);
    result
}

#[test]
fn test_insert() {
    init();
    let mut connection = get_connection();
    let result = insert_post(&mut connection, "title1", "body1", true);
    assert!(result.is_ok());
    assert!(result.unwrap() == 1);
}

#[test]
fn test_select() {
    use self::schema::posts::dsl::*;

    init();
    let mut connection = get_connection();
    let _ = insert_post(&mut connection, "title1", "body1", true);

    // Confirm that there is no data present.
    let results = posts
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");
    assert!(results.len() == 1);
    let head = results.get(0);
    assert!(head.is_some());
    let head = head.unwrap();
    assert!(head.title == "title1");
    assert!(head.body == "body1");
    assert!(head.published);
}

#[test]
fn test_basic_crud() {
    use self::schema::posts::dsl::*;

    init();

    let mut connection = get_connection();

    // Confirm that there is no data present.
    let results = posts
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");
    assert!(results.is_empty());

    // Insert two rows.
    insert_into(posts)
        .values((
            title.eq("test title1"),
            body.eq("test body1"),
            published.eq(true),
        ))
        .execute(&mut connection)
        .unwrap();

    insert_into(posts)
        .values((
            title.eq("test title2"),
            body.eq("test body2"),
            published.eq(false),
        ))
        .execute(&mut connection)
        .unwrap();

    // Confirm that there are two rows present.
    let results = posts
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");
    assert!(results.len() == 2);

    // Search by specifying search criteria.
    let results = posts
        .filter(published.eq(false))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");
    assert!(results.len() == 1);

    // Test that the retrieved record values are correct.
    let head = results.get(0);
    assert!(head.is_some());
    let head = head.unwrap();
    assert!(head.title == "test title2");
    assert!(head.body == "test body2");
    assert!(head.published == false);

    // Select only the specified columns by tuple.
    let results = posts
        .filter(published.eq(true))
        .select((title, body))
        .load::<(String, String)>(&mut connection)
        .expect("Error loading posts");
    assert!(results.len() == 1);
    let head = results.get(0).unwrap();
    assert!(head.0 == "test title1");
    assert!(head.1 == "test body1");

    // Select only the specified columns by struct.
    #[derive(Queryable, Debug)]
    struct PostTitleBody {
        id: Option<i32>,
        title: String,
        body: String,
    }
    let results = posts
        .filter(published.eq(true))
        .select((id, title, body))
        .load::<PostTitleBody>(&mut connection)
        .expect("Error loading posts");
    assert!(results.len() == 1);
    let head = results.get(0).unwrap();
    assert!(head.title == "test title1");
    assert!(head.body == "test body1");

    let update_result = diesel::update(posts.filter(id.eq(head.id)))
        .set((title.eq("new title"), body.eq("new body")))
        .execute(&mut connection);

    assert!(update_result.is_ok());
    assert!(update_result.unwrap() == 1);

    struct UpdatePost {
        title: String,
        body: String,
    }
}
