use x11rb::{
    errors::ReplyOrIdError,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
    rust_connection::RustConnection,
};

use crate::{rect::Rect, windows_history::WindowsHistory};
use common::TagId;

pub(crate) fn update(
    conn: &RustConnection,
    focus: &mut WindowsHistory,
    tags: Vec<TagId>,
    rect: &Rect,
    border_width: u32,
    gap: u32,
) -> Result<(), ReplyOrIdError> {
    // First window gets 60% of the screen (the left side), the rest stack on the side
    // ToDo bar space.
    let width = rect.width as u32;
    let height = rect.height as u32;

    let master_width = width * 60 / 100;
    let slave_width = width - master_width;

    let mut windows = focus.iter_on_tags_mut(tags).filter(|win| !win.floating);

    let master_win = windows.next();
    if master_win.is_none() {
        // No windows, nothing to do
        return Ok(());
    }
    let master_win = master_win.unwrap();
    let mut stack = windows.collect::<Vec<_>>();

    // We don't want gap if we only have one window
    let gap = !stack.is_empty() as i32 * gap as i32;
    // Same for border width
    let bw = !stack.is_empty() as u32 * border_width;
    let (mw, master_height) = {
        if stack.is_empty() {
            (width, height)
        } else {
            (
                master_width - (border_width * 2) - gap as u32,
                height - (border_width * 2) - (gap * 2) as u32,
            )
        }
    };

    let master_config = ConfigureWindowAux::new()
        .width(mw)
        .height(master_height)
        .x(rect.x as i32 + gap)
        .y(rect.y as i32 + gap)
        .border_width(bw);
    conn.configure_window(master_win.id, &master_config)?;
    master_win.x = rect.x + gap as i16;
    master_win.y = rect.y + gap as i16;
    master_win.width = mw as u16;
    master_win.height = master_height as u16;

    if let Some(slave_height) = height.checked_div(stack.len() as u32) {
        // If we get here it means there are slave windows
        let x = rect.x as i32 + master_width as i32 + gap * 2; // gap * 2 for both the slave window and the master window

        for (i, win) in stack.iter_mut().enumerate() {
            let y = if i == 0 {
                gap
            } else {
                slave_height as i32 * (i as i32) + gap
            } + rect.y as i32;
            let width = slave_width - (border_width * 2) - (gap * 3) as u32;
            let height = slave_height - (border_width * 2) - (gap * 2) as u32;

            conn.configure_window(
                win.id,
                &ConfigureWindowAux::new()
                    .x(x)
                    .y(y)
                    .width(width)
                    .height(height)
                    .border_width(border_width),
            )?;

            win.x = x as i16;
            win.y = y as i16;
            win.width = width as u16;
            win.height = height as u16;
        }
    }

    Ok(())
}
