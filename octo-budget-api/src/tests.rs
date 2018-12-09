mod db;
pub mod redis;

pub use self::db::DbSession;

#[macro_export]
macro_rules! tags_vec {
    ( $( $x:expr ),* ) => {
        {
            #[allow(unused_mut)]
            let mut temp_vec: Vec<String> = Vec::new();
            $(
                temp_vec.push($x.to_string());
            )*
            temp_vec
        }
    };
}
