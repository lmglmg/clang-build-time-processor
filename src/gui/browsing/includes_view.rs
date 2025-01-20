use iced::{alignment, widget::{button, column, horizontal_rule, row, text, Column}, Element};
use iced::widget::{scrollable, scrollable::Direction, scrollable::Scrollbar};

use crate::processing::AnalyisisResult;

use crate::gui::{style::MONO, BrowsingIncludesSelectedOption, Message};


pub fn view<'a>(state: &'a AnalyisisResult, pane_state: BrowsingIncludesSelectedOption) -> Element<'a, Message> {

    let top_row = row![
        button(text("TOTAL").font(MONO))
            .on_press(Message::BrowseIncludePaneTotalTimeClicked),
        button(text("SELF").font(MONO))
            .on_press(Message::BrowseIncludePaneSelfTimeClicked),
    ]
        .spacing(4);

    let limit = 100;

    let mut num_includes_col = Column::new()
        .spacing(6)
        .align_x(alignment::Alignment::End)
        .push(text("#").font(MONO).size(12))
        .push(text("").font(MONO).size(4));

    let mut sources_col = Column::new()
        .spacing(6)
        .push(text("Include Path").font(MONO).size(12))
        .push(text("").font(MONO).size(4));

    let mut self_time_col = Column::new()
        .spacing(6)
        .align_x(alignment::Alignment::End)
        .push(text("∑ Self [s]").font(MONO).size(12))
        .push(text("").font(MONO).size(4));

    let mut total_time_col = Column::new()
        .spacing(6)
        .align_x(alignment::Alignment::End)
        .push(text("∑ Total [s]").font(MONO).size(12))
        .push(text("").font(MONO).size(4));

    let mut self_time_avg_col = Column::new()
        .spacing(6)
        .align_x(alignment::Alignment::End)
        .push(text("AVG Self [ms]").font(MONO).size(12))
        .push(text("").font(MONO).size(4));

    let mut total_time_avg_col = Column::new()
        .spacing(6)
        .align_x(alignment::Alignment::End)
        .push(text("AVG Total [ms]").font(MONO).size(12))
        .push(text("").font(MONO).size(4));

    let sources_to_display = match pane_state {
        BrowsingIncludesSelectedOption::TotalTime => &state.summary.frontend_file_largest_time_indices,
        BrowsingIncludesSelectedOption::SelfTime => &state.summary.frontend_file_largest_self_time_indices,
    };

    for source in sources_to_display.iter().take(limit) {
        sources_col = sources_col.push(
            text(limit_string_name(source)).font(MONO).size(12)
        );

        let summary = state.summary.frontend_file_process_summaries.get(source).unwrap();

        self_time_col = self_time_col.push(
            text(format!("{:.2}", summary.self_time_us as f64 * 1e-6))
                .font(MONO)
                .size(12)
        );

        total_time_col = total_time_col.push(
            text(format!("{:.2}", summary.total_time_us as f64 * 1e-6))
                .font(MONO)
                .size(12)
        );

        num_includes_col = num_includes_col.push(
            text(summary.num.to_string())
                .font(MONO)
                .size(12)
        );

        self_time_avg_col = self_time_avg_col.push(
            text(format!("{:.2}", summary.self_time_us as f64 * 1e-3 / summary.num as f64))
                .font(MONO)
                .size(12)
        );

        total_time_avg_col = total_time_avg_col.push(
            text(format!("{:.2}", summary.total_time_us as f64 * 1e-3 / summary.num as f64))
                .font(MONO)
                .size(12)
        );
    }

    let table = row![
        num_includes_col,
        sources_col,
        self_time_col,
        total_time_col,
        self_time_avg_col,
        total_time_avg_col,
    ]
        .spacing(12);

    let content = scrollable(table)
        .width(iced::Length::Fill)
        .direction(Direction::Both { vertical: Scrollbar::new(), horizontal: Scrollbar::new() });

    column![
        top_row,
        horizontal_rule(2),
        content
    ]
        .spacing(4)
        .into()
}

// Fill the string with `...` in the misdle if it's too long
fn limit_string_name(name: &str) -> String {
    const MAX_LEN: usize = 80;
    const FIRST_LETTER_COUNT: usize = 12;

    if name.len() > MAX_LEN {

        let letters_to_skip = name.len() - MAX_LEN;

        let mut result = String::new();

        // take first letter in unicode compatible way
        for c in name.chars().take(FIRST_LETTER_COUNT) {
            result.push(c);
        }

        result.push_str("...");

        for c in name.chars().skip(letters_to_skip) {
            result.push(c);
        }

        result
    } else {
        name.to_string()
    }

}
