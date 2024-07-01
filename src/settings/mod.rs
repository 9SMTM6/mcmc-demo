#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default, PartialEq)]
pub enum Settings {
    EditDistribution(DistrEditKind),
    #[default]
    Default,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default, PartialEq)]
pub enum DistrEditKind {
    MoveCenter {
        idx: usize,
        orig_location: [f32; 2],
    },
    #[default]
    Stateless,
}
