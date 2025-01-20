use crate::gui::BrowsingSummarySelectedOption;
use crate::processing::AnalyisisResult;
use crate::gui::trace_bar::TraceBar;

use crate::gui::{Message, style::MONO};

use iced::widget::{canvas, horizontal_rule};
use iced::widget::{row, text, button, column, scrollable, scrollable::Direction, scrollable::Scrollbar};
use iced::widget::text::Wrapping;
use iced::widget::text_input;
use iced::widget::text_input::Status;
use iced::widget::TextInput;
use iced::widget::{Column, Row, Text};
use iced::{Element, Theme, Color, Border, Alignment};

const DESC_WIDTH: u16 = 200;
const VALUE_FONT_SIZE: u16 = 12;

fn selectable_text_style(t: &Theme, _s: Status) -> text_input::Style {
    text_input::Style {
        background: Color::TRANSPARENT.into(),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Default::default()
        },
        icon: Color::TRANSPARENT,
        placeholder: Color::TRANSPARENT,
        value: Color::WHITE,
        selection: t.extended_palette().secondary.strong.color
    }
}

fn selectable_text<'a, T: Into<String>>(value: T) -> TextInput<'a, Message> {
    text_input("", &value.into())
        .on_input(Message::Dummy)
        .style(selectable_text_style)
        .size(VALUE_FONT_SIZE)
        .font(MONO)
}

fn summary_row<'a, T: Into<String>>(label: &'static str, value: T) -> Row<'a, Message> {
    Row::new()
        .push(
            Text::new(label).width(DESC_WIDTH)
                .align_x(Alignment::End)
                .align_y(Alignment::Center)
        )
        .push(selectable_text(value))
        .spacing(8)
}

