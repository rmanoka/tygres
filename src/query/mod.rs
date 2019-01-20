use crate::{*, utils::*};
use postgres::types::{FromSql, ToSql};

pub struct SqlInput<V, W, L, O> {
    pub values: V,
    pub where_clause: W,
    pub limit: L,
    pub offset: O,
}

impl<
    'a,
    V: Takes<'a, Unit>,
    W: Takes<'a, Unit>,
    L: Takes<'a, Unit>,
    O: Takes<'a, Unit>,
> Takes<'a, Unit> for SqlInput<V, W, L, O> {
    fn push_values<'b:'a>(&'b self, _values: Unit, buf: &mut Vec<&'a ToSql>) {
        self.push_values(
            ValueBuilder::new(), buf
        );
    }
}

pub trait IntoSql: Sized {
    type Get;
    type Set;

    fn push_sql(&self, buf: &mut String, idx: usize) -> usize;
    fn into_types(self) -> (Self::Get, Self::Set);
}

pub trait Row {
    fn get<'b, T>(&'b self, idx: usize) -> T
    where T: FromSql<'b>;
}

impl Row for tokio_postgres::Row {
    fn get<'b, T>(&'b self, idx: usize) -> T
    where T: FromSql<'b> { self.get(idx) }
}

impl<'a> Row for postgres::rows::Row<'a> {
    fn get<'b, T>(&'b self, idx: usize) -> T
    where T: FromSql<'b> { self.get(idx) }
}

pub struct TypedRow<'a, Get, R: Row>(pub &'a Get, pub R);

impl<'a, Get, R: Row> TypedRow<'a, Get, R> {
    pub fn as_value<B>(&'a self) -> B
    where Wrap<B>: Makes<'a, Get> {
        (<Wrap<B> as Makes<'a, Get>>::get(&self.0, &self.1, 0).0).0
    }
}

pub mod synchronous;
pub mod asynchronous;
