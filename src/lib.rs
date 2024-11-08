#![allow(dead_code)]
#![allow(unused_variables)]

pub extern crate defile;
pub extern crate paste;

pub struct Related<T>(T);

#[macro_export]
macro_rules! zomg {
    // Main entrypoint.
    ($(#[$attr:meta])* $pub:vis struct $name:ident { $($fields:tt)* } ) => {
        $crate::internal_record!($(#[$attr])* $pub $name ($($fields)*));
        $crate::internal_model!($pub $name ($($fields)*));
        $crate::internal_impl!($name ($($fields)*));
    };
}

#[macro_export]
macro_rules! internal_record {
    // Done, generate struct.
    (@record () -> { $(#[$attr:meta])* $pub:vis $name:ident $(($field:ident : $type:ty))* }) => {
        $crate::paste::paste! {
            $(#[$attr])*
            $pub struct [<$name Record>] {
                $($field : $type),*
            }
        }

        $crate::internal_new_record!($pub $name ($($field : $type),*));
    };

    // Replace relation fields with foreign key.
    (@record ($field:ident : Related<$type:ty> $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        $crate::paste::paste! {
            $crate::internal_record!(@record ($($($rest)*)?) -> { $($output)* ([<$field _id>] : i32) });
        }
    };

    // Iterate over struct fields.
    (@record ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        $crate::internal_record!(@record ($($($rest)*)?) -> { $($output)* ($field : $type) });
    };

    // Entrypoint.
    ($(#[$attr:meta])* $pub:vis $name:ident ($($rest:tt)*)) => {
        $crate::internal_record!(@record ($($rest)*) -> { $(#[$attr])* $pub $name });
    };
}

#[macro_export]
macro_rules! internal_new_record {
    // Done, generate struct and generate new_record associated function for model.
    (@new_record () -> { $pub:vis $name:ident $(($field:ident : $type:ty))* }) => {
        $crate::paste::paste! {
            #[derive(Debug)]
            $pub struct [<New $name Record>]<'a> {
                $($field : $type,)*
            }
        }

        impl $name {
            $crate::paste::paste! {
                pub fn new_record<'a>($($field : $type),*) -> [<New $name Record>]<'a> {
                    [<New $name Record>] {
                        $($field,)*
                    }
                }
            }
        }
    };

    // @TODO handle other owned types.
    // Convert String fields to &'a str.
    (@new_record ($field:ident : String $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        $crate::defile::defile! {
            $crate::internal_new_record!(@@new_record ($($(@$rest)*)?) -> { $($output)* ($field : &'a str) });
        }
    };

    // Remove id field.
    (@new_record (id : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        $crate::defile::defile! {
            $crate::internal_new_record!(@@new_record ($($(@$rest)*)?) -> { $($output)* });
        }
    };

    // Iterate over struct fields.
    (@new_record ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        $crate::defile::defile! {
            $crate::internal_new_record!(@@new_record ($($(@$rest)*)?) -> { $($output)* ($field : $type) });
        }
    };

    // NewRecord entrypoint.
    ($pub:vis $name:ident ($($rest:tt)*)) => {
        $crate::internal_new_record!(@new_record ($($rest)*) -> { $pub $name });
    };
}

#[macro_export]
macro_rules! internal_model {
    // Done, generate struct.
    (@model () -> { $pub:vis $name:ident $(($field:ident : $type:ty))* }) => {
        $pub struct $name {
            $($field : $type),*
        }
    };

    // Strip out relation marker.
    (@model ($field:ident : Related<$type:ty> $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        $crate::internal_model!(@model ($($($rest)*)?) -> { $($output)* ($field : $type) });
    };

    // Iterate over struct fields.
    (@model ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* }) => {
        $crate::internal_model!(@model ($($($rest)*)?) -> { $($output)* ($field : $type) });
    };

    // Entrypoint.
    ($pub:vis $name:ident ($($rest:tt)*)) => {
        $crate::internal_model!(@model ($($rest)*) -> { $pub $name });
    };
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! internal_impl {
    // Done, generate model impl.
    (@impl () -> { $name:ident $(($field:ident : $type:ty))* } [ $(($key:ident ; $foreign_key:ident : $model:ty))* ]) => {
        impl $name {
            $crate::paste::paste! {
                pub async fn from_record(record: &[<$name Record>], conn: &mut Connection) -> QueryResult<Self> {
                    // @TODO can these queries be more efficient? Since we know all the relations
                    // ahead of time now, we may be able to just run one query.
                    $(
                        let $key: [<$model Record>] = crate::schema::[<$model:lower>]::table
                            .find(record.$foreign_key)
                            .first(conn)
                            .await?;
                        let $key = $model::from_record(&$key, conn).await?;
                    )*
                    Ok($name {
                        $($key,)*
                        $(
                            $field: record.$field.clone(),
                        )*
                    })
                }
            }
        }
    };

    // Put relation fields in a separate accumulator.
    (@impl ($field:ident : Related<$type:ty> $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($relations:tt)* ]) => {
        $crate::paste::paste! {
            $crate::internal_impl!(@impl ($($($rest)*)?) -> { $($output)* } [ $($relations)* ($field ; [<$field _id>] : $type) ]);
        }
    };

    // Iterate over struct fields.
    (@impl ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($relations:tt)* ]) => {
        $crate::internal_impl!(@impl ($($($rest)*)?) -> { $($output)* ($field : $type) } [ $($relations)* ]);
    };

    // Entrypoint.
    ($name:ident ($($rest:tt)*)) => {
        $crate::internal_impl!(@impl ($($rest)*) -> { $name } []);
    };
}
