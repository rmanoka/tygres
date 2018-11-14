#[macro_export]
macro_rules! seq {
    ($first:expr, $($rest: expr),* $(,)*) => {
        $crate::utils::Seq($first, seq!($($rest),+))
    };
    ($one:expr) => {
        $one
    };
}

#[macro_export]
macro_rules! Seq {
    ($first:ty, $($rest: ty),* $(,)*) => {
        $crate::utils::Seq<$first, Seq!($($rest),+)>
    };
    ($one:ty) => {
        $one
    };
}

#[macro_export]
macro_rules! table {
    (*$name:ident, $table_name: expr) => {
        table!(*$name, $table_name);
        impl Sources for $name {}
    };
    ($name:ident, $table_name: expr) => {
        pub struct $name;
        impl $crate::Source for $name {
            fn push_source(&self, buf: &mut String) {
                buf.push_str($table_name);
            }
        }
    };
}

#[macro_export]
macro_rules! column {
    ($table:ident, *$name:ident, $column_name: expr) => {
        impl $crate::Column<$table> for $name {
            fn push_name(&self, buf: &mut String) {
                buf.push_str($column_name);
            }
        }
    };
    ($table:ident, $name:ident, $caps:ident, $column_name: expr) => {
        pub struct $name;
        pub const $caps: $crate::utils::ColWrap<$name> = $crate::utils::ColWrap($name);
        column!($table, *$name, $column_name);
    };
}

#[macro_export]
macro_rules! takes {
    ($name:ident, $ty:ty) => {
        impl<'a> $crate::Takes<'a, &'a $ty> for $name {
            #[inline]
            fn push_values<'b>(&'a self, values: &'a $ty, buf: &'b mut Vec<&'a postgres::types::ToSql>) {
                buf.push(values);
            }
        }
    };
}

#[macro_export]
macro_rules! makes {
    ($name:ident, $ty:ty) => {
        impl<'a> $crate::Makes<'a, $ty> for $name {
            fn get<R: $crate::Row>(&'a self, row: &'a R, idx: usize) -> ($ty, usize) {
                (row.get(idx), idx + 1)
            }
        }
    }
}

#[macro_export]
macro_rules! takes_json {
    ($name:ident, $ty:ty) => {

        impl<'a> postgres::types::ToSql for $ty {
            fn to_sql(&self, ty: &postgres::types::Type, out: &mut Vec<u8>) -> Result<postgres::types::IsNull, Box<std::error::Error + Sync + Send>> {
                postgres::types::Json(self).to_sql(ty, out)
            }

            fn accepts(ty: &postgres::types::Type) -> bool {
                <postgres::types::Json::<$ty> as postgres::types::ToSql>::accepts(ty)
            }

            to_sql_checked!();
        }
        takes!($name, $ty);
    }
}

#[macro_export]
macro_rules! makes_json {
    ($name:ident, $ty:ty) => {

        impl<'a> postgres::types::FromSql<'a> for $ty {
            fn from_sql(ty: &postgres::types::Type, raw: &[u8]) -> Result<$ty, Box<std::error::Error + Sync + Send>> {
                postgres::types::Json::<$ty>::from_sql(ty, raw)
                    .map(|j| j.0)
            }

            fn accepts(ty: &postgres::types::Type) -> bool {
                <postgres::types::Json::<$ty> as postgres::types::FromSql>::accepts(ty)
            }
        }
        makes!($name, $ty);
    }
}

#[macro_export]
macro_rules! With {
    ($val:ty, $col:ty) => {
        $crate::utils::WithValue<
            $crate::utils::ColWrap<$col>,
            $val>
    }
}

#[macro_export]
macro_rules! Opt {
    ($val:ty, $col:ty) => {
        $crate::OptionalSetter<
            $crate::utils::ColWrap<$col>,
            $val>
    }
}
