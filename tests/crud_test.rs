use diesel::dsl::{count_star, sum};
use dotenvy;

use diesel::{insert_into, prelude::*};
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::{embed_migrations, MigrationHarness};

use self::models::*;
use new_tax_account_backend::*;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

fn get_connection() -> SqliteConnection {
    dotenvy::from_filename(".env.test").expect("failed to read .env file");
    let mut connection = establish_connection();
    connection.run_pending_migrations(MIGRATIONS).unwrap();
    connection
}

fn insert_post_simple(
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

fn insert_post_full(
    connection: &mut SqliteConnection,
    title: &str,
    body: &str,
    category_id: Option<i32>,
    author: Option<&str>,
    published: bool,
    good_count: i32,
) -> QueryResult<usize> {
    use self::schema::posts::dsl as posts;
    let new_record = Post {
        title: String::from(title),
        body: String::from(body),
        category_id,
        author: author.map(String::from),
        published,
        good_count,
        ..Default::default()
    };
    insert_into(posts::posts)
        .values(&new_record)
        .execute(connection)
}

#[test]
fn test_insert_simple() {
    let mut connection = get_connection();
    let result = insert_post_simple(&mut connection, "title1", "body1", true);
    assert!(result.is_ok());
    assert!(result.unwrap() == 1);
}

#[test]
fn test_insert_by_struct() {
    use self::schema::posts::dsl as posts;

    let new_record = Post {
        id: None,
        title: "title1".to_string(),
        body: "body1".to_string(),
        category_id: None,
        ..Default::default()
    };

    let mut connection = get_connection();
    let result = insert_into(posts::posts)
        .values(&new_record)
        .execute(&mut connection);

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);
}

#[test]
fn test_select_all_columns() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_simple(&mut connection, "title1", "body1", true);
    let results = posts::posts
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
fn test_select_specified_columns() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_simple(&mut connection, "title1", "body1", true);
    let results = posts::posts
        .select((posts::title, posts::body))
        .load::<(String, String)>(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0).unwrap();
    assert!(head.0 == "title1");
    assert!(head.1 == "body1");
}

#[test]
fn test_select_into_struct() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_simple(&mut connection, "title1", "body1", true);

    #[derive(Queryable, Debug)]
    struct PostTitleBody {
        id: Option<i32>,
        title: String,
        body: String,
    }

    let results = posts::posts
        .select((posts::id, posts::title, posts::body))
        .load::<PostTitleBody>(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0).unwrap();
    assert!(head.id.is_some());
    assert!(head.title == "title1");
    assert!(head.body == "body1");
}

#[test]
fn test_select_by_filter() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_simple(&mut connection, "title1", "body1", true);
    let _ = insert_post_simple(&mut connection, "title2", "body2", false);

    let results = posts::posts
        .filter(posts::title.eq("title2"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0);
    assert!(head.is_some());

    let head = head.unwrap();
    assert!(head.title == "title2");
    assert!(head.body == "body2");
    assert!(!head.published);
}

#[test]
fn test_select_by_multiple_filters() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_simple(&mut connection, "title1", "body1", true);
    let _ = insert_post_simple(&mut connection, "title2", "body2", false);

    let results = posts::posts
        .filter(posts::title.eq("title2"))
        .filter(posts::body.eq("body1"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 0);
}

#[test]
#[rustfmt::skip]
fn test_select_by_complex_filters() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_full(&mut connection,"title1", "body1", None, None, true, 100);
    let _ = insert_post_full(&mut connection,"title2", "body2", Some(1), Some("John"), true, 20);
    let _ = insert_post_full(&mut connection,"title3", "body3", Some(1), None, true, 40);
    let _ = insert_post_full(&mut connection,"title4", "body4", None, None, true, 5);
    let _ = insert_post_full(&mut connection,"title5", "body5", Some(2), Some("John"), false, 0);
    let _ = insert_post_full(&mut connection,"title6", "body6", Some(2), Some("Bob"), false, 0);
    let _ = insert_post_full(&mut connection,"title7", "body7", Some(2), None, true, 10);
    let _ = insert_post_full(&mut connection,"title8", "body8", None, None, true, 15);
    let _ = insert_post_full(&mut connection,"title9", "body9", Some(3), Some("Alice"), true, 200);

    let results = posts::posts
        .filter(posts::published.eq(true).and(posts::good_count.gt(50)))
        .or_filter(posts::published.eq(false).and(posts::author.eq("Bob")))
        .select(Post::as_select())
        .order_by(posts::good_count.desc())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 3);

    let (first, second, third) = (
        results.get(0).unwrap(),
        results.get(1).unwrap(),
        results.get(2).unwrap(),
    );
    assert!(first.title == "title9");
    assert!(second.title == "title1");
    assert!(third.title == "title6");
}

#[test]
#[rustfmt::skip]
fn test_select_grouping() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_full(&mut connection,"title1", "body1", None, None, true, 100);
    let _ = insert_post_full(&mut connection,"title2", "body2", Some(1), Some("John"), true, 20);
    let _ = insert_post_full(&mut connection,"title3", "body3", Some(1), None, true, 40);
    let _ = insert_post_full(&mut connection,"title4", "body4", None, None, true, 5);
    let _ = insert_post_full(&mut connection,"title5", "body5", Some(2), Some("John"), true, 50);
    let _ = insert_post_full(&mut connection,"title6", "body6", Some(2), Some("John"), false, 0);
    let _ = insert_post_full(&mut connection,"title7", "body7", Some(2), None, true, 10);
    let _ = insert_post_full(&mut connection,"title8", "body8", None, None, true, 15);
    let _ = insert_post_full(&mut connection,"title9", "body9", Some(3), Some("Alice"), true, 200);

    let results = posts::posts
        .filter(posts::published.eq(true).and(posts::author.is_not_null()))
        .group_by(posts::author)
        .select((posts::author, count_star(), sum(posts::good_count)))
        .order_by(posts::good_count.desc())
        .load::<(Option<String>, i64, Option<i64>)>(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 2);

    let (first, second) = (
        results.get(0).unwrap(),
        results.get(1).unwrap(),
    );
    assert!(first.0 == Some("Alice".to_string()));
    assert!(first.1 == 1);
    assert!(first.2 == Some(200));
    assert!(second.0 == Some("John".to_string()));
    assert!(second.1 == 2);
    assert!(second.2 == Some(70));
}
