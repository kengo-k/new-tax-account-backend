use std::process::Command;

use self::models::*;
use diesel::{insert_into, prelude::*};
use new_tax_account_backend::*;

fn setup() {
    Command::new("diesel")
        .arg("database")
        .arg("reset")
        .output()
        .expect("failed to execute process");
}

#[test]
fn test_basic_crud() {
    use self::schema::posts::dsl::*;

    setup();

    let connection = &mut establish_connection();

    // Confirm that there is no data present.
    let results = posts
        .select(Post::as_select())
        .load(connection)
        .expect("Error loading posts");
    assert!(results.is_empty());

    // Insert two rows.
    insert_into(posts)
        .values((
            title.eq("test title1"),
            body.eq("test body1"),
            published.eq(true),
        ))
        .execute(connection)
        .unwrap();

    insert_into(posts)
        .values((
            title.eq("test title2"),
            body.eq("test body2"),
            published.eq(false),
        ))
        .execute(connection)
        .unwrap();

    // Confirm that there are two rows present.
    let results = posts
        .select(Post::as_select())
        .load(connection)
        .expect("Error loading posts");
    assert!(results.len() == 2);

    // Search by specifying search criteria.
    let results = posts
        .filter(published.eq(false))
        .select(Post::as_select())
        .load(connection)
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
        .load::<(String, String)>(connection)
        .expect("Error loading posts");
    assert!(results.len() == 1);
    let head = results.get(0).unwrap();
    assert!(head.0 == "test title1");
    assert!(head.1 == "test body1");

    // Select only the specified columns by struct.
    #[derive(Queryable, Debug)]
    struct PostTitleBody {
        title: String,
        body: String,
    }
    let results = posts
        .filter(published.eq(true))
        .select((title, body))
        .load::<PostTitleBody>(connection)
        .expect("Error loading posts");
    assert!(results.len() == 1);
    let head = results.get(0).unwrap();
    assert!(head.title == "test title1");
    assert!(head.body == "test body1");
}
