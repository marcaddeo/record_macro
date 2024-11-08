#![feature(trace_macros)]
#![allow(dead_code)]
#![allow(unused_variables)]

// trace_macros!(true);

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::RunQueryDsl;
use record_macro::zomg;

pub mod schema {
    use diesel::table;

    table! {
        user (id) {
            id -> Integer,
            name -> Text,
        }
    }

    table! {
        post (id) {
            id -> Integer,
            user_id -> Integer,
            content -> Text,
        }
    }

    table! {
        comment (id) {
            id -> Integer,
            user_id -> Integer,
            post_id -> Integer,
            content -> Text,
        }
    }
}

pub type Connection = SyncConnectionWrapper<SqliteConnection>;

// Just a newtype, not part of macro.
// struct AvatarUrl(String);
//
// struct Profile {
//     id: i32,
//     avatar: Option<AvatarUrl>,
// }
//
// struct ProfileRecord {
//     id: i32,
//     avatar: Option<AvatarUrl>,
// }
//
// struct NewProfileRecord<'a> {
//     avatar: Option<&'a AvatarUrl>,
// }
//
// struct User {
//     id: i32,
//     name: String,
//     profile: Profile,
// }
//
// struct UserRecord {
//     id: i32,
//     name: String,
//     profile_id: i32,
// }
//
// struct NewUserRecord<'a> {
//     name: &'a str,
//     profile_id: i32,
// }

// struct Post {
//     id: i32,
//     author: User,
//     content: String,
// }
//
// impl Post {
//     pub fn new_record<'a>(user_id: i32, content: &'a str) -> NewPostRecord<'a> {
//         NewPostRecord { user_id, content }
//     }
//
//     pub fn or_new_record<'a>(user: &'a User, content: &'a str) -> NewPostRecord<'a> {
//         todo!()
//     }
//
//     // @TODO this should possibly be a separate #[derive(FromRecord)]
//     pub async fn from_record(record: &PostRecord, conn: &mut Connection) -> Self {
//         todo!()
//     }
//
//     pub async fn from_records<'a>(
//         records: impl IntoIterator<Item = &'a PostRecord>,
//         conn: &'a mut Connection,
//     ) -> Vec<Self> {
//         todo!()
//     }
// }
//
// struct PostRecord {
//     id: i32,
//     user_id: i32,
//     content: String,
// }
//
// #[derive(Debug)]
// struct NewPostRecord<'a> {
//     user_id: i32,
//     content: &'a str,
// }
//
// impl<'a> NewPostRecord<'a> {
//     pub fn new(user_id: i32, content: &str) -> Self {
//         todo!()
//     }
//
//     pub async fn create(&self, conn: &mut Connection) -> QueryResult<PostRecord> {
//         todo!()
//     }
// }

#[test]
fn it_works() {
    zomg!(
        #[derive(Clone, Debug, Default, Queryable, Identifiable, Selectable, Insertable)]
        #[diesel(table_name = crate::schema::user)]
        pub struct User {
            id: i32,
            name: String,
        }
    );

    zomg!(
        #[derive(Debug, Default, Queryable, Identifiable, Selectable, Insertable, Associations)]
        #[diesel(table_name = crate::schema::post)]
        #[diesel(belongs_to(UserRecord, foreign_key = user_id))]
        pub struct Post {
            id: i32,
            user: Related<User>,
            content: String,
        }
    );
    zomg!(
        #[derive(Debug, Default, Queryable, Identifiable, Selectable, Insertable, Associations)]
        #[diesel(table_name = crate::schema::comment)]
        #[diesel(belongs_to(UserRecord, foreign_key = user_id))]
        #[diesel(belongs_to(PostRecord, foreign_key = post_id))]
        pub struct Comment {
            id: i32,
            user: Related<User>,
            post: Related<Post>,
            content: String,
        }
    );
    let record = Post::new_record(123, "some content");

    dbg!(record);
}
