pub struct Seq<A, B>(pub A, pub B);
pub struct Wrap<C>(pub C);
pub struct ColWrap<C>(pub C);
pub struct Holder;
pub struct WithValue<S, A>(pub S, pub A);
pub struct Unit;
pub struct Reps<C>(pub usize, pub C);
