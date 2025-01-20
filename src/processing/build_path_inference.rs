use std::path::PathBuf;


// This module tries to find the `CMakeFiles` directory given a source directory path or a build directory path.
pub fn infer_cmake_cache_dir_path_from_source_dir_path(source_path: &str) -> Result<String, CmakeCachePathInferenceError> {
    let path = PathBuf::from(source_path);

    if !path.exists() {
        return Err(CmakeCachePathInferenceError::SourceDirNotFound);
    }

    if !path.is_dir() {
        return Err(CmakeCachePathInferenceError::SourceDirNotFound);
    }

    // Check if the CMakeFiles directory exists
    let first_candidate = path.join("CMakeFiles");

    if first_candidate.exists() && first_candidate.is_dir() {
        return Ok(first_candidate.to_str().unwrap().to_string());
    }

    // Check if the `build` directory exists
    let second_candidate = path.join("build");

    if second_candidate.exists() && second_candidate.is_dir() {
        // We are in the conna build directory which means that there is one more directory like
        // `macos-armv8-apple-clang-16.0.0` or `emscripten-wasm-advanced-clang-19.emsdk-3.1.57`.

        // Depending on the plaform, choose the first directory with the appropriate root name.
        #[cfg(target_os = "macos")]
        let root_name = "macos";

        #[cfg(target_os = "windows")]
        let root_name = "windows";

        #[cfg(target_os = "linux")]
        let root_name = "linux";

        // List all directories in the `build` directory beginning with the root name
        let mut candidates = std::fs::read_dir(&second_candidate)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with(root_name))
            .collect::<Vec<PathBuf>>();

        // Sort the candidates by name
        candidates.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        // Check if the first candidate has a `CMakeFiles` directory
        let first_candidate = candidates.first().unwrap().join("CMakeFiles");

        if first_candidate.exists() && first_candidate.is_dir() {
            return Ok(first_candidate.to_str().unwrap().to_string());
        }
    }

    Err(CmakeCachePathInferenceError::CmakeCacheNotFound)
}

#[derive(Debug)]
pub enum CmakeCachePathInferenceError {
    SourceDirNotFound,
    BuildDirNotFound,
    CmakeCacheNotFound,
}