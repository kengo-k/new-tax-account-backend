use std::vec;

use diesel::dsl::{count_star, sum};
use diesel::sqlite::Sqlite;
use dotenvy;

use diesel::{debug_query, insert_into, prelude::*};
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
fn test_select_with_subquery() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_simple(&mut connection, "title1", "body1", true);

    let subquery = posts::posts
        .select(posts::id)
        .filter(posts::title.eq("title1"))
        .into_boxed();

    let query = posts::posts
        .select((posts::title, posts::body))
        .filter(posts::id.eq_any(subquery));

    println!("Debug query: {:?}", debug_query::<Sqlite, _>(&query));

    let results = query
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

#[test]
fn test_update() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_simple(&mut connection, "title1", "body1", true);
    let _ = (&mut connection, "title2", "body2", false);

    let result = diesel::update(posts::posts.filter(posts::title.eq("title1")))
        .set(posts::body.eq("new body1"))
        .execute(&mut connection);

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);

    let results = posts::posts
        .filter(posts::title.eq("title1"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0);
    assert!(head.is_some());

    let head = head.unwrap();
    assert!(head.title == "title1");
    assert!(head.body == "new body1");
    assert!(head.published);
}

#[test]
fn test_update_by_struct() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_full(
        &mut connection,
        "title1",
        "body1",
        Some(1),
        Some("john"),
        true,
        100,
    );

    #[derive(AsChangeset)]
    #[diesel(table_name = crate::schema::posts)]
    struct UpdatePostAttributes {
        category_id: Option<i32>,
        author: Option<String>,
    }

    let new_attributes = UpdatePostAttributes {
        category_id: Some(2),
        author: Some("bob".to_string()),
    };

    let result = diesel::update(posts::posts.filter(posts::title.eq("title1")))
        .set(&new_attributes)
        .execute(&mut connection);

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);

    let results = posts::posts
        .filter(posts::title.eq("title1"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0);
    assert!(head.is_some());

    let head = head.unwrap();
    assert_eq!(head.title, "title1");
    assert_eq!(head.category_id, Some(2));
    assert_eq!(head.author, Some("bob".to_string()));

    let new_attributes = UpdatePostAttributes {
        category_id: Some(3),
        // Fields specified as 'None' are not included in the update target.
        author: None,
    };

    let result = diesel::update(posts::posts.filter(posts::title.eq("title1")))
        .set(&new_attributes)
        .execute(&mut connection);

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);

    let results = posts::posts
        .filter(posts::title.eq("title1"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0);
    assert!(head.is_some());

    let head = head.unwrap();
    assert_eq!(head.title, "title1");
    assert_eq!(head.category_id, Some(3));
    assert_eq!(head.author, Some("bob".to_string()));
}

#[test]
fn test_update_by_struct_none_as_null() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_full(
        &mut connection,
        "title1",
        "body1",
        Some(1),
        Some("john"),
        true,
        100,
    );

    #[derive(AsChangeset)]
    #[diesel(table_name = crate::schema::posts)]
    #[changeset_options(treat_none_as_null = "true")]
    struct UpdatePostAttributes {
        category_id: Option<i32>,
        author: Option<String>,
    }

    let new_attributes = UpdatePostAttributes {
        category_id: Some(2),
        author: Some("bob".to_string()),
    };

    let result = diesel::update(posts::posts.filter(posts::title.eq("title1")))
        .set(&new_attributes)
        .execute(&mut connection);

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);

    let results = posts::posts
        .filter(posts::title.eq("title1"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0);
    assert!(head.is_some());

    let head = head.unwrap();
    assert_eq!(head.title, "title1");
    assert_eq!(head.category_id, Some(2));
    assert_eq!(head.author, Some("bob".to_string()));

    let new_attributes = UpdatePostAttributes {
        category_id: Some(3),
        // Fields specified as 'None' is treated as null.
        author: None,
    };

    let result = diesel::update(posts::posts.filter(posts::title.eq("title1")))
        .set(&new_attributes)
        .execute(&mut connection);

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);

    let results = posts::posts
        .filter(posts::title.eq("title1"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0);
    assert!(head.is_some());

    let head = head.unwrap();
    assert_eq!(head.title, "title1");
    assert_eq!(head.category_id, Some(3));
    assert_eq!(head.author, None);
}

#[test]
fn test_update_by_struct_flexibly() {
    use self::schema::posts::dsl as posts;

    let mut connection = get_connection();
    let _ = insert_post_full(
        &mut connection,
        "title1",
        "body1",
        Some(1),
        Some("john"),
        true,
        100,
    );

    #[derive(AsChangeset)]
    #[diesel(table_name = crate::schema::posts)]
    struct UpdatePostAttributes {
        title: Option<String>,
        body: Option<String>,
        category_id: Option<Option<i32>>,
        author: Option<Option<String>>,
    }

    let new_attributes = UpdatePostAttributes {
        title: Some("new title1".to_string()),
        body: None,
        category_id: Some(Some(2)),
        author: Some(Some("bob".to_string())),
    };

    let result = diesel::update(posts::posts.filter(posts::title.eq("title1")))
        .set(&new_attributes)
        .execute(&mut connection);

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);

    let results = posts::posts
        .filter(posts::title.eq("new title1"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0);
    assert!(head.is_some());

    let head = head.unwrap();
    assert_eq!(head.title, "new title1");
    assert_eq!(head.body, "body1");
    assert_eq!(head.category_id, Some(2));
    assert_eq!(head.author, Some("bob".to_string()));

    let new_attributes = UpdatePostAttributes {
        title: None,
        body: Some("new body1".to_string()),
        category_id: None,
        author: Some(None),
    };

    let result = diesel::update(posts::posts.filter(posts::title.eq("new title1")))
        .set(&new_attributes)
        .execute(&mut connection);

    assert!(result.is_ok());
    assert!(result.unwrap() == 1);

    let results = posts::posts
        .filter(posts::title.eq("new title1"))
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 1);

    let head = results.get(0);
    assert!(head.is_some());

    let head = head.unwrap();
    assert_eq!(head.title, "new title1");
    assert_eq!(head.body, "new body1");
    assert_eq!(head.category_id, Some(2));
    assert_eq!(head.author, None);
}

#[test]
#[rustfmt::skip]
fn test_delete() {
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

    let query = diesel::delete(posts::posts.filter(posts::category_id.eq(2)));
    println!("Debug query: {:?}", debug_query(&query));

    query
        .execute(&mut connection)
        .expect("Error deleting posts");

    let results = posts::posts
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 6);
}

#[test]
#[rustfmt::skip]
fn test_delete_in() {
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

    let query = diesel::delete(posts::posts.filter(posts::category_id.eq_any(vec![1,2,3])));
    println!("Debug query: {:?}", debug_query::<Sqlite, _>(&query));

    query
        .execute(&mut connection)
        .expect("Error deleting posts");

    let results = posts::posts
        .select(Post::as_select())
        .load(&mut connection)
        .expect("Error loading posts");

    assert!(results.len() == 3);
}
