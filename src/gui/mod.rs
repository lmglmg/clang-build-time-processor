mod browsing;
mod persistance;
mod style;
mod trace_bar;

use iced::widget::{button, combo_box, text_input, Row};
use iced::widget::{Column, Text};
use iced::{Element, Task};

use persistance::Persistance;
use tracing::{event, Level};

use crate::processing::summary::FrontendOperationKey;
use crate::processing::AnalyisisResult;


pub struct App {
    cross_state_cache: AppCrossStateCache,
    state: AppState,
}

pub struct AppCrossStateCache {
    persistance: persistance::Persistance,
    errors: Vec<String>,
}

pub enum AppState {
    LookingForFiles {
        current_path: String,
        build_variants: combo_box::State<persistance::BuildVariant>,
        build_variant: persistance::BuildVariant,
    },
    AnalyzingFiles(AnalyzingFilesState),
}

pub struct AnalyzingFilesState {
    analysis: AnalyisisResult,
    browsing_pane: BrowsingPane,
}

pub enum BrowsingPane {
    Summary{
        selected_option: BrowsingSummarySelectedOption,
    },
    Includes {
        selected_option: BrowsingIncludesSelectedOption,
    },
    Sources {
        selected_option: BrowsingSourcesSelectedOption,
    },
    Frontend {
        selected_option: BrowsingFrontendSelectedOption,
        full_name_display: Option<FrontendOperationKey>,
    },
    Backend {
        full_name_display: Option<String>,
    }
}

#[derive(Copy, Clone)]
pub enum BrowsingSummarySelectedOption {
    Alpha,
    StartTime,
    EndTime,
    Duration,
}

#[derive(Copy, Clone)]
pub enum BrowsingIncludesSelectedOption {
    TotalTime,
    SelfTime,
}

#[derive(Copy, Clone)]
pub enum BrowsingSourcesSelectedOption {
    TotalTime,
    FrontendTime,
    BackendTime,
}

#[derive(Copy, Clone)]
pub enum BrowsingFrontendSelectedOption {
    TotalTime,
    SelfTime,
}

#[derive(Debug, Clone)]
pub enum Message {
    BrowseClicked,
    BrowseInputChanged(String),
    BrowseLastItemOpen(String),
    BrowserRemoveLastItem(String),
    BrowseSelectedBuildVariant(persistance::BuildVariant),

    // Browsing top level pane
    BrowseRefreshClicked,
    BrowseCloseClicked,
    BrowseTopLevelPaneSummaryClicked,
    BrowseTopLevelPaneIncludeClicked,
    BrowseTopLevelPaneSourceClicked,
    BrowseTopLevelPaneFrontendClicked,
    BrowseTopLevelPaneBackendClicked,

    // Browsing Summary Pane
    BrowseSummaryPaneAlphaClicked,
    BrowseSummaryPaneStartTimeClicked,
    BrowseSummaryPaneEndTimeClicked,
    BrowseSummaryPaneDurationClicked,

    // Browsing Include Pane
    BrowseIncludePaneTotalTimeClicked,
    BrowseIncludePaneSelfTimeClicked,

    // Browsing Sources Pane
    BrowseSourcePaneTotalTimeClicked,
    BrowseSourcePaneFrontendTimeClicked,
    BrowseSourcePaneBackendTimeClicked,

    // Frontend Pane
    BrowseFrontendTotalTimeClicked,
    BrowseFrontendSelfTimeClicked,
    BrowseFrontendFullNameClicked(FrontendOperationKey),
    BrowseFrontendFullNameClosed,

    // Backend Pane
    BrowseBackendFullNameClicked(String),
    BrowseBackendFullNameClosed,

    // Used for all text inputs which do nothing.
    // This enables copy-pasting from an input field, but not connecting it to any action.
    #[allow(dead_code)]
    Dummy(String),

    // Global message to copy to clipboard
    CopyToClipboard(String),
}

impl App {

    fn default_state(persistance: &Persistance) -> AppState {
        let last_selected_build_variant = persistance.build_variant();

        AppState::LookingForFiles {
            current_path: String::new(),
            build_variants: combo_box::State::new(persistance::BuildVariant::ALL.into()),
            build_variant: last_selected_build_variant,
        }
    }

