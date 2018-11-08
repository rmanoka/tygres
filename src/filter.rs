use crate::{*, utils::*};
use postgres::types::ToSql;

pub trait Clause<F: Source>: Send + Sized  {
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize;
}

// impl<F: Source, C: Clause<F>, A: Assignment<C>> Clause<F> for WithValue<C, A> {
//     fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
//         self.0.push_clause(buf, idx)
//     }
// }

pub struct And<L, R>(L, R);

impl<F: Source, L: Clause<F>, R: Clause<F>> Clause<F> for And<L, R> {
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("(");
        let idx = self.0.push_clause(buf, idx);
        buf.push_str(") AND (");
        let idx = self.1.push_clause(buf, idx);
        buf.push_str(")");
        idx
    }
}

impl<'a, L, R, A: Takes<'a, L>, B: Takes<'a, R>> Takes<'a, Seq<L, R>> for And<A, B> {
    fn push_values<'b>(&'a self, values: Seq<L, R>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}

// impl<L, R, A: Assignment<L>, B: Assignment<R>> Assignment<And<L, R>> for Seq<A, B> {
//     fn push_values<'b>(&'a self, setter: &'a And<L, R>, buf: &'b mut Vec<&'a (ToSql)>) {
//         self.0.push_values(&setter.0, buf);
//         self.1.push_values(&setter.1, buf);
//     }
// }

// impl<L, R> Assignment<And<L, R>> for Unit
// where Unit: Assignment<L>, Unit: Assignment<R> {
//     fn push_values<'b>(&'a self, setter: &'a And<L, R>, buf: &'b mut Vec<&'a (ToSql)>) {
//         Unit.push_values(&setter.0, buf);
//         Unit.push_values(&setter.1, buf);
//     }
// }


pub struct Or<L, R>(L, R);

impl<F: Source, L: Clause<F>, R: Clause<F>> Clause<F> for Or<L, R> {
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("( ");
        let idx = self.0.push_clause(buf, idx);
        buf.push_str(" ) OR ( ");
        let idx = self.1.push_clause(buf, idx);
        buf.push_str(" )");
        idx
    }
}

impl<'a, L, R, A: Takes<'a, L>, B: Takes<'a, R>> Takes<'a, Seq<L, R>> for Or<A, B> {
    fn push_values<'b>(&'a self, values: Seq<L, R>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}

pub struct Not<C>(C);

impl<F: Source, C: Clause<F>> Clause<F> for Not<C> {
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("NOT ( ");
        let idx = self.0.push_clause(buf, idx);
        buf.push_str(" )");
        idx
    }
}

impl<'a, A, C: Takes<'a, A>> Takes<'a, A> for Not<C> {
    fn push_values<'b>(&'a self, values: A, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values, buf);
    }
}


pub trait WhereClause<F: Source>: Send  {
    fn push_where_clause(&self, buf: &mut String, idx: usize) -> usize;
}

impl<F: Source, C: Clause<F>> WhereClause<F> for Wrap<C> {
    fn push_where_clause(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(" WHERE ");
        self.0.push_clause(buf, idx)
    }
}

impl<F: Source> WhereClause<F> for Unit {
    fn push_where_clause(&self, _buf: &mut String, idx: usize) -> usize {
        idx
    }
}
