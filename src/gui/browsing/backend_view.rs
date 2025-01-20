use iced::{alignment, widget::{button, column, horizontal_rule, row, text, Column}, Element};
use iced::widget::{scrollable, scrollable::Direction, scrollable::Scrollbar};

use crate::processing::AnalyisisResult;

use crate::gui::{style::MONO, Message};


pub fn view<'a>(state: &'a AnalyisisResult, full_name_display: &'a Option<String>) -> Element<'a, Message> {


    if let Some(full_name) = full_name_display {

        let descriptions = column![
            text("#").font(MONO).size(12),
            text("∑ Total [s]").font(MONO).size(12),
            text("AVG Total [ms]").font(MONO).size(12),
        ]
            .spacing(6)
            .align_x(alignment::Alignment::End);

        let summary = state.summary.backend_operation_summaries.get(full_name).unwrap();

        let values = column![
            text(summary.num.to_string()).font(MONO).size(12),
            text(format!("{:.2}", summary.total_time_us as f64 * 1e-6)).font(MONO).size(12),
            text(format!("{:.2}", summary.total_time_us as f64 * 1e-3 / summary.num as f64)).font(MONO).size(12),
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
                .on_press(Message::BrowseBackendFullNameClosed),
            button(text("COPY").font(MONO).size(12))
                .on_press(Message::CopyToClipboard(full_name.clone())),
        ]
            .spacing(12);

        column![
            buttons,
            horizontal_rule(2),
            table,
            horizontal_rule(2),
            text(full_name).font(MONO).size(12),
        ].spacing(4).into()

    } else {
        let limit = 100;

        let mut num_occurences = Column::new()
            .spacing(6)
            .align_x(alignment::Alignment::End)
            .push(text("#").font(MONO).size(12))
            .push(text("").font(MONO).size(4));

        let mut buttons = Column::new()
            .spacing(6)
            .push(text(" ").font(MONO).size(12))
            .push(text("").font(MONO).size(4));

        let mut name_col = Column::new()
            .spacing(6)
            .push(text("Name").font(MONO).size(12))
            .push(text("").font(MONO).size(4));

        let mut total_time_col = Column::new()
            .spacing(6)
            .align_x(alignment::Alignment::End)
            .push(text("∑ Total [s]").font(MONO).size(12))
            .push(text("").font(MONO).size(4));

        let mut total_time_avg_col = Column::new()
            .spacing(6)
            .align_x(alignment::Alignment::End)
            .push(text("AVG Total [ms]").font(MONO).size(12))
            .push(text("").font(MONO).size(4));

        let items_to_display = &state.summary.backend_operation_largest_total_time_indices;

        for key in items_to_display.iter().take(limit) {

            buttons = buttons.push(
                button(text("DISP").font(MONO).size(12))
                    .padding(0)
                    .on_press(Message::BrowseBackendFullNameClicked(key.clone()))
            );

            name_col = name_col.push(
                text(limit_string_name(key)).font(MONO).size(12)
            );

            let summary = state.summary.backend_operation_summaries.get(key).unwrap();

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

            total_time_avg_col = total_time_avg_col.push(
                text(format!("{:.2}", summary.total_time_us as f64 * 1e-3 / summary.num as f64))
                    .font(MONO)
                    .size(12)
            );
        }

        let table = row![
            num_occurences,
            buttons,
            name_col,
            total_time_col,
            total_time_avg_col,
        ]
            .spacing(12);

        let table_content = scrollable(table)
            .width(iced::Length::Fill)
            .direction(Direction::Both { vertical: Scrollbar::new(), horizontal: Scrollbar::new() });

        table_content.into()
    }
}

// Fill the string with `...` in the misdle if it's too long
fn limit_string_name(name: &str) -> String {
    const MAX_LEN: usize = 120;
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