    pub fn new() -> (Self, Task<Message>) {
        let persistance = persistance::Persistance::new();

        (
            App {
                state: Self::default_state(&persistance),
                cross_state_cache: AppCrossStateCache {
                    persistance,
                    errors: Vec::new(),
                },
            },
            Task::none()
        )
    }

    fn open_path(&mut self, path: &str, build_variant: persistance::BuildVariant) {

        let analysis = crate::processing::analyze_path(path, build_variant);

        match analysis {
            Ok(analysis) => {
                event!(Level::INFO, "Analysis complete");
                self.state = AppState::AnalyzingFiles(
                    AnalyzingFilesState {
                        analysis,
                        browsing_pane: BrowsingPane::Summary {
                            selected_option: BrowsingSummarySelectedOption::Alpha,
                        }
                    }
                );
            }
            Err(e) => {
                event!(Level::ERROR, "Analysis error: {:?}", e);
                self.cross_state_cache.errors.push(format!("Analysis error: {:?}", e));
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::BrowseInputChanged(path) => {
                if let AppState::LookingForFiles { current_path, build_variants: _, build_variant: _ } = &mut self.state {
                    *current_path = path;
                }
            }
            Message::BrowseLastItemOpen(path) => {
                if let AppState::LookingForFiles { current_path: _, build_variants: _, build_variant } = &self.state {
                    self.open_path(&path, *build_variant);
                }
            }
            Message::BrowserRemoveLastItem(path) => {
                self.cross_state_cache.persistance.remove_last_path(&path);
            }
            Message::BrowseSelectedBuildVariant(build_variant) => {
                if let AppState::LookingForFiles { current_path: _, build_variants: _, build_variant: current_build_variant } = &mut self.state {
                    *current_build_variant = build_variant;
                    self.cross_state_cache.persistance.push_build_variant(build_variant);
                }
            }
            Message::BrowseClicked => {
                if let AppState::LookingForFiles { current_path, build_variants: _, build_variant } = &self.state {

                    // Check if directory exists
                    if !std::path::Path::new(&current_path).exists() {
                        self.cross_state_cache.errors.push(format!("Path {} does not exist", current_path));
                        event!(Level::ERROR, "Path does not exist: {}", current_path);
                        return Task::none();
                    }

                    self.cross_state_cache.persistance.push_last_path(current_path, persistance::BuildVariant::Release);

                    self.open_path(&current_path.clone(), *build_variant);
                }
            }

            Message::BrowseCloseClicked => {
                self.state = Self::default_state(&self.cross_state_cache.persistance);
            }
            Message::BrowseRefreshClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    let path = state.analysis.selected_path.clone();
                    let build_variant = state.analysis.build_variant;
                    self.open_path(&path, build_variant);
                }
            }

            Message::BrowseTopLevelPaneSummaryClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    state.browsing_pane = BrowsingPane::Summary {
                        selected_option: BrowsingSummarySelectedOption::Alpha,
                    }
                }
            }

            // Top level pane
            Message::BrowseTopLevelPaneIncludeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    state.browsing_pane = BrowsingPane::Includes {
                        selected_option: BrowsingIncludesSelectedOption::TotalTime,
                    }
                }
            }
            Message::BrowseTopLevelPaneSourceClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    state.browsing_pane = BrowsingPane::Sources {
                        selected_option: BrowsingSourcesSelectedOption::TotalTime,
                    }
                }
            }
            Message::BrowseTopLevelPaneFrontendClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    state.browsing_pane = BrowsingPane::Frontend {
                        selected_option: BrowsingFrontendSelectedOption::TotalTime,
                        full_name_display: None,
                    }
                }
            }
            Message::BrowseTopLevelPaneBackendClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    state.browsing_pane = BrowsingPane::Backend {
                        full_name_display: None,
                    }
                }
            }

            // Summary pane
            Message::BrowseSummaryPaneAlphaClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Summary{ selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingSummarySelectedOption::Alpha;
                    }
                }
            }
            Message::BrowseSummaryPaneStartTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Summary{ selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingSummarySelectedOption::StartTime;
                    }
                }
            }
            Message::BrowseSummaryPaneEndTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Summary{ selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingSummarySelectedOption::EndTime;
                    }
                }
            }
            Message::BrowseSummaryPaneDurationClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Summary{ selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingSummarySelectedOption::Duration;
                    }
                }
            }

            // Include pane
            Message::BrowseIncludePaneTotalTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Includes { selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingIncludesSelectedOption::TotalTime;
                    }
                }
            }
            Message::BrowseIncludePaneSelfTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Includes { selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingIncludesSelectedOption::SelfTime;
                    }
                }
            }

            // Source pane
            Message::BrowseSourcePaneTotalTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Sources { selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingSourcesSelectedOption::TotalTime;
                    }
                }
            }
            Message::BrowseSourcePaneFrontendTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Sources { selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingSourcesSelectedOption::FrontendTime;
                    }
                }
            }
            Message::BrowseSourcePaneBackendTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Sources { selected_option } = &mut state.browsing_pane {
                        *selected_option = BrowsingSourcesSelectedOption::BackendTime;
                    }
                }
            }

            // Frontend pane
            Message::BrowseFrontendTotalTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Frontend { selected_option, full_name_display: _ } = &mut state.browsing_pane {
                        *selected_option = BrowsingFrontendSelectedOption::TotalTime;
                    }
                }
            }
            Message::BrowseFrontendSelfTimeClicked => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Frontend { selected_option, full_name_display: _ } = &mut state.browsing_pane {
                        *selected_option = BrowsingFrontendSelectedOption::SelfTime;
                    }
                }
            }
            Message::BrowseFrontendFullNameClicked(full_name) => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Frontend { selected_option: _, full_name_display } = &mut state.browsing_pane {
                        *full_name_display = Some(full_name);
                    }
                }
            }
            Message::BrowseFrontendFullNameClosed => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    if let BrowsingPane::Frontend { selected_option: _, full_name_display } = &mut state.browsing_pane {
                        *full_name_display = None;
                    }
                }
            }

            // Backend pane
            Message::BrowseBackendFullNameClicked(full_name) => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    state.browsing_pane = BrowsingPane::Backend {
                        full_name_display: Some(full_name),
                    }
                }
            }
            Message::BrowseBackendFullNameClosed => {
                if let AppState::AnalyzingFiles(state) = &mut self.state {
                    state.browsing_pane = BrowsingPane::Backend {
                        full_name_display: None,
                    }
                }
            }

            // Ignore this message always
            Message::Dummy(_) => {}

            // Copy to clipboard
            Message::CopyToClipboard(text) => {
                return iced::clipboard::write(text);
            }
        };

        Task::none()
    }

    pub fn title(&self) -> String {
        "Clang Build Time Processor".into()
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            AppState::LookingForFiles{ current_path, build_variants, build_variant } => {

                let recent_files = self.cross_state_cache.persistance.last_paths();

                let custom_input = text_input("Enter source path", current_path)
                    .font(style::MONO)
                    .on_input(Message::BrowseInputChanged);

                let build_variant_selector = combo_box(
                    build_variants,
                    "Select the build variant",
                    Some(build_variant),
                    Message::BrowseSelectedBuildVariant,
                )
                    .width(140);

                let first_row = Row::new()
                    .push(custom_input)
                    .push(build_variant_selector)
                    .push(
                        button("Browse")
                            .on_press(Message::BrowseClicked)
                    )
                    .spacing(6);

                let mut last_files_component = Column::new();

                for file in recent_files {
                    let row = Row::new()
                        .push(button("DEL").style(iced::widget::button::danger).on_press_with(|| Message::BrowserRemoveLastItem(file.into())))
                        .push(button("OPEN").on_press_with(|| Message::BrowseLastItemOpen(file.into())))
                        .push(Text::new(file).font(style::MONO))
                        .spacing(6);
                    last_files_component = last_files_component.push(row);
                }

                Column::new()
                    .push(first_row)
                    .push(last_files_component)
                    .spacing(6)
                    .padding(10)
                    .into()
            }
            AppState::AnalyzingFiles(state) => {
                browsing::top_level_selector::view(state)
            }
        }
    }
}
