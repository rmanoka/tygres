use crate::{*, utils::*};
use postgres::types::ToSql;

pub trait Clause<F: Source>: Sized  {
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize;

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
}

impl<
    F: Source, A,
    S: Clause<F>,
> Clause<F> for WithValue<S, A> {
    #[inline]
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
        self.0.push_clause(buf, idx)
    }
}


pub struct And<L, R>(L, R);

impl<F: Source, L: Clause<F>, R: Clause<F>> Clause<F> for And<L, R> {
    #[inline]
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
    #[inline]
    fn push_values<'b>(&'a self, values: Seq<L, R>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}


pub struct Or<L, R>(L, R);

impl<F: Source, L: Clause<F>, R: Clause<F>> Clause<F> for Or<L, R> {
    #[inline]
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
    #[inline]
    fn push_values<'b>(&'a self, values: Seq<L, R>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}


pub struct Not<C>(C);

impl<F: Source, C: Clause<F>> Clause<F> for Not<C> {
    #[inline]
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
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
    fn push_where_clause(&self, buf: &mut String, idx: usize) -> usize;
}

impl<F: Source, C: Clause<F>> WhereClause<F> for Wrap<C> {
    #[inline]
    fn push_where_clause(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(" WHERE ");
        self.0.push_clause(buf, idx)
    }
}

impl<F: Source> WhereClause<F> for Unit {
    #[inline]
    fn push_where_clause(&self, _buf: &mut String, idx: usize) -> usize {
        idx
    }
}

pub struct Equality<C>(C);

impl<C> ColWrap<C> {
    pub fn equality<F: Source>(self) -> Equality<Self>
    where C: Column<F> {
        Equality(self)
    }
}

impl<F: Source, C: Column<F>> Clause<F> for Equality<ColWrap<C>> {
    #[inline]
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
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

// impl<C, A: Assignment<Wrap<C>>> Assignment<Equality<Wrap<C>>> for A {
//     fn push_values<'b>(&'a self, setter: &'a Equality<Wrap<C>>, buf: &'b mut Vec<&'a (ToSql)>) {
//         <Self as Assignment<Wrap<C>>>::push_values(
//             self, &setter.0, buf
//         )
//     }
// }

