use crate::{*, utils::*};
use postgres::types::ToSql;

pub trait ColumnsSetter<F: Source> {
    fn push_selection(&self, buf: &mut String) -> bool;
    fn push_values(&self, buf: &mut String, idx: usize) -> usize;
}

impl<
    F: Source,
    A: ColumnsSetter<F>,
    B: ColumnsSetter<F>
> ColumnsSetter<F> for Seq<A, B> {
    #[inline]
    fn push_selection(&self, buf: &mut String) -> bool {
        if self.0.push_selection(buf) {
            buf.push_str(", ");
            self.1.push_selection(buf)
        } else {
            self.1.push_selection(buf)
        }
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> usize {
        let idx2 = self.0.push_values(buf, idx);
        if (idx2 != idx) {
            buf.push_str(", ");
        }
        self.1.push_values(buf, idx2)
    }
}

pub trait Takes<'a, S> {
    fn push_values<'b>(&'a self, values: S, buf: &'b mut Vec<&'a ToSql>);
}

impl<'a, S, T: Takes<'a, S>> Takes<'a, Wrap<S>> for Wrap<T> {
    #[inline]
    fn push_values<'b>(&'a self, values: Wrap<S>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
    }
}

impl<'a, S, T: Takes<'a, S>> Takes<'a, S> for ColWrap<T> {
    #[inline]
    fn push_values<'b>(&'a self, values: S, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values, buf);
    }
}

impl<'a, A, B, S: Takes<'a, A>, T: Takes<'a, B>> Takes<'a, Seq<A, B>> for Seq<S, T> {
    #[inline]
    fn push_values<'b>(&'a self, values: Seq<A, B>, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}

impl<'a, A: 'a, S: Takes<'a, A>, I: IntoIterator<Item = A>> Takes<'a, I> for Reps<S> {
    #[inline]
    fn push_values<'b>(&'a self, iter: I, buf: &'b mut Vec<&'a ToSql>) {
        for value in iter {
            self.1.push_values(value, buf);
        }
    }
}

// use std::op[]
// impl<'a, A: 'a, S: Takes<'a, &'a A>> Takes<'a, &'a [A]> for Reps<S> {
//     #[inline]
//     fn push_values<'b>(&'a self, slice: &'a [A], buf: &'b mut Vec<&'a ToSql>) {
//         for value in slice {
//             self.1.push_values(value, buf);
//         }
//     }
// }



impl<'a> Takes<'a, Unit> for Unit {
    #[inline]
    fn push_values<'b>(&'a self, _values: Unit, _buf: &'b mut Vec<&'a ToSql>) {}
}

impl<'a, S: Takes<'a, Unit>> Takes<'a, Unit> for Wrap<S> {
    #[inline]
    fn push_values<'b>(&'a self, values: Unit, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(values, buf);
    }
}

impl<'a, S: Takes<'a, Unit>, T: Takes<'a, Unit>> Takes<'a, Unit> for Seq<S, T> {
    #[inline]
    fn push_values<'b>(&'a self, _values: Unit, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(Unit, buf);
        self.1.push_values(Unit, buf);
    }
}


impl<'a, S, A: 'a> Takes<'a, Unit> for WithValue<S, A>
where S: Takes<'a, &'a A> {
    #[inline]
    fn push_values<'b>(&'a self, _values: Unit, buf: &'b mut Vec<&'a ToSql>) {
        self.0.push_values(&self.1, buf);
    }
}

impl<
    F: Source, A,
    S: ColumnsSetter<F>,
> ColumnsSetter<F> for WithValue<S, A> {
    #[inline]
    fn push_selection(&self, buf: &mut String) -> bool {
        self.0.push_selection(buf)
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> usize {
        <S as ColumnsSetter<F>>::push_values(
            &self.0, buf, idx
        )
    }
}

pub struct OptionalSetter<S, A>(pub S, pub A);

impl<
    'a, F: Source, A: 'a,
    S: ColumnsSetter<F>,
> ColumnsSetter<F> for OptionalSetter<S, Option<A>> {
    #[inline]
    fn push_selection(&self, buf: &mut String) -> bool {
        match self.1 {
            Some(_) => self.0.push_selection(buf),
            None => false,
        }
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> usize {
        if let Some(_) = self.1 {
            <S as ColumnsSetter<F>>::push_values(
                &self.0, buf, idx
            )
        } else {
            idx
        }
    }
}

impl<
    'a, F: Source, A: 'a,
    S: ColumnsSetter<F>,
> ColumnsSetter<F> for OptionalSetter<S, &'a Option<A>> {
    #[inline]
    fn push_selection(&self, buf: &mut String) -> bool {
        match self.1 {
            Some(_) => self.0.push_selection(buf),
            None => false,
        }
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> usize {
        if let Some(_) = self.1 {
            <S as ColumnsSetter<F>>::push_values(
                &self.0, buf, idx
            )
        } else {
            idx
        }
    }
}

impl<'a, S, A: 'a> Takes<'a, Unit> for OptionalSetter<S, Option<A>>
where S: Takes<'a, &'a A> {
    #[inline]
    fn push_values<'b>(&'a self, _values: Unit, buf: &'b mut Vec<&'a ToSql>) {
        if let Some(ref val) = self.1 {
            self.0.push_values(val, buf);
        }
    }
}

impl<'a, S, A: 'a> Takes<'a, Unit> for OptionalSetter<S, &'a Option<A>>
where S: Takes<'a, &'a A> {
    #[inline]
    fn push_values<'b>(&'a self, _values: Unit, buf: &'b mut Vec<&'a ToSql>) {
        if let Some(ref val) = self.1 {
            self.0.push_values(val, buf);
        }
    }
}

impl<C> ColWrap<C> {
    #[inline]
    pub fn taking<'a, A: 'a>(self, assignment: A) -> WithValue<Self, A>
    where Self: Takes<'a, &'a A> {
        WithValue(self, assignment)
    }

    // #[inline]
    // pub fn if_some<'a, A: 'a>(self, assignment: Option<A>) -> OptionalSetter<Self, Option<A>>
    // where Self: Takes<'a, &'a A> {
    //     OptionalSetter(self, assignment)
    // }
    #[inline]
    pub fn if_some<A>(self, assignment: A) -> OptionalSetter<Self, A> {
        OptionalSetter(self, assignment)
    }
}


