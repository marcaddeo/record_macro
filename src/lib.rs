#![feature(trace_macros)]
#![allow(dead_code)]
#![allow(unused_variables)]

trace_macros!(true);

use defile::defile;
use paste::paste;

struct Zomg<T>(T);

#[macro_export]
macro_rules! zomg {
    // Done, generate struct.
    (@new_record () -> { $pub:vis $name:ident $(($field:ident : $type:ty))* }) => {
        paste! {
            $pub struct [<New $name Record>]<'a> {
                $($field : $type),*
            }
        }
    };

    // Convert String fields to &'a str.
    (@new_record ($field:ident : String $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        zomg!(@new_record ($($($rest)*)?) -> { $($output)* ($field : &'a str) });
    };

    // Remove id field.
    (@new_record (id : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        zomg!(@new_record ($($($rest)*)?) -> { $($output)* });
    };

    // Iterate over struct fields.
    (@new_record ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        defile! {
            zomg!(@@new_record ($($($rest)*)?) -> { $($output)* ($field : $type) });
        }
    };

    // NewRecord entrypoint.
    (@new_record $pub:vis $name:ident ($($rest:tt)*)) => {
        zomg!(@new_record ($($rest)*) -> { $pub $name });
    };

    // Done, generate struct.
    (@record () -> { $(#[$attr:meta])* $pub:vis $name:ident $(($field:ident : $type:ty))* }) => {
        paste! {
            $(#[$attr])*
            $pub struct [<$name Record>] {
                $($field : $type),*
            }
        }

        zomg!(@new_record $pub $name ($($field : $type),*));
    };

    // Replace relation fields with foreign key.
    (@record ($field:ident : Zomg<$type:ty> $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        paste! {
            zomg!(@record ($($($rest)*)?) -> { $($output)* ([<$field _id>] : i32) });
        }
    };

    // Iterate over struct fields.
    (@record ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        zomg!(@record ($($($rest)*)?) -> { $($output)* ($field : $type) });
    };

    // Record entrypoint.
    (@record $(#[$attr:meta])* $pub:vis $name:ident ($($rest:tt)*)) => {
        zomg!(@record ($($rest)*) -> { $(#[$attr])* $pub $name });
    };

    // Done, generate struct.
    (@model () -> { $pub:vis $name:ident $(($field:ident : $type:ty))* }) => {
        $pub struct $name {
            $($field : $type),*
        }
    };

    // Strip out relation marker.
    (@model ($field:ident : Zomg<$type:ty> $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        zomg!(@model ($($($rest)*)?) -> { $($output)* ($field : $type) });
    };

    // Iterate over struct fields.
    (@model ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        zomg!(@model ($($($rest)*)?) -> { $($output)* ($field : $type) });
    };

    // Model entrypoint.
    (@model $pub:vis $name:ident ($($rest:tt)*)) => {
        zomg!(@model ($($rest)*) -> { $pub $name });
    };

    // Main entrypoint.
    ($(#[$attr:meta])* $pub:vis struct $name:ident { $($fields:tt)* } ) => {
        zomg!(@model $pub $name ($($fields)*));
        zomg!(@record $(#[$attr])* $pub $name ($($fields)*));
    };
}

zomg!(
    #[derive(Debug)]
    pub struct Comment {
        some_post: Zomg<Post>,
        id: i32,
        post: Zomg<Post>,
        content: String,
        other_post: Zomg<Post>,
    }
);

type Connection = ();
type QueryResult<T> = Result<T, ()>;

// Just a newtype, not part of macro.
struct AvatarUrl(String);

struct Profile {
    id: i32,
    avatar: Option<AvatarUrl>,
}

struct ProfileRecord {
    id: i32,
    avatar: Option<AvatarUrl>,
}

struct NewProfileRecord<'a> {
    avatar: Option<&'a AvatarUrl>,
}

struct User {
    id: i32,
    name: String,
    profile: Profile,
}

struct UserRecord {
    id: i32,
    name: String,
    profile_id: i32,
}

struct NewUserRecord<'a> {
    name: &'a str,
    profile_id: i32,
}

struct Post {
    id: i32,
    author: User,
    content: String,
}

impl Post {
    pub fn new_record(user_id: i32, content: &str) -> NewPostRecord {
        todo!()
    }

    pub fn or_new_record<'a>(user: &'a User, content: &'a str) -> NewPostRecord<'a> {
        todo!()
    }

    // @TODO this should possibly be a separate #[derive(FromRecord)]
    pub async fn from_record(record: &PostRecord, conn: &mut Connection) -> Self {
        todo!()
    }

    pub async fn from_records<'a>(
        records: impl IntoIterator<Item = &'a PostRecord>,
        conn: &'a mut Connection,
    ) -> Vec<Self> {
        todo!()
    }
}

struct PostRecord {
    id: i32,
    user_id: i32,
    content: String,
}

struct NewPostRecord<'a> {
    user_id: i32,
    content: &'a str,
}

impl<'a> NewPostRecord<'a> {
    pub fn new(user_id: i32, content: &str) -> Self {
        todo!()
    }

    pub async fn create(&self, conn: &mut Connection) -> QueryResult<PostRecord> {
        todo!()
    }
}
