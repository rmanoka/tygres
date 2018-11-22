use crate::{*, utils::*};

use tygres_macros::builder;
builder! {
    struct InsertBuilder {
        source: F[Source],
        values: V as setting(Val: ColumnsSetter<F>),
        selection: S as returning(Sel: Selection<F>),
        reps: R as *repeating(Rep),
    }
}


pub trait InsValue<F: Source> {
    fn push_values(&self, buf: &mut String, reps: usize, idx: usize) -> usize;
}

impl<F: Source, S: ColumnsSetter<F>> InsValue<F> for Wrap<S> {
    fn push_values(&self, buf: &mut String, reps: usize, idx: usize) -> usize {
        if reps == 0 {
            panic!("reps must be a positive integer");
        }
        buf.push_str("(");
        if !self.0.push_selection(buf) {
            panic!("selection empty");
        }
        buf.push_str(") VALUES");
        let mut idx = idx;
        for i in 0..reps {
            if i != 0 {
                buf.push_str(", ");
            }
            buf.push_str(" (");
            idx = self.0.push_values(buf, idx);
            buf.push_str(")");
        }
        idx
    }
}

impl<F: Source> InsValue<F> for Unit {
    fn push_values(&self, buf: &mut String, reps: usize, idx: usize) -> usize {
        if reps != 1 {
            panic!("Only one row can be inserted with default values");
        }
        buf.push_str(" DEFAULT VALUES");
        idx
    }
}

impl<F: Source, V, S: ReturningClause<F>> IntoSql
for InsertBuilder<F, Wrap<V>, S, usize>
where Wrap<V>: InsValue<F> {

    type Set = SqlInput<Wrap<Reps<V>>, Unit, Unit, Unit>;
    type Get = S;

    fn push_sql(&mut self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("INSERT INTO ");
        self.source.push_source(buf);
        let idx = self.values.push_values(buf, self.reps, idx);
        self.selection.push_returning(&self.source, buf);
        idx
    }

    fn into_types(self) -> (S, Self::Set) {
        (
            self.selection,
            SqlInput {
                values: Wrap(Reps(self.reps, self.values.0)),
                where_clause: Unit,
                limit: Unit,
                offset: Unit,
            }
        )
    }
}

impl<F: Source, V: InsValue<F>, S: ReturningClause<F>> IntoSql
for InsertBuilder<F, V, S, Unit> {

    type Set = SqlInput<V, Unit, Unit, Unit>;
    type Get = S;

    fn push_sql(&mut self, buf: &mut String, idx: usize) -> usize {
        buf.push_str("INSERT INTO ");
        self.source.push_source(buf);
        let idx = self.values.push_values(buf, 1, idx);
        self.selection.push_returning(&self.source, buf);
        idx
    }

    fn into_types(self) -> (S, Self::Set) {
        (
            self.selection,
            SqlInput {
                values: self.values,
                where_clause: Unit,
                limit: Unit,
                offset: Unit,
            }
        )
    }
}
