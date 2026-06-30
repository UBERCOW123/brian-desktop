//! Default desktop workbench grid seed (8-column grid).

use core_contracts::{WidgetCatalog, WidgetCatalogEntry};

pub const WORKBENCH_LAYOUT_EPOCH: i32 = 5;

/// Fallback rail height when viewport is not measured (seed / reset).
pub const DEFAULT_RAIL_ROW_UNITS: i32 = 8;

/// Widget types dropped from the desktop default template.
pub const REMOVED_TEMPLATE_WIDGETS: &[&str] = &["note_capture"];

pub struct DesktopLayoutSlot {
    pub widget_type: &'static str,
    pub pos_x: i32,
    pub pos_y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn template_rail_row_units(visible_row_units: i32) -> i32 {
    visible_row_units.max(4)
}

/// Default workbench: sync (1) | timeline (2) | assist (3) | tasks (2) across 8 columns.
pub fn build_template_slots(rail_row_units: i32) -> Vec<DesktopLayoutSlot> {
    let rail = template_rail_row_units(rail_row_units);
    vec![
        DesktopLayoutSlot {
            widget_type: "desktop_sync",
            pos_x: 0,
            pos_y: 0,
            width: 1,
            height: rail,
        },
        DesktopLayoutSlot {
            widget_type: "desktop_timeline",
            pos_x: 1,
            pos_y: 0,
            width: 2,
            height: rail,
        },
        DesktopLayoutSlot {
            widget_type: "desktop_assist",
            pos_x: 3,
            pos_y: 0,
            width: 3,
            height: rail,
        },
        DesktopLayoutSlot {
            widget_type: "task_queue",
            pos_x: 6,
            pos_y: 0,
            width: 2,
            height: rail,
        },
    ]
}

/// Cursor-style default at [DEFAULT_RAIL_ROW_UNITS].
pub fn desktop_default_layout() -> Vec<DesktopLayoutSlot> {
    build_template_slots(DEFAULT_RAIL_ROW_UNITS)
}

pub fn catalog_entry<'a>(catalog: &'a WidgetCatalog, widget_type: &str) -> Option<&'a WidgetCatalogEntry> {
    catalog.find(widget_type)
}

pub fn template_slot_for(widget_type: &str, rail_row_units: i32) -> Option<DesktopLayoutSlot> {
    build_template_slots(rail_row_units)
        .into_iter()
        .find(|slot| slot.widget_type == widget_type)
}

pub fn widgets_overlap(widgets: &[crate::projections::WidgetInstanceProjection]) -> bool {
    for i in 0..widgets.len() {
        for j in (i + 1)..widgets.len() {
            let a = &widgets[i];
            let b = &widgets[j];
            if a.pos_x < b.pos_x + b.width
                && a.pos_x + a.width > b.pos_x
                && a.pos_y < b.pos_y + b.height
                && a.pos_y + a.height > b.pos_y
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_template_slots_default_layout_is_sync_timeline_assist_tasks() {
        let slots = build_template_slots(8);
        assert_eq!(slots.len(), 4);

        let sync = slots.iter().find(|s| s.widget_type == "desktop_sync").unwrap();
        assert_eq!(sync.pos_x, 0);
        assert_eq!(sync.width, 1);
        assert_eq!(sync.height, 8);

        let timeline = slots.iter().find(|s| s.widget_type == "desktop_timeline").unwrap();
        assert_eq!(timeline.pos_x, 1);
        assert_eq!(timeline.width, 2);

        let assist = slots.iter().find(|s| s.widget_type == "desktop_assist").unwrap();
        assert_eq!(assist.pos_x, 3);
        assert_eq!(assist.width, 3);

        let tasks = slots.iter().find(|s| s.widget_type == "task_queue").unwrap();
        assert_eq!(tasks.pos_x, 6);
        assert_eq!(tasks.width, 2);

        assert!(slots.iter().all(|s| s.pos_y == 0 && s.height == 8));
    }
}
