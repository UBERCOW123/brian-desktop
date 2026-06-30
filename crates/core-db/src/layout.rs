//! Dashboard grid helpers — mirrors mobile `DashboardGridConstants` / layout service.

use crate::projections::WidgetInstanceProjection;

pub const COLUMN_COUNT: i32 = 8;
/// Desktop workbench grid can grow beyond 8 columns; widget coords still stored as grid units.
pub const DESKTOP_MAX_COLUMNS: i32 = 48;

pub fn clamp_layout(pos_x: i32, width: i32) -> (i32, i32) {
    clamp_layout_to(pos_x, width, COLUMN_COUNT)
}

pub fn clamp_layout_desktop(pos_x: i32, width: i32) -> (i32, i32) {
    clamp_layout_to(pos_x, width, DESKTOP_MAX_COLUMNS)
}

fn clamp_layout_to(pos_x: i32, width: i32, max_columns: i32) -> (i32, i32) {
    let width = width.clamp(1, max_columns);
    let pos_x = pos_x.clamp(0, max_columns - width);
    (pos_x, width)
}

pub fn next_available_row(instances: &[WidgetInstanceProjection]) -> i32 {
    instances
        .iter()
        .map(|w| w.pos_y + w.height)
        .max()
        .unwrap_or(0)
}

pub fn can_install(
    widget_type: &str,
    allow_multiple: bool,
    instances: &[WidgetInstanceProjection],
) -> bool {
    if allow_multiple {
        return true;
    }
    !instances.iter().any(|w| w.widget_type == widget_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn next_row_after_tallest_widget() {
        let instances = vec![WidgetInstanceProjection {
            id: Uuid::new_v4(),
            widget_type: "clock_widget".into(),
            pos_x: 0,
            pos_y: 4,
            width: 2,
            height: 3,
            config_json: "{}".into(),
        }];
        assert_eq!(next_available_row(&instances), 7);
    }

    #[test]
    fn single_instance_widgets_reject_duplicates() {
        let instances = vec![WidgetInstanceProjection {
            id: Uuid::new_v4(),
            widget_type: "task_queue".into(),
            pos_x: 0,
            pos_y: 0,
            width: 4,
            height: 4,
            config_json: "{}".into(),
        }];
        assert!(!can_install("task_queue", false, &instances));
        assert!(can_install("clock_widget", false, &instances));
    }

    #[test]
    fn desktop_clamp_allows_beyond_core_eight_columns() {
        let (pos_x, width) = clamp_layout_desktop(10, 4);
        assert_eq!(pos_x, 10);
        assert_eq!(width, 4);
    }
}
