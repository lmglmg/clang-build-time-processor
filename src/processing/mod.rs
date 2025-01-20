mod trace_format;
mod build_path_inference;

pub mod summary;

use std::{collections::BTreeMap, path::{Path, PathBuf}};
use build_path_inference::CmakeCachePathInferenceError;
use cpp_demangle::{DemangleOptions, Symbol};
use tracing::{error, trace};

use crate::model::BuildVariant;
use summary::{FrontendOperation, FrontendOperationKey, Summary};

pub struct AnalyisisResult {
    pub selected_path: String,
    pub resolved_cmake_files_path: String,
    pub build_variant: BuildVariant,
    pub summary: Summary,
}

#[derive(Debug)]
pub enum AnalysisError {
    InvalidPath(CmakeCachePathInferenceError),
    Other(String),
}

struct ProfileLocationiInfo {
    path: PathBuf,
    relative_path: String,
    target_name: String,
}

// In a multi config build, expect that there are `Debug`, `DevRelease` and `Debug` directories
// after each `.dir` directory.
fn enumerate_all_trace_files_for_multi_config(cmake_file_dir_path: &str, build_variant_name: &str) -> Result<Vec<ProfileLocationiInfo>, AnalysisError> {
    let mut trace_files = Vec::new();

    // First go over top level dirs in the `CMakeFiles` directory
    // The top level directories end with `.dir`
    // There are other files which need to be ignored
    let Ok(top_entries) = std::fs::read_dir(cmake_file_dir_path) else {
        error!("Failed to read directory: {:?}", cmake_file_dir_path);
        return Err(AnalysisError::Other("Failed to read directory".to_string()));
    };

    for top_entry in top_entries {
        let top_entry = top_entry.map_err(|e| AnalysisError::Other(e.to_string()))?;
        let top_entry_path = top_entry.path();

        if top_entry_path.extension().unwrap_or_default() == "dir" && top_entry_path.is_dir() {

            let target_name = top_entry_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Now to go to the build variant directory
            let candidate = top_entry_path.join(build_variant_name);

            if candidate.is_dir() {
                // Now recursively add all the `.json` files

                for entry in walkdir::WalkDir::new(&candidate) {
                    let entry = entry.map_err(|e| AnalysisError::Other(e.to_string()))?;
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(s) = path.to_str() {

                            // If the file ends with `.json` then we know that it's a trace file
                            if s.ends_with(".json") {
                                // trace_files.push(path.to_path_buf());
                                //
                                let readable_source_path = path
                                    .strip_prefix(&candidate)
                                    .unwrap_or(path)
                                    .to_string_lossy();

                                let readable_source_path = readable_source_path
                                    .strip_suffix(".json")
                                    .unwrap(); // We know it ends with `.json`

                                trace_files.push(ProfileLocationiInfo {
                                    path: path.to_owned(),
                                    relative_path: readable_source_path.to_string(),
                                    target_name: target_name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(trace_files)
}

// For a single config, iterate recursively over the `build_path` and add all the `.json` files
//  - Remove all `CMakeFiles` directories from the final relative path
//  - Remove all `.dir` directories from the final relative path
// Skip only the `compile_commands.json` file
fn enumerate_all_trace_file_for_single_config(build_path: &str) -> Result<Vec<ProfileLocationiInfo>, AnalysisError> {
    let mut trace_files = Vec::new();

    for entry in walkdir::WalkDir::new(build_path) {

        let entry = entry.map_err(|e| AnalysisError::Other(e.to_string()))?;
        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Ignore weird paths
        let Some(s) = path.to_str() else {
            continue;
        };

        // Process only `.json` files
        if !s.ends_with(".json") {
            continue;
        }

        // Skip the `compile_commands.json` file
        if s.ends_with("compile_commands.json") {
            continue;
        }

        // Determine the relative path
        // - Remove the build_path prefix
        // - Remove all `CMakeFiles` directories from the final relative path
        // - Remove all `.dir` directories from the final relative path

        let relative_path = path
            .strip_prefix(build_path)
            .unwrap_or(path)
            .to_string_lossy();

        let mut target_name = "NONE".to_string();

        let mut final_relative_path = Vec::new();

        for part in relative_path.split('/') {
            if part.ends_with(".dir") {
                target_name = part.trim_end_matches(".dir").to_string();
            } else if part != "CMakeFiles" {
                final_relative_path.push(part);
            }
        }

        let final_relative_path = final_relative_path.join("/");

        trace_files.push(ProfileLocationiInfo {
            path: path.to_owned(),
            relative_path: final_relative_path,
            target_name,
        });
    }

    Ok(trace_files)
}


pub fn analyze_path(selected_path: &str, build_variant: BuildVariant) -> Result<AnalyisisResult, AnalysisError> {
    let resolved_cmake_files_path = match build_path_inference::infer_cmake_cache_dir_path_from_source_dir_path(selected_path) {
        Ok(path) => path,
        Err(e) => return Err(AnalysisError::InvalidPath(e)),
    };

    let mut summary: Summary = Default::default();

    let mut demangle_cache = BTreeMap::new();

    let trace_files = match build_variant {
        BuildVariant::SingleConfig => enumerate_all_trace_file_for_single_config(selected_path),
        multi_config_value => enumerate_all_trace_files_for_multi_config(&resolved_cmake_files_path, &multi_config_value.to_string()),
    };

    for trace_file_info in trace_files? {
        process_single_file(
            &mut summary,
            &mut demangle_cache,
            &trace_file_info.target_name,
            &trace_file_info.path,
            &trace_file_info.relative_path
        );
    }

    trace!("Analyzing path: {}", resolved_cmake_files_path);

    process_summary_indices(&mut summary);

    process_include_indices(&mut summary);

    process_source_indices(&mut summary);

    process_frontend_indices(&mut summary);

    process_backend_indices(&mut summary);

    Ok(AnalyisisResult {
        selected_path: selected_path.to_string(),
        resolved_cmake_files_path,
        build_variant,
        summary,
    })
}

fn process_single_file(
    summary: &mut Summary,
    demangle_cache: &mut BTreeMap<String, String>,
    target_name: &str,
    path: &Path,
    readable_path: &str
) {
    let Ok(content) = std::fs::read_to_string(path) else {
        error!("Failed to read file: {:?}", path);
        return;
    };

    let target_summary = summary
        .target_summaries
        .entry(target_name.to_string())
        .or_default();

    let Ok(parsed) = serde_json::from_str::<trace_format::Profile>(&content) else {
        error!("Failed to parse JSON: {:?}", path);
        summary.total_invalid_files += 1;
        return;
    };

    summary.total_valid_files += 1;
    target_summary.total_files += 1;

    if summary.first_event_time == 0 {
        summary.first_event_time = parsed.beginningOfTime;
    } else {
        summary.first_event_time = summary.first_event_time.min(parsed.beginningOfTime);
    }

    let mut frontend_duration_total_us = 0;
    let mut backend_duration_total_us = 0;

    let mut sources = Vec::new();
    let mut frontend_events = Vec::new();
    let mut backend_events = Vec::new();

    for item in parsed.traceEvents.iter() {
        match item.name {
            "Frontend" => {
                frontend_duration_total_us += item.dur.unwrap_or_default();
            },
            "Backend" => {
                backend_duration_total_us += item.dur.unwrap_or_default();
            },
            "Source" => {
                sources.push(item);
            },
            "InstantiateFunction" | "InstantiateClass" | "ParseClass" | "DebugType" | "CodeGenFunction" => {
                frontend_events.push(item);
            },
            "DevirtSCCRepeatedPass" | "OptFunction" => {
                backend_events.push(item);
            }
            _ => {}
        }
    }

    let my_end_of_time = parsed.beginningOfTime + frontend_duration_total_us as u128 + backend_duration_total_us as u128;

    summary.last_event_time = summary.last_event_time.max(my_end_of_time);

    summary.frontend_duration_total_us += frontend_duration_total_us;
    summary.backend_duration_total_us += backend_duration_total_us;

    target_summary.total_frontend_duration_us += frontend_duration_total_us;
    target_summary.total_backend_duration_us += backend_duration_total_us;

    if target_summary.first_event_time == 0 {
        target_summary.first_event_time = parsed.beginningOfTime;
    } else {
        target_summary.first_event_time = target_summary.first_event_time.min(parsed.beginningOfTime);
    }

    target_summary.last_event_time = target_summary.last_event_time.max(my_end_of_time);

    process_sources(summary, sources);

    let source_file_summary = summary.source_file_process_summaries
        .entry(readable_path.to_string())
        .or_default();

    source_file_summary.total_time_us = frontend_duration_total_us + backend_duration_total_us;
    source_file_summary.total_frontend_time_us = frontend_duration_total_us;
    source_file_summary.total_backend_time_us = backend_duration_total_us;

    process_frontend_operations(summary, frontend_events);

    process_backend_operations(summary, demangle_cache, backend_events);
}

fn process_sources(summary: &mut Summary, mut sources: Vec<&trace_format::Event>) {
    // Sort the sources by the timestamp to ensure that the parent will be always processed
    // before the child
    sources.sort_by_key(|s| s.ts);

    // Stack for tracking the timings...
    // First element is the name of the source
    // Second element is the end time of the source
    let mut stack: Vec<(String, u64)> = Vec::new();

    // Second pass to calculate the time spent in each source and subtract the time spent in the parent source
    for source in sources.iter() {
        if let Some(name) = source.args.as_ref().and_then(|a| a.detail.as_ref())
        {
            // Calculate total time and end time
            let time_spent = source.dur.unwrap_or_default();
            let self_end_time = source.ts + time_spent;

            // Add time to the self
            let source_summary = summary
                .frontend_file_process_summaries
                .entry(name.to_string())
                .or_default();

            source_summary.num += 1;
            source_summary.total_time_us += time_spent;

            // Always add the total time as the self time. Other steps will subtract the parent time
            source_summary.self_time_us += time_spent;

            // Subtract the parent self time if it is the parent
            if let Some(top_parent) = stack.last() {
                let parent_end_time = top_parent.1;

                // If the parent end time is greater than the current source end time
                if parent_end_time > self_end_time {
                    let parent_summary = summary
                        .frontend_file_process_summaries
                        .get_mut(&top_parent.0)
                        .unwrap();

                    parent_summary.self_time_us -= time_spent;
                }
            }

            // Pop all parents that are not parents anymore
            while let Some(top_parent) = stack.last() {
                let parent_end_time = top_parent.1;

                // If the parent end time is less than the current source end time
                // then this is not a parent anymore
                if parent_end_time <= self_end_time {
                    break;
                }

                stack.pop();
            }

            // Push the current source to the stack
            stack.push((name.to_string(), self_end_time));
        }
    }
}

fn process_summary_indices(summary: &mut Summary) {
    // Get all the source file names
    summary.target_summaries_alpha_order = summary
        .target_summaries
        .keys()
        .cloned()
        .collect();

    summary.target_summaries_first_event_indices      = summary.target_summaries_alpha_order.clone();
    summary.target_summaries_last_event_indices       = summary.target_summaries_first_event_indices.clone();
    summary.target_summaries_largest_duration_indices = summary.target_summaries_first_event_indices.clone();

    // Sort the sources by the first event time
    summary.target_summaries_first_event_indices.sort_by(|a, b| {
        let a_time = summary.target_summaries.get(a).unwrap().first_event_time;
        let b_time = summary.target_summaries.get(b).unwrap().first_event_time;

        a_time.cmp(&b_time)
    });

    // Sort the sources by the last event time
    summary.target_summaries_last_event_indices.sort_by(|a, b| {
        let a_time = summary.target_summaries.get(a).unwrap().last_event_time;
        let b_time = summary.target_summaries.get(b).unwrap().last_event_time;

        a_time.cmp(&b_time)
    });

    // Sort the sources by the total time (reverse order so that largest is first)
    summary.target_summaries_largest_duration_indices.sort_by(|a, b| {
        let a_time = summary.target_summaries.get(a).unwrap().total_frontend_duration_us + summary.target_summaries.get(a).unwrap().total_backend_duration_us;
        let b_time = summary.target_summaries.get(b).unwrap().total_frontend_duration_us + summary.target_summaries.get(b).unwrap().total_backend_duration_us;

        b_time.cmp(&a_time)
    });
}

fn process_include_indices(summary: &mut Summary) {
    // Get all the source file names
    summary.frontend_file_largest_self_time_indices = summary.frontend_file_process_summaries
        .keys()
        .cloned()
        .collect();
    summary.frontend_file_largest_time_indices = summary.frontend_file_largest_self_time_indices.clone();

    // Sort the sources by the self time (reverse order so that largest is first)
    summary.frontend_file_largest_self_time_indices.sort_by(|a, b| {
        let a_time = summary.frontend_file_process_summaries.get(a).unwrap().self_time_us;
        let b_time = summary.frontend_file_process_summaries.get(b).unwrap().self_time_us;

        b_time.cmp(&a_time)
    });

    // Sort the sources by the total time (reverse order so that largest is first)
    summary.frontend_file_largest_time_indices.sort_by(|a, b| {
        let a_time = summary.frontend_file_process_summaries.get(a).unwrap().total_time_us;
        let b_time = summary.frontend_file_process_summaries.get(b).unwrap().total_time_us;

        b_time.cmp(&a_time)
    });
}

fn process_source_indices(summary: &mut Summary) {
    // Get all the source file names
    summary.source_file_largest_total_time_indices = summary.source_file_process_summaries
        .keys()
        .cloned()
        .collect();
    summary.source_file_largest_frontend_time_indices = summary.source_file_largest_total_time_indices.clone();
    summary.source_file_largest_backend_time_indices = summary.source_file_largest_total_time_indices.clone();

    // Sort the sources by the total time (reverse order so that largest is first)
    summary.source_file_largest_total_time_indices.sort_by(|a, b| {
        let a_time = summary.source_file_process_summaries.get(a).unwrap().total_time_us;
        let b_time = summary.source_file_process_summaries.get(b).unwrap().total_time_us;

        b_time.cmp(&a_time)
    });

    // Sort the sources by the frontend time (reverse order so that largest is first)
    summary.source_file_largest_frontend_time_indices.sort_by(|a, b| {
        let a_time = summary.source_file_process_summaries.get(a).unwrap().total_frontend_time_us;
        let b_time = summary.source_file_process_summaries.get(b).unwrap().total_frontend_time_us;

        b_time.cmp(&a_time)
    });

    // Sort the sources by the backend time (reverse order so that largest is first)
    summary.source_file_largest_backend_time_indices.sort_by(|a, b| {
        let a_time = summary.source_file_process_summaries.get(a).unwrap().total_backend_time_us;
        let b_time = summary.source_file_process_summaries.get(b).unwrap().total_backend_time_us;

        b_time.cmp(&a_time)
    });
}

fn process_frontend_operations(summary: &mut Summary, mut frontend_events: Vec<&trace_format::Event>) {
    // Sort the sources by the timestamp to ensure that the parent will be always processed
    // before the child
    frontend_events.sort_by_key(|s| s.ts);

    let mut stack : Vec<(FrontendOperationKey, u64)> = Vec::new();

    for item in frontend_events.iter() {
        let op_type: FrontendOperation = match item.name {
            "InstantiateFunction" => FrontendOperation::InstantiateFunction,
            "InstantiateClass" => FrontendOperation::InstantiateClass,
            "ParseClass" => FrontendOperation::ParseClass,
            "DebugType" => FrontendOperation::DebugType,
            "CodeGenFunction" => FrontendOperation::CodeGenFunction,
            _ => continue,
        };

        let op_arg = item
            .args
            .as_ref()
            .and_then(|a| a.detail.as_ref())
            .unwrap()
            .to_string();

        let key = (op_arg, op_type);

        let time_spent = item.dur.unwrap_or_default();

        let self_end_time = item.ts + time_spent;

        let source_summary = summary
            .frontend_operation_summaries
            .entry(key.clone())
            .or_default();

        source_summary.num += 1;

        source_summary.total_time_us += time_spent;

        // Always add the total time as the self time. Other steps will subtract the parent time
        source_summary.self_time_us += time_spent;

        // Subtract the parent self time if it is the parent
        if let Some(top_parent) = stack.last() {
            let parent_end_time = top_parent.1;

            // If the parent end time is greater than the current source end time
            if parent_end_time > self_end_time {
                let parent_summary = summary
                    .frontend_operation_summaries
                    .get_mut(&top_parent.0)
                    .unwrap();

                parent_summary.self_time_us -= time_spent;
            }
        }

        // Pop all parents that are not parents anymore
        while let Some(top_parent) = stack.last() {
            let parent_end_time = top_parent.1;

            // If the parent end time is less than the current source end time
            // then this is not a parent anymore
            if parent_end_time <= self_end_time {
                break;
            }

            stack.pop();
        }

        // Push the current source to the stack
        stack.push((key, self_end_time));
    }
}

fn process_frontend_indices(summary: &mut Summary) {
    // Get all the source file names
    summary.frontend_operation_largest_total_time_indices = summary
        .frontend_operation_summaries
        .keys()
        .cloned()
        .collect();

    summary.frontend_operation_largest_self_time_indices = summary.frontend_operation_largest_total_time_indices.clone();

    // Sort the sources by the self time (reverse order so that largest is first)
    summary.frontend_operation_largest_self_time_indices.sort_by(|a, b| {
        let a_time = summary.frontend_operation_summaries.get(a).unwrap().self_time_us;
        let b_time = summary.frontend_operation_summaries.get(b).unwrap().self_time_us;

        b_time.cmp(&a_time)
    });

    // Sort the sources by the total time (reverse order so that largest is first)
    summary.frontend_operation_largest_total_time_indices.sort_by(|a, b| {
        let a_time = summary.frontend_operation_summaries.get(a).unwrap().total_time_us;
        let b_time = summary.frontend_operation_summaries.get(b).unwrap().total_time_us;

        b_time.cmp(&a_time)
    });
}

fn process_backend_operations(
    summary: &mut Summary,
    demangle_cache: &mut BTreeMap<String, String>,
    backend_events: Vec<&trace_format::Event>
) {
    // No need to sort the events, just go over them, demangle them and add them to the summary

    for item in backend_events.iter() {
        let op_name = item
            .args
            .as_ref()
            .and_then(|a| a.detail.as_ref())
            .unwrap()
            .to_string();

        // At this point the name is still surrounded with parenthesis. Remove them.
        let op_name = op_name.trim_start_matches('(').trim_end_matches(')').to_string();

        let demangle_options = DemangleOptions::default();

        // Demangle the name, check the cache first
        let demangled = match demangle_cache.get(&op_name) {
            Some(d) => d.clone(),
            None => {
                let demangled = Symbol::new(&op_name)
                    .ok() // Just convert to an option... do not care about the error
                    .and_then(|s| s.demangle(&demangle_options).ok()) // Demangle and convert to an option to handle errors
                    .unwrap_or(op_name.clone()); // Just return the mangled name if demangling fails

                demangle_cache.insert(op_name.clone(), demangled.clone());
                demangled
            }
        };

        let time_spent = item.dur.unwrap_or_default();

        let source_summary = summary
            .backend_operation_summaries
            .entry(demangled)
            .or_default();

        source_summary.num += 1;
        source_summary.total_time_us += time_spent;

        summary.backend_duration_single_events_us += time_spent;
    }
}

fn process_backend_indices(summary: &mut Summary) {
    // Get all the source file names
    summary.backend_operation_largest_total_time_indices = summary
        .backend_operation_summaries
        .keys()
        .cloned()
        .collect();

    // Sort the sources by the total time (reverse order so that largest is first)
    summary.backend_operation_largest_total_time_indices.sort_by(|a, b| {
        let a_time = summary.backend_operation_summaries.get(a).unwrap().total_time_us;
        let b_time = summary.backend_operation_summaries.get(b).unwrap().total_time_us;

        b_time.cmp(&a_time)
    });
}
