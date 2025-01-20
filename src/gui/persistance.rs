use serde::{Deserialize, Serialize};

pub type BuildVariant = crate::model::BuildVariant;

pub struct Persistance {
    persistance_file_path: String,
    last_files: Vec<String>,
    last_selected_build_variant: BuildVariant,
    is_functional: bool,
}

impl Persistance {

    // Calculate cross-platform cache directory
    fn cache_dir() -> Option<String> {

        // This is a cross-platform way to get the cache directory
        directories::ProjectDirs::from("org", "lmglmg", "Clang Build Time Processor")
            .map(|base_dirs| base_dirs.cache_dir().to_str().unwrap().to_string())
    }

    pub fn new() -> Self {
        if let Some(cache_dir_path) = Persistance::cache_dir() {
            let persistance_file_path = format!("{}/persistance.json", cache_dir_path);

            // Create cache directory if it doesn't exist
            let is_functional_dir = std::fs::create_dir_all(&cache_dir_path).is_ok();

            if !is_functional_dir {
                return Persistance {
                    persistance_file_path,
                    last_files: Vec::new(),
                    last_selected_build_variant: BuildVariant::Release,
                    is_functional: false,
                };
            }

             // Check that the persistance file can be read
            let parsed_persistance = PersistanceFormat::from_path(&persistance_file_path);

            if let Ok(persistance) = parsed_persistance {
                Persistance {
                    persistance_file_path,
                    last_files: persistance.last_paths,
                    last_selected_build_variant: persistance.last_selected_build_variant,
                    is_functional: true,
                }
            } else {
                Persistance {
                    persistance_file_path,
                    last_files: Vec::new(),
                    last_selected_build_variant: BuildVariant::Release,
                    is_functional: false,
                }
            }
        } else {
            Persistance {
                persistance_file_path: String::new(),
                last_files: Vec::new(),
                last_selected_build_variant: BuildVariant::Release,
                is_functional: false,
            }
        }
    }

    pub fn push_last_path(&mut self, path: &str, build_variant: BuildVariant) {
        // Only add unique paths
        if self.last_files.contains(&path.to_string()) {
            return;
        }
        self.last_files.push(path.to_string());

        let persistance = PersistanceFormat {
            last_paths: self.last_files.clone(),
            last_selected_build_variant: build_variant,
        };

        let persistance_json = serde_json::to_string(&persistance).unwrap();

        std::fs::write(&self.persistance_file_path, persistance_json).unwrap();
    }

    pub fn build_variant(&self) -> BuildVariant {
        self.last_selected_build_variant
    }

    pub fn push_build_variant(&mut self, build_variant: BuildVariant) {
        let persistance = PersistanceFormat {
            last_paths: self.last_files.clone(),
            last_selected_build_variant: build_variant,
        };

        let persistance_json = serde_json::to_string(&persistance).unwrap();

        std::fs::write(&self.persistance_file_path, persistance_json).unwrap();
    }

    pub fn last_paths(& self) -> Vec<&str> {
        self.last_files.iter().map(|s| s.as_str()).collect()
    }

    pub fn remove_last_path(&mut self, path: &str) {
        self.last_files.retain(|p| p != path);

        let persistance = PersistanceFormat {
            last_paths: self.last_files.clone(),
            last_selected_build_variant: self.last_selected_build_variant,
        };

        let persistance_json = serde_json::to_string(&persistance).unwrap();

        std::fs::write(&self.persistance_file_path, persistance_json).unwrap();
    }
}

#[derive(Deserialize, Serialize)]
struct PersistanceFormat {
    last_paths: Vec<String>,
    last_selected_build_variant: BuildVariant,
}

enum PersistanceError {
    FileError(String),
    ParseError(serde_json::Error),
}

impl PersistanceFormat {
    fn from_path(path: &str) -> Result<Self, PersistanceError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PersistanceError::FileError(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| PersistanceError::ParseError(e))
    }
}
