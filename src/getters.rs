use crate::{*, utils::*};
use postgres::types::{FromSql};

pub trait Selection<F: Source> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) -> bool;
}

impl<'a, F: Source, S: Selection<F>> Selection<F> for &'a S {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) -> bool {
        <S as Selection<F>>::push_selection(self, src, buf)
    }
}

impl<F: Source, A: Selection<F>, B: Selection<F>> Selection<F> for Seq<A, B> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) -> bool {
        if self.0.push_selection(src, buf) {
            buf.push_str(", ");
        }
        self.1.push_selection(src, buf)
    }
}

pub trait Makes<'a, S>: Sized {
    fn get<R: Row>(s: &'a S, row: &'a R, idx: usize) -> (Self, usize);
}

impl<'a, S, T: Makes<'a, S>> Makes<'a, &'a S> for T {
    fn get<R: Row>(s: &'a &'a S, row: &'a R, idx: usize) -> (Self, usize) {
        <T as Makes<'a, S>>::get(*s, row, idx)
    }
}

impl<
    'a, S, T,
    A: Makes<'a, S>,
    B: Makes<'a, T>
> Makes<'a, Seq<S, T>> for Seq<A, B> {
    fn get<R: Row>(s: &'a Seq<S, T>, row: &'a R, idx: usize) -> (Self, usize) {
        let (a, idx) = Makes::get(&s.0, row, idx);
        let (b, idx) = Makes::get(&s.1, row, idx);
        (Seq(a, b), idx)
    }
}

impl<'a, S, A: Makes<'a, S>> Makes<'a, ColWrap<S>> for A {
    fn get<R: Row>(s: &'a ColWrap<S>, row: &'a R, idx: usize) -> (Self, usize) {
        Makes::get(&s.0, row, idx)
    }
}

impl<'a, S, A: Makes<'a, S>> Makes<'a, Wrap<S>> for Wrap<A> {
    fn get<R: Row>(s: &'a Wrap<S>, row: &'a R, idx: usize) -> (Self, usize) {
        let (el, idx) = Makes::get(&s.0, row, idx);
        (Wrap(el), idx)
    }
}

pub trait Value: Sized {
    type Get;
    fn get<'a, R: Row>(g: &'a Self::Get, row: &'a R, idx: usize) -> (Self, usize);
}

pub trait ReturningClause<F: Source> {
    #[inline]
    fn push_returning(&self, src: &F, buf: &mut String);
}

impl<F: Source, S: Selection<F>> ReturningClause<F> for Wrap<S> {
    #[inline]
    fn push_returning(&self, src: &F, buf: &mut String) {
        buf.push_str(" RETURNING ");
        self.0.push_selection(src, buf);
    }
}

impl<F: Source> ReturningClause<F> for Unit {
    #[inline]
    fn push_returning(&self, _src: &F, _buf: &mut String) {}
}

pub struct OptionalSelection<S>(pub S, bool);

impl<C> ColWrap<C> {
    #[inline]
    pub fn select_if<S: Source>(self, assignment: bool) -> OptionalSelection<Self>
    where Self: Selection<S> {
        OptionalSelection(self, assignment)
    }
}

impl<F: Source, S: Selection<F>> Selection<F> for OptionalSelection<S> {
    #[inline]
    fn push_selection(&self, src: &F, buf: &mut String) -> bool {
        self.1 &&
            self.0.push_selection(src, buf)
    }
}

impl<'a, S, A: Makes<'a, S>> Makes<'a, OptionalSelection<S>> for Option<A> {
    fn get<R: Row>(s: &'a OptionalSelection<S>, row: &'a R, idx: usize) -> (Self, usize) {
        if s.1 {
            let (obj, idx) = Makes::get(&s.0, row, idx);
            (Some(obj), idx)
        } else {
            (None, idx)
        }
    }
}

pub trait Getter {
    type Src: Source;
    type Sel: Selection<Self::Src>;
    fn getter() -> Self::Sel;
}
