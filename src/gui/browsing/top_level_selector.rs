use iced::{widget::{button, column, horizontal_rule, row, text}, Element};

use crate::gui::{AnalyzingFilesState, BrowsingPane, Message, style::MONO};

use super::{summary_view, includes_view, sources_view, frontend_view, backend_view};

pub fn view(state: & AnalyzingFilesState) -> Element<'_, Message> {
    let pane_content = match &state.browsing_pane {
        BrowsingPane::Summary{ selected_option } => summary_view::view(&state.analysis, *selected_option),
        BrowsingPane::Includes{ selected_option } => includes_view::view(&state.analysis, *selected_option),
        BrowsingPane::Sources{ selected_option } => sources_view::view(&state.analysis, *selected_option),
        BrowsingPane::Frontend{ selected_option, full_name_display } => frontend_view::view(&state.analysis, *selected_option, full_name_display),
        BrowsingPane::Backend{ full_name_display } => backend_view::view(&state.analysis, full_name_display),
    };

    const SIDEBAR_WIDTH: u16 = 68;

    let sidebar = column![
        button(text("BACK").font(MONO))
            .width(SIDEBAR_WIDTH)
            .on_press(Message::BrowseCloseClicked)
            .style(iced::widget::button::danger),
        button(text("REF").font(MONO))
            .width(SIDEBAR_WIDTH)
            .on_press(Message::BrowseRefreshClicked),
        horizontal_rule(2),
        button(text("SMRY").font(MONO))
            .width(SIDEBAR_WIDTH)
            .on_press(Message::BrowseTopLevelPaneSummaryClicked),
        button(text("INCS").font(MONO))
            .width(SIDEBAR_WIDTH)
            .on_press(Message::BrowseTopLevelPaneIncludeClicked),
        button(text("SRCS").font(MONO))
            .width(SIDEBAR_WIDTH)
            .on_press(Message::BrowseTopLevelPaneSourceClicked),
        button(text("FRNT").font(MONO))
            .width(SIDEBAR_WIDTH)
            .on_press(Message::BrowseTopLevelPaneFrontendClicked),
        button(text("BCKN")
            .font(MONO))
            .width(SIDEBAR_WIDTH)
            .on_press(Message::BrowseTopLevelPaneBackendClicked),
    ]
        .padding(4)
        .spacing(4)
        .width(SIDEBAR_WIDTH);

    row![
        sidebar,
        pane_content,
    ]
        .spacing(8)
        .padding(4)
        .into()
}