pub fn view<'a>(analysis: &'a AnalyisisResult, selected_option: BrowsingSummarySelectedOption) -> Element<'a, Message> {

    let top_row = row![
        button(text("ALPHA").font(MONO))
            .on_press(Message::BrowseSummaryPaneAlphaClicked),
        button(text("START").font(MONO))
            .on_press(Message::BrowseSummaryPaneStartTimeClicked),
        button(text("END").font(MONO))
            .on_press(Message::BrowseSummaryPaneEndTimeClicked),
        button(text("DUR").font(MONO))
            .on_press(Message::BrowseSummaryPaneDurationClicked),
    ]
        .spacing(4);

    let frontend_secs = format!("{:.2}", analysis.summary.frontend_duration_sec());
    let backend_secs = format!("{:.2}", analysis.summary.backend_duration_sec());
    let backend_single_events_secs = format!("{:.2}", analysis.summary.backend_duration_single_events_sec());
    let inferred_user_time_secs = format!("{:.2}", analysis.summary.inferred_used_time_secs());

    let summary_table = Column::new()
        .padding(8)
        .spacing(4);

    let summary_table: Element<'a, Message> = summary_table
        .push(summary_row("Selected path", &analysis.selected_path))
        .push(summary_row("Resolved path", &analysis.resolved_cmake_files_path))
        .push(summary_row("Total files", analysis.summary.total_files().to_string()))
        .push(summary_row("Total valid", analysis.summary.total_valid_files.to_string()))
        .push(summary_row("Total invalid", analysis.summary.total_invalid_files.to_string()))
        .push(summary_row("Frontend [s]", frontend_secs))
        .push(summary_row("Backend [s]", backend_secs))
        .push(summary_row("Backend single events [s]", backend_single_events_secs))

        .push(summary_row("User time [s]", inferred_user_time_secs))
        .into();

    let description_row_height = 32;

    let mut target_names_row = Column::new()
        .push(
            Text::new("Target")
            .height(description_row_height)
            .wrapping(Wrapping::None)
        );

    let mut target_files_row = Column::new()
        .align_x(Alignment::End)
        .push(
            Text::new("Total files")
            .height(description_row_height)
            .wrapping(Wrapping::None)
        );

    let mut target_frontend_row = Column::new()
        .align_x(Alignment::End)
        .push(
            Text::new("Frontend [s]")
            .height(description_row_height)
            .wrapping(Wrapping::None)
        );

    let mut target_backend_row = Column::new()
        .align_x(Alignment::End)
        .push(
            Text::new("Backend [s]")
            .height(description_row_height)
            .wrapping(Wrapping::None)
        );

    let mut target_first_time_row = Column::new()
        .align_x(Alignment::End)
        .push(
            Text::new("Start [s]")
            .height(description_row_height)
            .wrapping(Wrapping::None)
        );

    let mut duration_graphics = Column::new()
        .align_x(Alignment::End)
        .push(
            Text::new(" ")
            .height(description_row_height)
            .wrapping(Wrapping::None)
        );

    let mut target_last_time_row = Column::new()
        .align_x(Alignment::End)
        .push(
            Text::new("End [s]")
            .height(description_row_height)
            .wrapping(Wrapping::None)
        );

    let target_keys = match selected_option {
        BrowsingSummarySelectedOption::Alpha => &analysis.summary.target_summaries_alpha_order,
        BrowsingSummarySelectedOption::StartTime => &analysis.summary.target_summaries_first_event_indices,
        BrowsingSummarySelectedOption::EndTime => &analysis.summary.target_summaries_last_event_indices,
        BrowsingSummarySelectedOption::Duration => &analysis.summary.target_summaries_largest_duration_indices,
    };

    // for (target_name, target_summary) in &analysis.summary.target_summaries {
    for target_name in target_keys {

        let target_summary = &analysis.summary.target_summaries[target_name];

        let row_height = 24;

        target_names_row = target_names_row.push(
            Text::new(target_name)
            .font(MONO)
            .size(VALUE_FONT_SIZE)
            .wrapping(Wrapping::None)
            .height(row_height)
        );

        target_files_row = target_files_row.push(
            Text::new(target_summary.total_files.to_string())
            .font(MONO)
            .size(VALUE_FONT_SIZE)
            .wrapping(Wrapping::None)
            .height(row_height)
        );

        let target_frontend_secs = format!("{:.2}", target_summary.frontend_duration_sec());

        target_frontend_row = target_frontend_row.push(
            Text::new(target_frontend_secs)
            .font(MONO)
            .size(VALUE_FONT_SIZE)
            .wrapping(Wrapping::None)
            .height(row_height)
        );

        let target_backend_secs = format!("{:.2}", target_summary.backend_duration_sec());

        target_backend_row = target_backend_row.push(
            Text::new(target_backend_secs)
            .font(MONO)
            .size(VALUE_FONT_SIZE)
            .wrapping(Wrapping::None)
            .height(row_height)
        );

        let relative_first_time = (target_summary.first_event_time - analysis.summary.first_event_time) as f64 * 1e-6;
        let relative_last_time = (target_summary.last_event_time - analysis.summary.first_event_time) as f64 * 1e-6;

        let target_first_time_secs = format!("{:.2}", relative_first_time);

        target_first_time_row = target_first_time_row.push(
            Text::new(target_first_time_secs)
            .font(MONO)
            .size(VALUE_FONT_SIZE)
            .wrapping(Wrapping::None)
            .height(row_height)
        );

        let target_last_time_secs = format!("{:.2}", relative_last_time);

        target_last_time_row = target_last_time_row.push(
            Text::new(target_last_time_secs)
            .font(MONO)
            .size(VALUE_FONT_SIZE)
            .wrapping(Wrapping::None)
            .height(row_height)
        );

        let relative_graphics_start = (relative_first_time / analysis.summary.inferred_used_time_secs()).clamp(0.0, 1.0);
        let relative_graphics_end   = (relative_last_time  / analysis.summary.inferred_used_time_secs()).clamp(0.0, 1.0);

        let c = canvas(
            TraceBar{
                start: relative_graphics_start as f32,
                end: relative_graphics_end as f32
            }
        )
            .width(iced::Length::Fill)
            .height(row_height);

        duration_graphics = duration_graphics.push(c);
    }

    let target_table = row![
        target_names_row,
        target_files_row,
        target_frontend_row,
        target_backend_row,
        target_first_time_row,
        duration_graphics,
        target_last_time_row
    ].spacing(12);

    let outer = column![
        top_row,
        horizontal_rule(2),
        scrollable(
            column![
                summary_table,
                target_table,
            ].spacing(4).padding(24)
        ).direction(Direction::Vertical( Scrollbar::new() ))
    ].spacing(4);

    outer.into()
}
