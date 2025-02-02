use fltk::{enums::*, prelude::*, utils::oncelock::Lazy, *};
use std::path::PathBuf;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
static STATE: Lazy<app::GlobalState<State>> = Lazy::new(|| app::GlobalState::<State>::get());

pub struct State {
    pub saved: bool,
    pub buf: text::TextBuffer,
    pub current_file: PathBuf,
}

impl State {
    fn new(buf: text::TextBuffer) -> Self {
        State {
            saved: false,
            buf,
            current_file: PathBuf::new(),
        }
    }
}

fn init_menu(m: &mut menu::SysMenuBar) {
    m.add(
        "&File/New...\t",
        Shortcut::Ctrl | 'n',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/Open...\t",
        Shortcut::Ctrl | 'o',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/Save\t",
        Shortcut::Ctrl | 's',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&File/Save as...\t",
        Shortcut::Ctrl | 'w',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    let idx = m.add(
        "&File/Quit\t",
        Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.at(idx).unwrap().set_label_color(Color::Red);
    m.add(
        "&Edit/Cut\t",
        Shortcut::Ctrl | 'x',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Edit/Copy\t",
        Shortcut::Ctrl | 'c',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Edit/Paste\t",
        Shortcut::Ctrl | 'v',
        menu::MenuFlag::Normal,
        menu_cb,
    );
    m.add(
        "&Help/About\t",
        Shortcut::None,
        menu::MenuFlag::Normal,
        menu_cb,
    );
}

fn nfc_get_file(mode: dialog::NativeFileChooserType) -> PathBuf {
    let mut nfc = dialog::NativeFileChooser::new(mode);
    nfc.show();
    nfc.filename()
}

fn win_cb(w: &mut window::Window) {
    if app::event() == Event::Close {
        w.hide();
    }
}

fn editor_cb(_e: &mut text::TextEditor) {
    STATE.with(|s| s.saved = false);
}

fn handle_drag_drop(editor: &mut text::TextEditor) {
    editor.handle({
        let mut dnd = false;
        let mut released = false;
        let buf = editor.buffer().unwrap().clone();
        move |_, ev| match ev {
            Event::DndEnter => {
                dnd = true;
                true
            }
            Event::DndDrag => true,
            Event::DndRelease => {
                released = true;
                true
            }
            Event::Paste => {
                if dnd && released {
                    let path = app::event_text();
                    let path = path.trim();
                    let path = path.replace("file://", "");
                    let path = std::path::PathBuf::from(&path);
                    if path.exists() {
                        // we use a timeout to avoid pasting the path into the buffer
                        app::add_timeout3(0.0, {
                            let mut buf = buf.clone();
                            move |_| match buf.load_file(&path) {
                                Ok(_) => (),
                                Err(e) => dialog::alert_default(&format!(
                                    "An issue occured while loading the file: {}",
                                    e
                                )),
                            }
                        });
                    }
                    dnd = false;
                    released = false;
                    true
                } else {
                    false
                }
            }
            Event::DndLeave => {
                dnd = false;
                released = false;
                true
            }
            _ => false,
        }
    });
}

fn menu_cb(m: &mut impl MenuExt) {
    if let Ok(mpath) = m.item_pathname(None) {
        let ed: text::TextEditor = app::widget_from_id("ed").unwrap();
        match mpath.as_str() {
            "&File/New\t" => {
                STATE.with(|s| {
                    if !s.buf.text().is_empty() {
                        let c = dialog::choice2_default(
                            "Are you sure you want to clear the buffer?",
                            "Yes",
                            "No",
                            "",
                        );
                        if c == Some(0) {
                            s.buf.set_text("");
                            s.saved = false;
                        }
                    }
                });
            }
            "&File/Open...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseFile);
                if let Ok(text) = std::fs::read_to_string(&c) {
                    STATE.with(move |s| {
                        s.buf.set_text(&text);
                        s.saved = false;
                        s.current_file = c.clone();
                    });
                }
            }
            "&File/Save\t" => {
                STATE.with(|s| {
                    if !s.saved && s.current_file.exists() {
                        std::fs::write(&s.current_file, &s.buf.text()).ok();
                    }
                });
            }
            "&File/Save as...\t" => {
                let c = nfc_get_file(dialog::NativeFileChooserType::BrowseSaveFile);
                STATE.with(move |s| {
                    std::fs::write(&c, &s.buf.text()).ok();
                    s.saved = true;
                    s.current_file = c.clone();
                });
            }
            "&File/Quit\t" => {
                STATE.with(|s| {
                    if s.saved {
                        app::quit();
                    } else {
                        let c = dialog::choice2_default(
                            "Are you sure you want to exit without saving?",
                            "Yes",
                            "No",
                            "",
                        );
                        if c == Some(0) {
                            app::quit();
                        }
                    }
                });
            }
            "&Edit/Cut\t" => ed.cut(),
            "&Edit/Copy\t" => ed.copy(),
            "&Edit/Paste\t" => ed.paste(),
            "&Help/About\t" => {
                dialog::message_default("A minimal text editor written using fltk-rs!")
            }
            _ => unreachable!(),
        }
    }
}

fn main() {
    let a = app::App::default().with_scheme(app::Scheme::Oxy);
    app::get_system_colors();

    let mut buf = text::TextBuffer::default();
    buf.set_tab_distance(4);

    let state = State::new(buf.clone());
    app::GlobalState::new(state);

    let mut w = window::Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("Ted");
    w.set_xclass("ted");
    {
        let mut col = group::Flex::default_fill().column();
        col.set_pad(0);
        let mut m = menu::SysMenuBar::default();
        init_menu(&mut m);
        let mut ed = text::TextEditor::default().with_id("ed");
        ed.set_buffer(buf);
        ed.set_linenumber_width(40);
        ed.set_text_font(Font::Courier);
        ed.set_trigger(CallbackTrigger::Changed);
        ed.set_callback(editor_cb);
        handle_drag_drop(&mut ed);
        w.resizable(&col);
        col.set_size(&m, 30);
        col.end();
    }
    w.end();
    w.show();
    w.set_callback(win_cb);
    a.run().unwrap();
}
