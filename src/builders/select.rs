use crate::{*, utils::*};

use tygres_macros::builder;
builder! {
    struct SelectBuilder {
        source: F[Source],
        selection: S as selecting(Sel: Selection<F>),
        where_clause: W as filter(Cl: Clause<F>),
        order: O as ordering(Order: OrderByClause<F>),
        limit: L as limiting(Lim),
        offset: Of as offsetting(Off),
        suffix: Suf as *suffixing(Suff),
    }
}

pub trait Suffix {
    fn push_sql(&self, buf: &mut String);
}

impl Suffix for Unit {
    fn push_sql(&self, buf: &mut String) {}
}

impl Suffix for &'static str {
    fn push_sql(&self, buf: &mut String) {
        buf.push_str(self);
    }
}

pub trait Limiting {
    type Set;
    fn to_setter(self) -> Self::Set;

    fn push_limit(&self, buf: &mut String, idx: usize) -> usize;
}

impl Limiting for Unit {
    type Set = Unit;
    fn to_setter(self) -> Self::Set { self }

    fn push_limit(&self, _buf: &mut String, idx: usize) -> usize { idx }
}

impl Limiting for Wrap<usize> {
    type Set = Unit;
    fn to_setter(self) -> Self::Set { Unit }
    fn push_limit(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(&format!(" LIMIT {}", self.0));
        idx
    }
}

impl Limiting for Wrap<Holder> {
    type Set = Holder;
    fn to_setter(self) -> Self::Set { self.0 }
    fn push_limit(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(&format!(" LIMIT ${}", idx));
        idx + 1
    }
}


pub trait Offsetting {
    type Set;
    fn push_offset(&self, buf: &mut String, idx: usize) -> usize;
    fn to_setter(self) -> Self::Set;
}

impl Offsetting for Unit {
    type Set = Unit;
    fn push_offset(&self, _buf: &mut String, idx: usize) -> usize {
        idx
    }
    fn to_setter(self) -> Self::Set {
        Unit
    }
}

impl Offsetting for Wrap<usize> {
    type Set = Unit;
    fn push_offset(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(&format!(" OFFSET {}", self.0));
        idx
    }
    fn to_setter(self) -> Self::Set {
        Unit
    }

}

impl Offsetting for Wrap<Holder> {
    type Set = Self;
    fn push_offset(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(&format!(" OFFSET ${}", idx));
        idx + 1
    }
    fn to_setter(self) -> Self::Set {
        self
    }

}

impl<
    F: Source, S: Selection<F>, W: WhereClause<F>,
    O: OrderByClause<F>, L: Limiting, Of: Offsetting, Suf: Suffix,
> IntoSql for SelectBuilder<F, Wrap<S>, W, O, L, Of, Suf> {

    type Set = SqlInput<Unit, W::Set, L::Set, Of::Set>;
    type Get = Wrap<S>;

    fn push_sql(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("SELECT ");
        self.selection.0.push_selection(&self.source, buf);
        buf.push_str(" FROM ");
        self.source.push_source(buf);
        let idx = self.where_clause.push_where_clause(buf, idx);
        let idx = self.limit.push_limit(buf, idx);
        let idx = self.offset.push_offset(buf, idx);
        self.suffix.push_sql(buf);
        idx
    }

    fn into_types(self) -> (Self::Get, Self::Set) {
        (
            self.selection,
            SqlInput {
                values: Unit,
                where_clause: self.where_clause.into_types(),
                limit: self.limit.to_setter(),
                offset: self.offset.to_setter(),
            }
        )
    }

}

pub struct CursorQuery<S> {
    pub prepared: String,
    setter: S,
    idx: usize,
}

pub struct Fetcher<G> {
    pub name: String,
    getter: G,
}

impl<
    F: Source, S: Selection<F>, W: WhereClause<F>,
    O: OrderByClause<F>, L: Limiting, Of: Offsetting, Suf: Suffix,
> SelectBuilder<F, Wrap<S>, W, O, L, Of, Suf> {

    pub fn into_cursor(self, name: &str)
    -> (
        CursorQuery<SqlInput<Unit, W::Set, L::Set, Of::Set>>,
        Fetcher<Wrap<S>>,
    ) {

        let mut sql: String = String::with_capacity(0x1000);
        sql.push_str("DECLARE ");
        sql.push_str(name);
        sql.push_str(" CURSOR FOR ");
        let idx = self.push_sql(&mut sql, 1);
        let (getter, setter) = self.into_types();

        (
            CursorQuery{
                prepared: sql,
                setter, idx,
            },
            Fetcher {
                name: name.to_owned(),
                getter,
            },
        )

    }
}

impl<S> IntoSql for CursorQuery<S> {

    type Set = S;
    type Get = Unit;

    fn push_sql(&self, buf: &mut String, idx: usize) -> usize {
        if idx != 1 {
            panic!("cursors can not be sub-queries");
        }
        buf.push_str(&self.prepared);
        self.idx
    }

    fn into_types(self) -> (Self::Get, Self::Set) {
        (Unit, self.setter)
    }
}


pub struct Batch<'a, G> {
    pub name: &'a str,
    getter: &'a G,
    count: usize,
}

impl<G> Fetcher<G> {
    pub fn fetch(&self, count: usize) -> Batch<G> {
        Batch{
            name: self.name.as_ref(),
            getter: &self.getter,
            count,
        }
    }
}


impl<'a, G> IntoSql for Batch<'a, G> {
    type Set = Unit;
    type Get = &'a G;

    fn push_sql(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("FETCH ");
        buf.push_str(&self.count.to_string());
        buf.push_str(" FROM ");
        buf.push_str(self.name);
        idx
    }

    fn into_types(self) -> (Self::Get, Self::Set) {
        (self.getter, Unit)
    }
}
