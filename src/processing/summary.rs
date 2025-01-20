use std::collections::BTreeMap;

#[derive(Default)]
pub struct Summary {
    pub total_valid_files: usize,
    pub total_invalid_files: usize,
    pub frontend_duration_total_us: u64,
    pub backend_duration_total_us: u64,
    pub backend_duration_single_events_us: u64,
    pub first_event_time: u128,
    pub last_event_time: u128,

    pub target_summaries: BTreeMap<String, TopLevelTargetSummary>,
    pub target_summaries_alpha_order: Vec<String>,
    pub target_summaries_first_event_indices: Vec<String>,
    pub target_summaries_last_event_indices: Vec<String>,
    pub target_summaries_largest_duration_indices: Vec<String>,

    pub frontend_file_process_summaries: BTreeMap<String, FrontendFileProcessSummary>,
    pub frontend_file_largest_self_time_indices: Vec<String>,
    pub frontend_file_largest_time_indices: Vec<String>,

    pub source_file_process_summaries: BTreeMap<String, SourceFileProcessSummary>,
    pub source_file_largest_total_time_indices: Vec<String>,
    pub source_file_largest_frontend_time_indices: Vec<String>,
    pub source_file_largest_backend_time_indices: Vec<String>,

    pub frontend_operation_summaries: FrontendOperationSummaries,
    pub frontend_operation_largest_total_time_indices: Vec<FrontendOperationKey>,
    pub frontend_operation_largest_self_time_indices: Vec<FrontendOperationKey>,

    pub backend_operation_summaries: BTreeMap<String, BackendOperationSummaries>,
    pub backend_operation_largest_total_time_indices: Vec<String>,
}

impl Summary {
    pub fn total_files(&self) -> usize {
        self.total_valid_files + self.total_invalid_files
    }

    pub fn frontend_duration_sec(&self) -> f64 {
        self.frontend_duration_total_us as f64 * 1e-6
    }

    pub fn backend_duration_sec(&self) -> f64 {
        self.backend_duration_total_us as f64 * 1e-6
    }

    pub fn backend_duration_single_events_sec(&self) -> f64 {
        self.backend_duration_single_events_us as f64 * 1e-6
    }

    pub fn inferred_used_time_secs(&self) -> f64 {
        if self.first_event_time != 0 && self.last_event_time != 0 {
            (self.last_event_time - self.first_event_time) as f64 * 1e-6
        } else {
            0.0
        }
    }
}

#[derive(Default)]
pub struct TopLevelTargetSummary {
    pub total_files: usize,
    pub total_frontend_duration_us: u64,
    pub total_backend_duration_us: u64,
    pub first_event_time: u128,
    pub last_event_time: u128,
}

impl TopLevelTargetSummary {
    pub fn frontend_duration_sec(&self) -> f64 {
        self.total_frontend_duration_us as f64 * 1e-6
    }

    pub fn backend_duration_sec(&self) -> f64 {
        self.total_backend_duration_us as f64 * 1e-6
    }
}

#[derive(Default)]
pub struct FrontendFileProcessSummary {
    pub total_time_us: u64,
    pub self_time_us: u64,
    pub num: usize,
}

#[derive(Default)]
pub struct SourceFileProcessSummary {
    pub total_time_us: u64,
    pub total_frontend_time_us: u64,
    pub total_backend_time_us: u64,
}

// Frontend operations which are specific to some class or function
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum FrontendOperation {
    CodeGenFunction,
    DebugType,
    InstantiateClass,
    InstantiateFunction,
    ParseClass,
}

#[derive(Default)]
pub struct FrontendOperationSummary {

    pub total_time_us: u64,
    pub self_time_us: u64,
    pub num: usize,
}

pub type FrontendOperationKey = (String, FrontendOperation);

pub type FrontendOperationSummaries = BTreeMap<FrontendOperationKey, FrontendOperationSummary>;

#[derive(Default)]
pub struct BackendOperationSummaries {
    pub total_time_us: u64,
    pub num: usize,
}
