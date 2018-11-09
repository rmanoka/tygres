use crate::{*, utils::*};
use postgres::{
    stmt::Statement as PgStatement,
    Connection, Error,
    rows::{Rows, Iter},
};

pub trait Synchronous: IntoSql + Sized {
    fn prepare(self, conn: &Connection)
    -> Result<Statement<Self::Get, Self::Set>, Error> {

        let mut sql: String = String::with_capacity(0x1000);
        let count = self.push_sql(&mut sql);
        let types = self.into_types();
        Ok(Statement{
            statement: conn.prepare(&sql)?,
            getter: types.0,
            setter: types.1,
            setter_count: count,
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
    pub fn execute_with<A>(&'a self, assignment: A) -> Result<u64, Error>
    where Set: Takes<'a, A> {
        let mut values = Vec::with_capacity(self.setter_count);
        self.setter.push_values(assignment, &mut values);
        self.statement.execute(&values[..])
    }

    pub fn query_with<A>(&'a self, assignment: A)
    -> Result<QueryRows<'a, Get>, Error>
    where Set: Takes<'a, A> {
        let mut values = Vec::with_capacity(self.setter_count);
        self.setter.push_values(assignment, &mut values);

        Ok(QueryRows{
            rows: self.statement.query(&values[..])?,
            getter: &self.getter,
        })
    }
}

impl<'a, Get, Set> Statement<'a, Get, Set>
where Set: Takes<'a, Unit> {
    pub fn execute(&'a self) -> Result<u64, Error> {
        self.execute_with(Unit)
    }

    pub fn query(&'a self) -> Result<QueryRows<'a, Get>, Error> {
        self.query_with(Unit)
    }
}

pub struct QueryRows<'a, Get> {
    rows: Rows,
    getter: &'a Get,
}

impl<'a, Get> QueryRows<'a, Get> {
    pub fn iter(&'a self) -> QueryIter<'a, Get> {
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