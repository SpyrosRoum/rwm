use x11rb::{
    connection::Connection,
    errors::ReplyOrIdError,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
    rust_connection::RustConnection,
};

use crate::{focus_history::FocusHist, rect::Rect};
use common::TagId;

pub(crate) fn update(
    conn: &RustConnection,
    focus: &mut FocusHist,
    tags: Vec<TagId>,
    rect: &Rect,
    border_width: u32,
    gap: u32,
) -> Result<(), ReplyOrIdError> {
    // TODO bar space
    let mut windows = focus
        .iter_on_tags_mut(tags)
        .filter(|win| !win.floating)
        .collect::<Vec<_>>();

    let width = rect.width as u32;
    let height = rect.height as u32;

    if windows.is_empty() {
        return Ok(());
    } else if windows.len() == 1 {
        let win = &windows[0];
        conn.configure_window(
            win.id,
            &ConfigureWindowAux::new()
                .x(rect.x as i32)
                .y(rect.y as i32)
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

    let ww = win_width - (border_width * 2) - gap * 2;
    let wh = win_height - (border_width * 2) - gap * 2;
    for (i, win) in windows.iter_mut().enumerate() {
        let y = (if i == 0 {
            gap
        } else {
            win_height * (i / 2) as u32 + gap
        } + rect.y as u32) as i32;
        let x = (if i % 2 == 0 { gap } else { win_width + gap } as i32) + rect.x as i32;

        let config = ConfigureWindowAux::new()
            .x(x)
            .y(y)
            .width(ww)
            .height(wh)
            .border_width(border_width);

        conn.configure_window(win.id, &config)?;

        win.x = x as i16;
        win.y = y as i16;
        win.width = ww as u16;
        win.height = wh as u16;
    }

    Ok(())
}
