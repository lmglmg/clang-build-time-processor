use iced::{alignment, widget::{button, column, horizontal_rule, row, text, Column}, Element};
use iced::widget::{scrollable, scrollable::Direction, scrollable::Scrollbar};

use crate::processing::{summary::FrontendOperationKey, AnalyisisResult};

use crate::processing::summary::FrontendOperation;

use crate::gui::{style::MONO, BrowsingFrontendSelectedOption, Message};


pub fn view<'a>(
    state: &'a AnalyisisResult,
    pane_state: BrowsingFrontendSelectedOption,
    full_name_display: &'a Option<FrontendOperationKey>
) -> Element<'a, Message> {

    if let Some(full_name_display) = full_name_display {
        let summary = state.summary.frontend_operation_summaries.get(full_name_display).unwrap();

        let descriptions = column![
            text("#").font(MONO).size(12),
            text("∑ Total [s]").font(MONO).size(12),
            text("∑ Self [s]").font(MONO).size(12),
            text("AVG Total [ms]").font(MONO).size(12),
            text("AVG Self [ms]").font(MONO).size(12),
        ]
            .spacing(6)
            .align_x(alignment::Alignment::End);

        let values = column![
            text(summary.num.to_string()).font(MONO).size(12),
            text(format!("{:.2}", summary.total_time_us as f64 * 1e-6)).font(MONO).size(12),
            text(format!("{:.2}", summary.self_time_us as f64 * 1e-6)).font(MONO).size(12),
            text(format!("{:.2}", summary.total_time_us as f64 * 1e-3 / summary.num as f64)).font(MONO).size(12),
            text(format!("{:.2}", summary.self_time_us as f64 * 1e-3 / summary.num as f64)).font(MONO).size(12),
        ]
            .spacing(6)
            .align_x(alignment::Alignment::End);

        let table = row![
            descriptions,
            values,
        ]
            .spacing(12);

        let buttons = row![
            button(text("BACK").font(MONO).size(12))
                .on_press(Message::BrowseFrontendFullNameClosed),
            button(text("COPY").font(MONO).size(12))
                .on_press(Message::CopyToClipboard(full_name_display.0.clone())),
        ]
            .spacing(12);

        column![
            buttons,
            horizontal_rule(2),
            table,
            horizontal_rule(2),
            text(&full_name_display.0).font(MONO).size(12),
        ].spacing(4).into()
    } else {
    let top_row = row![
        button(text("TOTAL").font(MONO))
            .on_press(Message::BrowseFrontendTotalTimeClicked),
        button(text("SELF").font(MONO))
            .on_press(Message::BrowseFrontendSelfTimeClicked),
    ]
        .spacing(4);

        let limit = 100;

        let mut num_occurences = Column::new()
            .spacing(6)
            .align_x(alignment::Alignment::End)
            .push(text("#").font(MONO).size(12))
            .push(text("").font(MONO).size(4));

        let mut type_col = Column::new()
            .spacing(6)
            .push(text("Type").font(MONO).size(12))
            .push(text("").font(MONO).size(4));

        let mut disp_buttons = Column::new()
            .spacing(6)
            .push(text(" ").font(MONO).size(12))
            .push(text("").font(MONO).size(4));

        let mut name_col = Column::new()
            .spacing(6)
            .push(text("Name").font(MONO).size(12))
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

        let items_to_display = match pane_state {
            BrowsingFrontendSelectedOption::TotalTime => &state.summary.frontend_operation_largest_total_time_indices,
            BrowsingFrontendSelectedOption::SelfTime => &state.summary.frontend_operation_largest_self_time_indices,
        };

        for key in items_to_display.iter().take(limit) {
            name_col = name_col.push(
                text(limit_string_name(&key.0)).font(MONO).size(12)
            );

            let summary = state.summary.frontend_operation_summaries.get(key).unwrap();

            disp_buttons = disp_buttons.push(
                button(text("DISP").font(MONO).size(12))
                    .padding(0)
                    .on_press(Message::BrowseFrontendFullNameClicked(key.clone()))
            );

            let type_text = match key.1 {
                FrontendOperation::CodeGenFunction => "CF",
                FrontendOperation::DebugType => "DT",
                FrontendOperation::InstantiateClass => "IC",
                FrontendOperation::InstantiateFunction => "IF",
                FrontendOperation::ParseClass => "PC",
            };

            type_col = type_col.push(
                text(type_text).font(MONO).size(12)
            );

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

            num_occurences = num_occurences.push(
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
            num_occurences,
            disp_buttons,
            type_col,
            name_col,
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
}

// Fill the string with `...` in the misdle if it's too long
fn limit_string_name(name: &str) -> String {
    const MAX_LEN: usize = 100;
    const FIRST_LETTER_COUNT: usize = 24;

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
