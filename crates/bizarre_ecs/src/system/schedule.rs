#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Schedule {
    /// Should be called once before first `Preupdate`
    Init,
    /// Should be called before every `Update`
    Preupdate,
    /// Should be called every frame
    Update,
}
