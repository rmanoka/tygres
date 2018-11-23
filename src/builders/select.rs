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
    O: OrderByClause<F>, L: Limiting, Of: Offsetting
> IntoSql for SelectBuilder<F, Wrap<S>, W, O, L, Of> {

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

