use crate::{*, utils::*};
pub use postgres::GenericConnection as Connection;
use postgres::{
    stmt::Statement as PgStatement,
    Error, rows::{Rows, Iter},
};

pub trait Synchronous: IntoSql + Sized {
    fn prepare(mut self, conn: &dyn Connection)
    -> Result<Statement<Self::Get, Self::Set>, Error> {

        let mut sql: String = String::with_capacity(0x1000);
        let idx = self.push_sql(&mut sql, 1);
        let (getter, setter) = self.into_types();

        Ok(Statement{
            statement: conn.prepare(&sql)?,
            getter, setter,
            setter_count: idx - 1,
        })

    }
}

impl<T: IntoSql + Sized> Synchronous for T {}

pub struct Statement<'a, Get, Set> {
    pub statement: PgStatement<'a>,
    pub getter: Get,
    pub setter: Set,
    setter_count: usize,
}

impl<'a, Get, Set> Statement<'a, Get, Set> {
    pub fn execute_with<'b, A>(&'b self, assignment: A) -> Result<u64, Error>
    where Set: Takes<'b, A> {
        let mut values = Vec::with_capacity(self.setter_count);
        self.setter.push_values(assignment, &mut values);
        self.statement.execute(&values[..])
    }

    pub fn query_with<'b, A>(&'b self, assignment: A)
    -> Result<QueryRows<'b, Get>, Error>
    where Set: Takes<'b, A> {
        let mut values = Vec::with_capacity(self.setter_count);
        self.setter.push_values(assignment, &mut values);

        Ok(QueryRows{
            rows: self.statement.query(&values[..])?,
            getter: &self.getter,
        })
    }
}

impl<'a, Get, Set> Statement<'a, Get, Set> {
    pub fn execute<'b>(&'b self) -> Result<u64, Error>
    where Set: Takes<'b, Unit> {
        self.execute_with(Unit)
    }

    pub fn query<'b>(&'b self) -> Result<QueryRows<'b, Get>, Error>
    where Set: Takes<'b, Unit> {
        self.query_with(Unit)
    }
}

pub struct QueryRows<'a, Get> {
    rows: Rows,
    getter: &'a Get,
}

impl<'a, Get> QueryRows<'a, Get> {
    pub fn iter<'b>(&'b self) -> QueryIter<'b, Get> {
        QueryIter {
            iter: self.rows.iter(),
            getter: &self.getter,
        }
    }
}

impl<'a, Get> IntoIterator for &'a QueryRows<'a, Get> {
    type Item = <QueryIter<'a, Get> as Iterator>::Item;
    type IntoIter = QueryIter<'a, Get>;

    fn into_iter(self) -> QueryIter<'a, Get> {
        self.iter()
    }
}


pub struct QueryIter<'a, Get> {
    getter: &'a Get,
    iter: Iter<'a>,
}

impl<'a, Get> Iterator for QueryIter<'a, Get> {
    type Item = TypedRow<'a, Get, postgres::rows::Row<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|row| TypedRow(self.getter, row))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

}