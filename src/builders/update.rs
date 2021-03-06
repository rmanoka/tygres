use crate::{*, utils::*};

use tygres_macros::builder;
builder! {
    struct UpdateBuilder {
        source: F[Source],
        values: V as setting(Val: ColumnsSetter<F>),
        selection: S as returning(Sel: Selection<F>),
        where_clause: W as filter(Cl: Clause<F>),
    }
}

pub trait UpdValue<F: Source> {
    fn push_values(&self, buf: &mut String, idx: usize) -> usize;
}

impl<F: Source, S: ColumnsSetter<F>> UpdValue<F> for Wrap<S> {
    fn push_values(&self, buf: &mut String, idx: usize) -> usize {
        buf.push_str(" SET ");
        buf.push_str("(");
        if !self.0.push_selection(buf) {
            panic!("selection empty");
        }
        buf.push_str(") = ( ROW (");
        let (idx, _) = self.0.push_values(buf, idx);
        buf.push_str("))");
        idx
    }
}


impl<F: Source, V: UpdValue<F>, S: ReturningClause<F>, W: WhereClause<F>> IntoSql
for UpdateBuilder<F, V, S, W> {

    type Set = SqlInput<V, W::Set, Unit, Unit>;
    type Get = S;

    fn push_sql(&self, buf: &mut String, idx: usize) ->  usize {
        buf.push_str("UPDATE ");
        self.source.push_source(buf);
        let idx = self.values.push_values(buf, idx);
        let idx = self.where_clause.push_where_clause(buf, idx);
        self.selection.push_returning(&self.source, buf);
        idx
    }

    fn into_types(self) -> (Self::Get, Self::Set) {
        (
            self.selection,
            SqlInput {
                values: self.values,
                where_clause: self.where_clause.into_types(),
                limit: Unit,
                offset: Unit,
            }
        )
    }
}
