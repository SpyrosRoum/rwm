use std::{env, path::{Path, PathBuf}};

use {
    anyhow::{anyhow, Context, Result},
    flexi_logger::{FlexiLoggerError, LoggerHandle},
    oorandom::Rand32,
    x11rb::{
        connection::Connection as _,
        errors::ReplyOrIdError,
        protocol::{randr, xproto::*},
        rust_connection::RustConnection,
    },
};

use crate::{
    config::Config,
    mod_mask::XModMask,
    rect::Rect,
    states::{Monitor, TagState, WinState},
};

pub(crate) fn clean_mask(mask: u16) -> u16 {
    // ToDo: I think num lock is not always Mod2, find a way to get that dynamically
    let num_lock = u16::from(KeyButMask::MOD2 | KeyButMask::LOCK);
    (mask & !num_lock)
        & u16::from(
            KeyButMask::SHIFT
                | KeyButMask::CONTROL
                | KeyButMask::MOD1
                | KeyButMask::MOD2
                | KeyButMask::MOD3
                | KeyButMask::MOD4
                | KeyButMask::MOD5,
        )
}

pub(crate) fn clean_up() -> Result<()> {
    // Socket is being deleted on Drop implementation for WmState so there is nothing to do here
    // Stays for potentially future use
    Ok(())
}

/// Get all the visible tags only
pub(crate) fn visible(tags: &[TagState]) -> Vec<TagState> {
    tags.iter().filter(|tag| tag.visible).copied().collect()
}

/// Returns true if the window is in a tag that's visible
pub(crate) fn is_visible(win: &WinState, tags: &[TagState]) -> bool {
    tags.iter()
        .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
}

/// A help function to get the WM_TRANSIENT_FOR hint for the given window
pub(crate) fn get_transient_for(conn: &RustConnection, win_id: Window) -> Result<Option<Window>> {
    Ok(conn
        .get_property(
            false,
            win_id,
            AtomEnum::WM_TRANSIENT_FOR,
            AtomEnum::WINDOW,
            0,
            1,
        )?
        .reply()?
        .value32()
        .ok_or_else(|| anyhow!("Wrong format"))?
        .next())
}

pub(crate) fn grab_buttons(
    conn: &RustConnection,
    window: Window,
    mod_key: XModMask,
    focus: bool,
) -> Result<(), ReplyOrIdError> {
    conn.ungrab_button(ButtonIndex::ANY, window, ModMask::ANY)?;

    // This ugly line is needed because grab_button expects something that implements Into<u16>
    // but EventMask is u32
    let event_mask =
        u32::from(EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION);
    let mod_key = u16::from(mod_key);
    if focus {
        // We need to grab for our modifier key, for our mod key + numlock, mod + lock, and mod + numlock + lock
        for mask in std::array::IntoIter::new([
            0_u16,
            ModMask::LOCK.into(),
            ModMask::M2.into(), // ToDo `M2` might not always be numlock (see utils.rs as well)
            u16::from(ModMask::LOCK) | u16::from(ModMask::M2),
        ]) {
            conn.grab_button(
                false,
                window,
                event_mask as u16,
                GrabMode::ASYNC,
                GrabMode::SYNC,
                x11rb::NONE,
                x11rb::NONE,
                ButtonIndex::ANY,
                mod_key | mask,
            )?;
        }
    } else {
        // Grab everything
        conn.grab_button(
            false,
            window,
            event_mask as u16,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
            x11rb::NONE,
            x11rb::NONE,
            ButtonIndex::ANY,
            ModMask::ANY,
        )?;
    }

    Ok(())
}

pub(crate) fn get_monitors(
    conn: &RustConnection,
    config: &Config,
    screen_num: usize,
    mut rng: &mut Rand32,
) -> Result<Vec<Monitor>> {
    let screen = &conn.setup().roots[screen_num];
    // Todo: Should check randr version? https://github.com/linebender/druid/pull/1804/files#diff-887fa9b9ca679a0017ce71c83d9be00af81b30d94c20f2430ffac6caa2fc3531R74

    Ok(randr::get_monitors(conn, screen.root, true)
        .context("Failed to get monitors from randr")?
        .reply()
        .context("Failed to get monitors from randr")?
        .monitors
        .iter()
        .map(|info| {
            Monitor::new(
                config,
                &mut rng,
                Rect::new(info.x, info.y, info.width, info.height),
            )
        })
        .collect())
}

pub(crate) fn expand_tilde(path: &Path) -> PathBuf {
    if path.starts_with("~") {
        let mut base = dirs::home_dir().expect("Failed to get home dir");
        let dir = path.to_string_lossy();
        base.push(
            dir.strip_prefix("~/")
                .unwrap_or_else(|| dir.strip_prefix('~').unwrap()),
        );
        base
    } else {
        path.to_owned()
    }
}

pub(crate) fn init_logging(log_dir: &Option<PathBuf>) -> Result<LoggerHandle, FlexiLoggerError> {
    use flexi_logger::*;

    /// Same as `flexi_logger::colored_opt_format` just with different timestamp format
    fn custom_colored_opt_format(
        w: &mut dyn std::io::Write,
        now: &mut DeferredNow,
        record: &Record,
    ) -> Result<(), std::io::Error> {
        let level = record.level();

        write!(
            w,
            "[{}] {} [{}:{}]: {}",
            style(level, now.now().format("%Y-%m-%d %H:%M:%S")),
            style(level, level),
            record.file().unwrap_or("<unnamed>"),
            record.line().unwrap_or(0),
            style(level, &record.args())
        )
    }

    // If the user provided a path that contains `~` we need to expand it to the home dir
    let log_dir = if let Some(dir) = log_dir {
        expand_tilde(dir)
    } else {
        let mut dir = dirs::config_dir().unwrap();
        dir.push("rwm/logs");
        dir
    };

    Logger::try_with_str(env::var("RWM_LOG").as_deref().unwrap_or("info"))?
        .format(custom_colored_opt_format)
        .log_to_file(FileSpec::default().directory(log_dir))
        .write_mode(WriteMode::Async)
        .rotate(
            Criterion::AgeOrSize(Age::Day, 100_000_000), // A day or 100mb
            Naming::Numbers,
            Cleanup::KeepLogFiles(3),
        )
        .duplicate_to_stderr(Duplicate::All)
        .start()
}
