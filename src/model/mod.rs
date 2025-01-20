use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone, Debug)]
pub enum BuildVariant {
    Debug,
    DevRelease,
    Release,
    SingleConfig,
}

impl BuildVariant {
    pub const ALL: [BuildVariant; 4] = [
        BuildVariant::Debug,
        BuildVariant::DevRelease,
        BuildVariant::Release,
        BuildVariant::SingleConfig
    ];
}

impl std::fmt::Display for BuildVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildVariant::Debug => write!(f, "Debug"),
            BuildVariant::DevRelease => write!(f, "DevRelease"),
            BuildVariant::Release => write!(f, "Release"),
            BuildVariant::SingleConfig => write!(f, "SingleConfig"),
        }
    }
}
