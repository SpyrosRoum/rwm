use x11rb::{
    connection::Connection,
    errors::ReplyOrIdError,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
    rust_connection::RustConnection,
};

use crate::focus_history::FocusHist;
use common::TagID;

pub(crate) fn update(
    conn: &RustConnection,
    focus: &mut FocusHist,
    tags: Vec<TagID>,
    screen_num: usize,
    border_width: u32,
) -> Result<(), ReplyOrIdError> {
    // TODO Gaps
    // TODO bar space
    let mut windows = focus
        .iter_on_tags_mut(tags)
        .filter(|win| !win.floating)
        .collect::<Vec<_>>();

    let width = conn.setup().roots[screen_num].width_in_pixels as u32;
    let height = conn.setup().roots[screen_num].height_in_pixels as u32;

    if windows.is_empty() {
        return Ok(());
    } else if windows.len() == 1 {
        let win = &windows[0];
        conn.configure_window(
            win.id,
            &ConfigureWindowAux::new()
                .x(0)
                .y(0)
                .border_width(0)
                .width(width)
                .height(height),
        )?;
        conn.flush()?;
        return Ok(());
    }

    let rows = windows.len() as f32 / 2f32;
    let rows = rows.ceil() as u32;

    let win_height = height / rows;
    let win_width = width / windows.len().min(2) as u32;

    for (i, win) in windows.iter_mut().enumerate() {
        let y = (win_height as usize * (i / 2)) as i32;
        let x = if i % 2 == 0 { 0 } else { win_width as i32 };
        let config = ConfigureWindowAux::new()
            .x(x)
            .y(y)
            .width(win_width - (border_width * 2))
            .height(win_height - (border_width * 2))
            .border_width(border_width);

        win.x = x as i16;
        win.y = y as i16;
        win.width = width as u16;
        win.height = height as u16;

        conn.configure_window(win.id, &config)?;
    }

    conn.flush()?;
    Ok(())
}
