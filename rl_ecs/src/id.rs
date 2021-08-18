pub trait Id {
    type Type;
    type Idx;

    fn get_type(&self) -> Self::Type;
    fn get_idx(&self) -> Self::Idx;
}
