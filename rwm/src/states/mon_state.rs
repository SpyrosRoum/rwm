use {
    oorandom::Rand32,
    x11rb::{errors::ReplyOrIdError, protocol::xproto::Window, rust_connection::RustConnection},
};

use crate::{
    config::Config,
    rect::Rect,
    states::WinState,
    {windows_history::WindowsHistory, layouts::LayoutType, states::TagState},
};
use common::{Direction, LayoutSubcommand, TagId};

#[derive(Debug)]
pub(crate) struct Monitor {
    /// A randomly generated number
    pub(crate) id: u32,
    /// The window ids of the windows currently in this monitor
    pub(crate) windows: WindowsHistory,
    /// The tags that are currently visible
    // FixMe: I don't think the above comment is true, see `on_tag_cmd` for example
    // But it might not be a problem because when I am going through this I do check if it's visible
    // see `reset_focus`
    pub(crate) tags: Vec<TagState>,
    pub(crate) layout: LayoutType,

    pub(crate) rect: Rect,
}

impl Monitor {
    pub(crate) fn new(config: &Config, rng: &mut Rand32, rect: Rect) -> Self {
        let def_layout = config.layouts[0];
        // tags are 1-9 and the default is 1
        let mut tags: Vec<TagState> = (1..=9)
            .map(|i| TagState::new(i, false, def_layout).unwrap())
            .collect();
        tags[0].visible = true;

        Self {
            id: rng.rand_u32(),
            windows: WindowsHistory::new(),
            tags,
            layout: def_layout,
            rect,
        }
    }

    pub(crate) fn contains_point(&self, x: i16, y: i16) -> bool {
        self.rect.contains_point(x, y)
    }

    pub(crate) fn contains_window<W: Into<Window>>(&self, window: W) -> bool {
        self.windows.contains(window)
    }

    /// Get the next window to ge focused
    pub(crate) fn get_next_win(&self) -> Option<&WinState> {
        self.windows.find_next(&self.tags).map(|(_, new)| new)
    }

    /// Get the prev window to ge focused
    pub(crate) fn get_prev_win(&self) -> Option<&WinState> {
        self.windows.find_prev(&self.tags).map(|(_, new)| new)
    }

    pub(crate) fn shift_windows(&mut self, dir: Direction) {
        self.windows.shift(dir, self.tags.as_slice());
    }

    /// Return the window that gets forgotten and the window that should get focus
    pub(crate) fn forget(&mut self, window: Window) -> (Option<WinState>, Option<&WinState>) {
        self.windows.forget(window, self.tags.as_slice())
    }

    pub(crate) fn change_layout(&mut self, dir: &LayoutSubcommand, layouts: &[LayoutType]) {
        self.layout = match dir {
            LayoutSubcommand::Next => self.layout.next(layouts),
            LayoutSubcommand::Prev => self.layout.prev(layouts),
        };
    }

    /// Call `self.layout.update`
    pub(crate) fn update_layout(
        &mut self,
        conn: &RustConnection,
        config: &Config,
    ) -> Result<(), ReplyOrIdError> {
        let visible_tags = self
            .tags
            .iter()
            .filter(|tag_state| tag_state.visible)
            .map(|tag_state| tag_state.id)
            .collect::<Vec<_>>();

        self.layout.update(
            conn,
            &mut self.windows,
            visible_tags,
            &self.rect,
            config.border_width,
            config.gap,
        )
    }

    /// Find the first visible window in the tags and set it as focused
    pub(crate) fn reset_focus(&mut self) -> Option<usize> {
        self.windows.reset_focus(self.tags.as_slice())
    }

    pub(crate) fn switch_tag(&mut self, tag_id: TagId) {
        for tag in self.tags.iter_mut() {
            if *tag == tag_id {
                tag.visible = true;
                self.layout = tag.layout;
            } else {
                tag.visible = false;
            }
        }
    }
}

impl PartialEq<u32> for Monitor {
    fn eq(&self, other: &u32) -> bool {
        self.id.eq(other)
    }
}

impl PartialEq<Monitor> for Monitor {
    fn eq(&self, other: &Monitor) -> bool {
        self.id.eq(&other.id)
    }
}
