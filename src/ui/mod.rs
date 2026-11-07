pub mod interactive_lines_editor;
mod keys_combo_box;
mod line_edit_dialog;
use ratatui::layout::Rect;

fn centered_popup_rect(area: Rect, h: u16, v: u16) -> Rect {
    let v_padding = area.height.saturating_sub(v) / 2;
    let h_padding = area.width.saturating_sub(h) / 2;
    if v_padding == 0 || h_padding == 0 {
        return Rect::default();
    }
    Rect::new(area.x + h_padding, area.y + v_padding, h, v)
}
