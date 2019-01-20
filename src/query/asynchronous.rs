use crate::{*, utils::*};
use futures::{Future, Poll, Async, Stream};
use tokio_postgres::{
    Client, Prepare, Statement, Query, Execute,
    error::Error,
};

pub trait Asynchronous: IntoSql + Sized {

    fn prepare(self, cl: &mut Client)
    -> PrepareFuture<Self::Get, Self::Set> {

        let mut sql: String = String::with_capacity(0x1000);
        let idx = self.push_sql(&mut sql, 1);
        let (get, set) = self.into_types();

        PrepareFuture {
            prepare: cl.prepare(&sql),
            types: Some((get, set)),
            setter_count: idx - 1,
        }
    }
}

impl<T: IntoSql + Sized> Asynchronous for T {}

pub struct PrepareFuture<Get, Set> {
    pub types: Option<(Get, Set)>,
    pub prepare: Prepare,
    pub setter_count: usize,
}

impl<Get, Set> Future for PrepareFuture<Get, Set> {
    type Item = Prepared<Get, Set>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.prepare.poll() {
            Ok(Async::Ready(statement)) => {
                let types = std::mem::replace(&mut self.types, None);
                let (getter, setter) = match types {
                    Some(t) => t,
                    _ => panic!("PrepareFuture polled without types."),
                };
                Ok(Async::Ready(Prepared {
                    statement, getter, setter,
                    setter_count: self.setter_count,
                }))
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}

pub struct Prepared<Get, Set> {
    pub statement: Statement,
    pub getter: Get,
    pub setter: Set,
    pub setter_count: usize,
}

impl<Get, Set> Prepared<Get, Set> {
    pub fn execute_with<'a, A>(&'a self, cl: &mut Client, assignment: A) -> Execute
    where Set: Takes<'a, A> {

        let mut values = Vec::with_capacity(self.setter_count);
        self.setter.push_values(assignment, &mut values);
        cl.execute(&self.statement, &values[..])
    }

    pub fn query_with<'a, A>(&'a self, cl: &mut Client, assignment: A) -> QueryStream<'a, Get>
    where Set: Takes<'a, A> {
        let mut values = Vec::with_capacity(self.setter_count);
        self.setter.push_values(assignment, &mut values);

        QueryStream {
            query: cl.query(&self.statement, &values[..]),
            getter: &self.getter,
        }

    }
}

impl<Get, Set> Prepared<Get, Set>
where Set: for<'a> Takes<'a, Unit> {
    pub fn execute(&self, cl: &mut Client) -> Execute {
        self.execute_with(cl, Unit)
    }

    pub fn query<'a>(&'a self, cl: &mut Client) -> QueryStream<'a, Get> {
        self.query_with(cl, Unit)
    }
}


pub struct QueryStream<'a, Get> {
    getter: &'a Get,
    query: Query,
}

impl<'a, Get> Stream for QueryStream<'a, Get> {
    type Item = TypedRow<'a, Get, tokio_postgres::Row>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Error> {
        match self.query.poll() {
            Ok(Async::Ready(Some(row)))
                => Ok(Async::Ready(Some(
                    TypedRow(&self.getter, row)
                ))),
            Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}
