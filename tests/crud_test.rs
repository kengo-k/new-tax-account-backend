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

    insert_into(posts)
        .values((title.eq("test title"), body.eq("tes"), published.eq(true)))
        .execute(connection)
        .unwrap();

    let results = posts
        .filter(published.eq(true))
        .select(Post::as_select())
        .load(connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!(
            "id:{:?}, title:{}, body: {}, published: {}",
            post.id, post.title, post.body, post.published
        );
    }
}
