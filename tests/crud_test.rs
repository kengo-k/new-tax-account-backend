use std::process::Command;

use self::models::*;
use diesel::{insert_into, prelude::*};
use new_tax_account_backend::*;

#[test]
fn test_crud() {
    use self::schema::posts::dsl::*;

    Command::new("diesel")
        .arg("database")
        .arg("reset")
        .output()
        .expect("failed to execute process");

    let connection = &mut establish_connection();

    let results = posts
        .select(Post::as_select())
        .load(connection)
        .expect("Error loading posts");

    assert!(results.is_empty());

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

    let results = posts
        .select(Post::as_select())
        .load(connection)
        .expect("Error loading posts");

    assert!(results.len() == 2);

    let results = posts
        .filter(published.eq(false))
        .select(Post::as_select())
        .load(connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);
}
