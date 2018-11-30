use crate::{*, utils::*};
use postgres::types::ToSql;

pub trait Clause<F: Source>: Sized  {
    type Set;
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize;
    fn into_types(self) -> Self::Set;

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
    where Self::Set: Takes<'a, &'a A> {
        WithValue(self, assignment)
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

    type Set = WithValue<S::Set, A>;
    fn into_types(self) -> Self::Set {
        WithValue(self.0.into_types(), self.1)
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
    type Set = Seq![L::Set, R::Set];
    fn into_types(self) -> Self::Set {
        seq![self.0.into_types(), self.1.into_types()]
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
    type Set = Seq![L::Set, R::Set];
    fn into_types(self) -> Self::Set {
        seq![self.0.into_types(), self.1.into_types()]
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
    type Set = C::Set;
    fn into_types(self) -> Self::Set {
        self.0.into_types()
    }
}

pub trait WhereClause<F: Source>  {
    #[inline]
    fn push_where_clause(&self, buf: &mut String, idx: usize) -> usize;
    type Set;
    fn into_types(self) -> Self::Set;
}

impl<F: Source, C: Clause<F>> WhereClause<F> for Wrap<C> {
    #[inline]
    fn push_where_clause(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(" WHERE ");
        self.0.push_clause(buf, idx)
    }
    type Set = C::Set;
    fn into_types(self) -> Self::Set { self.0.into_types() }
}

impl<F: Source> WhereClause<F> for Unit {
    #[inline]
    fn push_where_clause(&self, _buf: &mut String, idx: usize) -> usize {
        idx
    }
    type Set = Unit;
    fn into_types(self) -> Self::Set { Unit }
}

pub struct Equality<C>(C);
pub struct IsNull<C>(C);

impl<C> ColWrap<C> {
    pub fn equality<F: Source>(self) -> Equality<Self>
    where C: Column<F> {
        Equality(self)
    }

    pub fn is_null<F: Source>(self) -> IsNull<Self>
    where C: Column<F> {
        IsNull(self)
    }

    pub fn in_query<'a, Q: IntoSql<Get=Wrap<Self>>>(
            self, query: Q) -> InSubQuery<Self, Q> {

        InSubQuery(self, query)

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
    type Set = ColWrap<C>;
    fn into_types(self) -> Self::Set {
        self.0
    }
}

impl<F: Source, C: Column<F>> Clause<F> for IsNull<ColWrap<C>> {
    #[inline]
    fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
        (self.0).0.push_name(buf);
        buf.push_str(" IS NULL");
        idx
    }
    type Set = Unit;
    fn into_types(self) -> Self::Set {
        Unit
    }
}

pub struct InSubQuery<C, Q: IntoSql>(C, Q);

impl<F: Source, C: Column<F>,
        Q: IntoSql<Get = Wrap<ColWrap<C>>>>
            Clause<F> for InSubQuery<ColWrap<C>, Q> {

    fn push_clause(&self, buf: &mut String, idx: usize) -> usize {
        (self.0).0.push_name(buf);
        buf.push_str(" IN ( ");
        let idx = self.1.push_sql(buf, idx);
        buf.push_str(")");
        idx
    }

    type Set = Q::Set;
    fn into_types(self) -> Self::Set {
        self.1.into_types().1
    }
}

