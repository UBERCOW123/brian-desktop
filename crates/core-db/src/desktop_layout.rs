//! Default desktop workbench grid seed (8-column grid).

use core_contracts::{WidgetCatalog, WidgetCatalogEntry};

pub const WORKBENCH_LAYOUT_EPOCH: i32 = 4;

/// Fallback rail height when viewport is not measured (seed / reset).
pub const DEFAULT_RAIL_ROW_UNITS: i32 = 8;

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

/// Right-column stack heights (3:3:2 ratio) that sum to [rail_row_units].
pub fn right_stack_heights(rail_row_units: i32) -> (i32, i32, i32, i32, i32, i32) {
    let task_h = (rail_row_units * 3) / 8;
    let note_h = (rail_row_units * 3) / 8;
    let sync_h = rail_row_units - task_h - note_h;
    let task_y = 0;
    let note_y = task_h;
    let sync_y = task_h + note_h;
    (task_h, task_y, note_h, note_y, sync_h, sync_y)
}

pub fn build_template_slots(rail_row_units: i32) -> Vec<DesktopLayoutSlot> {
    let rail = template_rail_row_units(rail_row_units);
    let (task_h, task_y, note_h, note_y, sync_h, sync_y) = right_stack_heights(rail);
    vec![
        DesktopLayoutSlot {
            widget_type: "desktop_timeline",
            pos_x: 0,
            pos_y: 0,
            width: 2,
            height: rail,
        },
        DesktopLayoutSlot {
            widget_type: "desktop_assist",
            pos_x: 2,
            pos_y: 0,
            width: 4,
            height: rail,
        },
        DesktopLayoutSlot {
            widget_type: "task_queue",
            pos_x: 6,
            pos_y: task_y,
            width: 2,
            height: task_h,
        },
        DesktopLayoutSlot {
            widget_type: "note_capture",
            pos_x: 6,
            pos_y: note_y,
            width: 2,
            height: note_h,
        },
        DesktopLayoutSlot {
            widget_type: "desktop_sync",
            pos_x: 6,
            pos_y: sync_y,
            width: 2,
            height: sync_h,
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
    fn right_stack_heights_sum_to_rail_units() {
        for rail in [8, 10, 12, 16] {
            let (task_h, _, note_h, _, sync_h, _) = right_stack_heights(rail);
            assert_eq!(task_h + note_h + sync_h, rail, "rail={rail}");
        }
    }

    #[test]
    fn build_template_slots_default_matches_legacy_ratios() {
        let slots = build_template_slots(8);
        let task = slots.iter().find(|s| s.widget_type == "task_queue").unwrap();
        let note = slots.iter().find(|s| s.widget_type == "note_capture").unwrap();
        let sync = slots.iter().find(|s| s.widget_type == "desktop_sync").unwrap();
        assert_eq!(task.height, 3);
        assert_eq!(note.height, 3);
        assert_eq!(sync.height, 2);
        assert_eq!(sync.pos_y, 6);
    }
}
