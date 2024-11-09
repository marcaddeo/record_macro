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
#[doc(hidden)]
macro_rules! internal_record {
    // Done, generate struct.
    (@record () -> { $(#[$attr:meta])* $pub:vis $name:ident $(($field:ident : $type:ty))* } [$(($from:ident : $from_type:ty))*] [$(($from_related: ident : $from_related_model:ty))*]) => {
        $crate::paste::paste! {
            $(#[$attr])*
            $pub struct [<$name Record>] {
                $($field : $type),*
            }

            impl From<$name> for [<$name Record>] {
                fn from(value: $name) -> Self {
                $(
                    let $from_related = value.[<$from_related_model:lower>].id;
                )*

                    Self {
                        $($from : value.$from,)*
                        $($from_related,)*
                    }
                }
            }
        }

        $crate::internal_new_record!($pub $name ($($field : $type),*));
    };

    // Strip out vec relation fields. These fields are "virtual" and used for one-to-many relations.
    (@record ($field:ident : Related<Vec<$type:ty>> $(, $($rest:tt)*)?) -> { $($output:tt)* } [$($from:tt)*] [$($from_related:tt)*]) => {
        $crate::paste::paste! {
            $crate::internal_record!(@record ($($($rest)*)?) -> { $($output)* } [$($from)*] [$($from_related)*]);
        }
    };

    // Replace relation fields with foreign key.
    (@record ($field:ident : Related<$type:ty> $(, $($rest:tt)*)?) -> { $($output:tt)* } [$($from:tt)*] [$($from_related:tt)*]) => {
        $crate::paste::paste! {
            $crate::internal_record!(@record ($($($rest)*)?) -> { $($output)* ([<$field _id>] : i32) } [$($from)*] [$($from_related)* ([<$field _id>] : $type)]);
        }
    };

    // Iterate over struct fields.
    (@record ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* } [$($from:tt)*] [$($from_related:tt)*]) => {
        $crate::internal_record!(@record ($($($rest)*)?) -> { $($output)* ($field : $type) } [$($from)* ($field : $type)] [$($from_related)*]);
    };

    // Entrypoint.
    ($(#[$attr:meta])* $pub:vis $name:ident ($($rest:tt)*)) => {
        $crate::internal_record!(@record ($($rest)*) -> { $(#[$attr])* $pub $name } [] []);
    };
}

#[macro_export]
#[doc(hidden)]
#[allow(clippy::crate_in_macro_def)]
macro_rules! internal_new_record {
    // Done, generate struct and generate new_record associated function for model.
    (@new_record () -> { $pub:vis $name:ident $(($field:ident : $type:ty))* } [ $(($option:ident : $option_type:ty))* ]) => {
        $crate::paste::paste! {
            // NewModelRecord
            #[derive(Clone, Debug, Default, Insertable)]
            #[diesel(table_name = crate::schema::[<$name:lower>])]
            #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
            $pub struct [<New $name Record>]<'a> {
                $($field : $type ,)*
                $($option : $option_type ,)*
            }

            impl<'a> [<New $name Record>]<'a> {
                // NewModelRecord::new
                #[doc = "Create a new `" [<New $name Record>] "` object."]
                pub fn new($($field : $type ,)*) -> [<New $name Record>]<'a> {
                    Self {
                        $($field ,)*
                        $($option : None ,)*
                    }
                }

            $(
                // NewRecord::with_$option
                #[doc = "Add the optional `" $option "` field to the `" [<New $name Record>] "` object."]
                 pub fn [<with_ $option>](self, $option : $option_type) -> Self {
                    Self {
                        $option,
                        ..self
                    }
                 }
            )*

                // NewModelRecord::create
                #[doc = "Create a new `" [<$name:lower>] "` in the database."]
                pub async fn create(&self, conn: &mut Connection) -> QueryResult<[<$name Record>]> {
                    diesel::insert_into(crate::schema::[<$name:lower>]::table)
                        .values(self)
                        .returning(crate::schema::[<$name:lower>]::table::all_columns())
                        .get_result(conn)
                        .await
                }
            }

            impl $name {
                // Model::new_record
                #[doc = "Create a new `" [<New $name Record>] "` object."]
                pub fn new_record<'a>($($field : $type ,)*) -> [<New $name Record>]<'a> {
                    [<New $name Record>]::new($($field ,)*)
                }
            }
        }
    };

    // @TODO handle other owned types.
    // @TODO seems like the Option<OwnedType> is going to be repetitive, maybe see if twe can get
    // around that?

    // Convert Option<String> fields to Option<&'a str>, and put them in the optionial accumulator.
    (@new_record ($field:ident : Option<String> $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($optional:tt)* ]) => {
        $crate::defile::defile! {
            $crate::internal_new_record!(@@new_record ($($(@$rest)*)?) -> { $($output)* } [ $($optional)* ($field : Option<&'a str>) ]);
        }
    };

    // Put optional fields in a separate optional accumulator.
    (@new_record ($field:ident : Option<$type:ty> $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($optional:tt)* ]) => {
        $crate::defile::defile! {
            $crate::internal_new_record!(@@new_record ($($(@$rest)*)?) -> { $($output)* } [ $($optional)* ($field : Option<$type>) ]);
        }
    };

    // Convert String fields to &'a str.
    (@new_record ($field:ident : String $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($optional:tt)* ]) => {
        $crate::defile::defile! {
            $crate::internal_new_record!(@@new_record ($($(@$rest)*)?) -> { $($output)* ($field : &'a str) } [ $($optional)* ]);
        }
    };

    // Remove id field.
    (@new_record (id : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($optional:tt)* ]) => {
        $crate::defile::defile! {
            $crate::internal_new_record!(@@new_record ($($(@$rest)*)?) -> { $($output)* } [ $($optional)* ]);
        }
    };

    // Iterate over struct fields.
    (@new_record ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($optional:tt)* ]) => {
        $crate::defile::defile! {
            $crate::internal_new_record!(@@new_record ($($(@$rest)*)?) -> { $($output)* ($field : $type) } [ $($optional)* ]);
        }
    };

    // NewRecord entrypoint.
    ($pub:vis $name:ident ($($rest:tt)*)) => {
        $crate::internal_new_record!(@new_record ($($rest)*) -> { $pub $name } []);
    };
}

#[macro_export]
#[doc(hidden)]
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
#[doc(hidden)]
#[allow(clippy::crate_in_macro_def)]
macro_rules! internal_impl {
    // Done, generate model impl.
    // @TODO i can probably change $name -> $model everywhere to make this a bit clearer. And get
    // rid of the $model:ty here since i can just use the $model (name).
    (@impl () -> { $name:ident $(($field:ident : $type:ty))* } [ $(($key:ident ; $foreign_key:ident : $model:ty))* ] [ $(($many:ident : $many_model:ty))* ]) => {
        impl $name {
            $crate::paste::paste! {
                pub async fn from_record(record: &[<$name Record>], conn: &mut Connection) -> QueryResult<Self> {
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
                            $field : record.$field.clone(),
                        )*
                        $($many : vec![],)*
                    })
                }

                // pub async fn from_records<'a>(
                //     records: impl IntoIterator<Item = &'a PostRecord>,
                //     conn: &'a mut Connection,
                // ) -> Vec<Self> {
                //     todo!()
                // }

            $(
                pub async fn [<with_ $many>](self, conn: &mut Connection) -> QueryResult<Self> {
                    let record: [<$name Record>] = self.into();
                    let $many: Vec<[<$many_model Record>]> = [<$many_model Record>]::belonging_to(&record)
                        .select(crate::schema::[<$many_model:lower>]::table::all_columns())
                        .load(conn)
                        .await?;

                    todo!()
                }
            )*

            }
        }
    };

    // Put vec relation fields in a separate one-to-many accumulator.
    (@impl ($field:ident : Related<Vec<$type:ty>> $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($relations:tt)* ] [ $($many:tt)* ]) => {
        $crate::paste::paste! {
            $crate::internal_impl!(@impl ($($($rest)*)?) -> { $($output)* } [ $($relations)* ] [ $($many)* ($field : $type) ]);
        }
    };

    // Put relation fields in a separate accumulator.
    (@impl ($field:ident : Related<$type:ty> $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($relations:tt)* ] [ $($many:tt)* ]) => {
        $crate::paste::paste! {
            $crate::internal_impl!(@impl ($($($rest)*)?) -> { $($output)* } [ $($relations)* ($field ; [<$field _id>] : $type) ] [ $($many)* ]);
        }
    };

    // Iterate over struct fields.
    (@impl ($field:ident : $type:ty $(, $($rest:tt)*)?) -> { $($output:tt)* } [ $($relations:tt)* ] [ $($many:tt)* ]) => {
        $crate::internal_impl!(@impl ($($($rest)*)?) -> { $($output)* ($field : $type) } [ $($relations)* ] [ $($many)* ]);
    };

    // Entrypoint.
    ($name:ident ($($rest:tt)*)) => {
        $crate::internal_impl!(@impl ($($rest)*) -> { $name } [] []);
    };
}
