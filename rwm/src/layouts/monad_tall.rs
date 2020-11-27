use x11rb::{
    connection::Connection,
    errors::ReplyOrIdError,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
    rust_connection::RustConnection,
};

use crate::{focus_history::FocusHist, tag_id::TagID};

pub(crate) fn update(
    conn: &RustConnection,
    focus: &FocusHist,
    tags: Vec<TagID>,
    screen_num: usize,
    border_width: u32,
) -> Result<(), ReplyOrIdError> {
    // First window gets 60% of the screen (the left side), the rest stack on the side
    // ToDo Gaps.
    // ToDo bar space.
    let width = conn.setup().roots[screen_num].width_in_pixels as u32;
    let height = conn.setup().roots[screen_num].height_in_pixels as u32;

    let master_width = width * 60 / 100;
    let slave_width = width - master_width;

    let mut windows = focus.iter_on_tags(tags).filter(|win| !win.floating);

    let master_win = windows.next();
    if master_win.is_none() {
        // No windows, nothing to do
        return Ok(());
    }
    let master_win = master_win.unwrap();
    let stack = windows.collect::<Vec<_>>();

    let master_config = {
        if stack.is_empty() {
            ConfigureWindowAux::new()
                .width(width)
                .height(height)
                .border_width(0)
        } else {
            ConfigureWindowAux::new()
                .width(master_width - (border_width * 2))
                .height(height - (border_width * 2))
                .border_width(border_width)
        }
        .x(0)
        .y(0)
    };
    conn.configure_window(master_win.id, &master_config)?;

    if let Some(slave_height) = height.checked_div(stack.len() as u32) {
        // If we get here it means there are slave windows
        let x = master_width as i32;
        for (i, win) in stack.iter().enumerate() {
            let y = (slave_height * i as u32) as i32;
            conn.configure_window(
                win.id,
                &ConfigureWindowAux::new()
                    .x(x)
                    .y(y)
                    .width(slave_width - (border_width * 2))
                    .height(slave_height - (border_width * 2))
                    .border_width(border_width),
            )?;
        }
    }
    conn.flush()?;

    Ok(())
}
