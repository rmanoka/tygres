use crate::{*, utils::*};
use postgres::types::ToSql;

pub trait ColumnsSetter<F: Source> {
    fn push_selection(&self, buf: &mut String) -> bool;
    fn push_values(&self, buf: &mut String, idx: usize) -> (usize, bool);
}

impl<
    F: Source,
    A: ColumnsSetter<F>,
    B: ColumnsSetter<F>
> ColumnsSetter<F> for Seq<A, B> {
    #[inline]
    fn push_selection(&self, buf: &mut String) -> bool {
        let did = self.0.push_selection(buf);
        if did { buf.push_str(", "); }
        self.1.push_selection(buf)
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> (usize, bool) {
        let (idx, did) = self.0.push_values(buf, idx);
        if did { buf.push_str(", "); }
        self.1.push_values(buf, idx)
    }
}

pub trait Takes<'a, S> {
    fn push_values<'b:'a>(&'b self, values: S, buf: &mut Vec<&'a ToSql>);
}

impl<'a, S, T: Takes<'a, S>> Takes<'a, Wrap<S>> for Wrap<T> {
    #[inline]
    fn push_values<'b:'a>(&'b self, values: Wrap<S>, buf: &mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
    }
}

impl<'a, S, T: Takes<'a, S>> Takes<'a, S> for ColWrap<T> {
    #[inline]
    fn push_values<'b:'a>(&'b self, values: S, buf: &mut Vec<&'a ToSql>) {
        self.0.push_values(values, buf);
    }
}

impl<'a, A, B, S: Takes<'a, A>, T: Takes<'a, B>> Takes<'a, Seq<A, B>> for Seq<S, T> {
    #[inline]
    fn push_values<'b:'a>(&'b self, values: Seq<A, B>, buf: &mut Vec<&'a ToSql>) {
        self.0.push_values(values.0, buf);
        self.1.push_values(values.1, buf);
    }
}

impl<'a,
        A, B,
        S: Takes<'a, &'a A>,
        T: Takes<'a, &'a B>> Takes<'a, &'a Seq<A, B>> for Seq<S, T> {
    #[inline]
    fn push_values<'b:'a>(&'b self, values: &'a Seq<A, B>, buf: &mut Vec<&'a ToSql>) {
        self.0.push_values(&values.0, buf);
        self.1.push_values(&values.1, buf);
    }
}


impl<'a, A: 'a, S: Takes<'a, A>, I: IntoIterator<Item = A>> Takes<'a, I> for Reps<S> {
    #[inline]
    fn push_values<'b:'a>(&'b self, iter: I, buf: &mut Vec<&'a ToSql>) {
        for value in iter {
            self.1.push_values(value, buf);
        }
    }
}

impl<'a> Takes<'a, Unit> for Unit {
    #[inline]
    fn push_values<'b:'a>(&'b self, _values: Unit, _buf: &mut Vec<&'a ToSql>) {}
}

impl<'a, S: Takes<'a, Unit>> Takes<'a, Unit> for Wrap<S> {
    #[inline]
    fn push_values<'b:'a>(&'b self, values: Unit, buf: &mut Vec<&'a ToSql>) {
        self.0.push_values(values, buf);
    }
}

impl<'a, S: Takes<'a, Unit>, T: Takes<'a, Unit>> Takes<'a, Unit> for Seq<S, T> {
    #[inline]
    fn push_values<'b:'a>(&'b self, _values: Unit, buf: &mut Vec<&'a ToSql>) {
        self.0.push_values(Unit, buf);
        self.1.push_values(Unit, buf);
    }
}


impl<'a, S, A: 'a> Takes<'a, Unit> for WithValue<S, A>
where S: Takes<'a, &'a A> {
    #[inline]
    fn push_values<'b:'a>(&'b self, _values: Unit, buf: &mut Vec<&'a ToSql>) {
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
    fn push_values(&self, buf: &mut String, idx: usize) -> (usize, bool) {
        <S as ColumnsSetter<F>>::push_values(
            &self.0, buf, idx
        )
    }
}

pub struct OptValue<S, A>(pub S, pub A);

impl<
    'a, F: Source, A: 'a,
    S: ColumnsSetter<F>,
> ColumnsSetter<F> for OptValue<S, Option<A>> {
    #[inline]
    fn push_selection(&self, buf: &mut String) -> bool {
        match self.1 {
            Some(_) => self.0.push_selection(buf),
            None => false,
        }
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> (usize, bool) {
        if let Some(_) = self.1 {
            <S as ColumnsSetter<F>>::push_values(
                &self.0, buf, idx
            )
        } else {
            (idx, false)
        }
    }
}

impl<
    'a, 'c, F: Source, A,
    S: ColumnsSetter<F>,
> ColumnsSetter<F> for OptValue<S, &'c Option<A>> {
    #[inline]
    fn push_selection(&self, buf: &mut String) -> bool {
        match self.1 {
            Some(_) => self.0.push_selection(buf),
            None => false,
        }
    }

    #[inline]
    fn push_values(&self, buf: &mut String, idx: usize) -> (usize, bool) {
        if let Some(_) = self.1 {
            <S as ColumnsSetter<F>>::push_values(
                &self.0, buf, idx
            )
        } else {
            (idx, false)
        }
    }
}

impl<'a, S, A: 'a> Takes<'a, Unit> for OptValue<S, Option<A>>
where S: Takes<'a, &'a A> {
    #[inline]
    fn push_values<'b:'a>(&'b self, _values: Unit, buf: &mut Vec<&'a ToSql>) {
        if let Some(ref val) = self.1 {
            self.0.push_values(val, buf);
        }
    }
}

impl<'a, 'c, S, A> Takes<'a, Unit> for OptValue<S, &'c Option<A>>
where S: for<'b> Takes<'b, &'b A> {
    #[inline]
    fn push_values<'b:'a>(&'b self, _values: Unit, buf: &mut Vec<&'a ToSql>) {
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

    #[inline]
    pub fn if_some<'a, A: 'a>(self, assignment: Option<A>) -> OptValue<Self, Option<A>>
    where Self: Takes<'a, &'a A> {
        OptValue(self, assignment)
    }

    #[inline]
    pub fn if_some_ref<'a, A: 'a>(self, assignment: &'a Option<A>) -> OptValue<Self, &'a Option<A>>
    where Self: Takes<'a, &'a A> {
        OptValue(self, assignment)
    }
}

pub trait RefSetter<'a> {
    type Out: Takes<'a, Unit> + 'a;
    fn as_setter(&'a self) -> Self::Out;
}

pub trait ValSetter {
    type Out: for<'a> Takes<'a, Unit>;
    fn as_setter(self) -> Self::Out;
}

pub trait HasSetter<'a> {
    type Val;
    // type ValOwned;
    type Set: Takes<'a, Self::Val>;
    fn setter() -> Self::Set;
    fn as_value(&'a self) -> Self::Val;
    // fn into_value(self) -> Self::ValOwned;
}

pub trait HasOwnedSetter {
    type Val;
    type Set: for<'a> Takes<'a, &'a Self::Val>;
    fn setter() -> Self::Set;
    fn into_value(self) -> Self::Val;
}

