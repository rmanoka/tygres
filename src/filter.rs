use crate::{*, utils::*};
use postgres::types::ToSql;

pub trait Clause<F: Source>: Sized  {
    fn push_clause(&mut self, buf: &mut String, idx: usize) -> usize;

    #[inline]
    fn and<C: Clause<F>>(self, other: C) -> And<Self, C> {
        And(self, other)
    }

    #[inline]
    fn or<C: Clause<F>>(self, other: C) -> Or<Self, C> {
        Or(self, other)
    }

    #[inline]
    fn not(self) -> Not<Self> {
        Not(self)
    }

    #[inline]
    fn taking<'a, A: 'a>(self, assignment: A) -> WithValue<Self, A>
    where Self: Takes<'a, &'a A> {
        WithValue(self, assignment)
    }
}

impl<
    F: Source, A,
    S: Clause<F>,
> Clause<F> for WithValue<S, A> {
    #[inline]
    fn push_clause(&mut self, buf: &mut String, idx: usize) -> usize {
        self.0.push_clause(buf, idx)
    }
}


pub struct And<L, R>(L, R);

impl<F: Source, L: Clause<F>, R: Clause<F>> Clause<F> for And<L, R> {
    #[inline]
    fn push_clause(&mut self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("(");
        let idx = self.0.push_clause(buf, idx);
        buf.push_str(") AND (");
        let idx = self.1.push_clause(buf, idx);
        buf.push_str(")");
        idx
    }
}

impl<'a, L, R, A: Takes<'a, L>, B: Takes<'a, R>> Takes<'a, Seq<L, R>> for And<A, B> {
    #[inline]
    fn push_values<'b>(&'a self, values: Seq<L, R>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}


pub struct Or<L, R>(L, R);

impl<F: Source, L: Clause<F>, R: Clause<F>> Clause<F> for Or<L, R> {
    #[inline]
    fn push_clause(&mut self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("( ");
        let idx = self.0.push_clause(buf, idx);
        buf.push_str(" ) OR ( ");
        let idx = self.1.push_clause(buf, idx);
        buf.push_str(" )");
        idx
    }
}

impl<'a, L, R, A: Takes<'a, L>, B: Takes<'a, R>> Takes<'a, Seq<L, R>> for Or<A, B> {
    #[inline]
    fn push_values<'b>(&'a self, values: Seq<L, R>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}


pub struct Not<C>(C);

impl<F: Source, C: Clause<F>> Clause<F> for Not<C> {
    #[inline]
    fn push_clause(&mut self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("NOT ( ");
        let idx = self.0.push_clause(buf, idx);
        buf.push_str(" )");
        idx
    }
}

impl<'a, A, C: Takes<'a, A>> Takes<'a, A> for Not<C> {
    #[inline]
    fn push_values<'b>(&'a self, values: A, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values, buf);
    }
}


pub trait WhereClause<F: Source>  {
    #[inline]
    fn push_where_clause(&mut self, buf: &mut String, idx: usize) -> usize;
}

impl<F: Source, C: Clause<F>> WhereClause<F> for Wrap<C> {
    #[inline]
    fn push_where_clause(&mut self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(" WHERE ");
        self.0.push_clause(buf, idx)
    }
}

impl<F: Source> WhereClause<F> for Unit {
    #[inline]
    fn push_where_clause(&mut self, _buf: &mut String, idx: usize) -> usize {
        idx
    }
}

pub struct Equality<C>(C);
pub struct IsNull<C>(C);

pub struct InSubQuery<C, Q: IntoSql>(C, StoredQuery<Q>);
pub enum StoredQuery<Q: IntoSql> {
    Query(Q),
    Setter(Q::Set),
}

impl<C> ColWrap<C> {
    pub fn equality<F: Source>(self) -> Equality<Self>
    where C: Column<F> {
        Equality(self)
    }

    pub fn is_null<F: Source>(self) -> IsNull<Self>
    where C: Column<F> {
        IsNull(self)
    }

    pub fn in_query<Q: IntoSql<Get=Wrap<Self>>>(self, query: Q) -> InSubQuery<Self, Q> {
        InSubQuery(self, StoredQuery::Query(query))
    }
}

impl<F: Source, C: Column<F>> Clause<F> for Equality<ColWrap<C>> {
    #[inline]
    fn push_clause(&mut self, buf: &mut String, idx: usize) -> usize {
        (self.0).0.push_name(buf);
        buf.push_str(" = ");
        buf.push_str(&format!("${}", idx));
        idx + 1
    }
}

impl<'a, S, T: Takes<'a, S>> Takes<'a, S> for Equality<T> {
    #[inline]
    fn push_values<'b>(&'a self, values: S, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values, buf);
    }
}

impl<F: Source, C: Column<F>> Clause<F> for IsNull<ColWrap<C>> {
    #[inline]
    fn push_clause(&mut self, buf: &mut String, idx: usize) -> usize {
        (self.0).0.push_name(buf);
        buf.push_str(" IS NULL");
        idx
    }
}

impl<'a, T> Takes<'a, Unit> for IsNull<T> {
    #[inline]
    fn push_values<'b>(&'a self, _values: Unit, _buf: &'b mut Vec<&'a ToSql>) {
    }
}

impl<'a, F: Source, C: Column<F>,
        Q: IntoSql<Get = Wrap<ColWrap<C>>>>
            Clause<F> for InSubQuery<ColWrap<C>, Q> {

    fn push_clause(&mut self, buf: &mut String, idx: usize) -> usize {
        use self::StoredQuery::*;
        use std::{mem::replace, ptr};
        match self.1 {
            Query(ref mut query) => {
                (self.0).0.push_name(buf);
                buf.push_str(" IN ( ");
                let idx = query.push_sql(buf, idx);
                buf.push_str(" FOR UPDATE )");

                // Replacing Stored object in-place
                unsafe {
                    let query = ptr::read(query);
                    let types = query.into_types();
                    ptr::write(&mut self.1, Setter(types.1));
                }

                idx
            },
            _ => { panic!("push_clause may not be called more than once on sub-query clauses!"); }
        }
    }

}

impl<'a, A, C, S: Takes<'a, A> + 'a,
        Q: IntoSql<Get = Wrap<ColWrap<C>>, Set = S>>
        Takes<'a, A> for InSubQuery<ColWrap<C>, Q> {

    #[inline]
    fn push_values<'b>(&'a self, values: A, buf: &'b mut Vec<&'a ToSql>) {
        use self::StoredQuery::*;
        match self.1 {
            Setter(ref setter) => setter.push_values(values, buf),
            _ => panic!("push_values called before push_clause"),
        }
    }

}

