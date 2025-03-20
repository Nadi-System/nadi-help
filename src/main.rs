use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{column, container, horizontal_space, pick_list, row, text, toggler};
use iced::{Element, Fill, Task, Theme};

use nadi::editor::{self, Editor};
use nadi::help::{self, MdHelp};
use nadi::icons;
use nadi::style;
use nadi::svg::SvgView;
use nadi::terminal::{self, Terminal};

pub fn main() -> iced::Result {
    iced::application("NADI", MainWindow::update, MainWindow::view)
        .font(icons::FONT)
        .theme(MainWindow::theme)
        .run()
}

struct MainWindow {
    light_theme: bool,
    panes: pane_grid::State<Pane>,
    panes_count: usize,
    focus: Option<pane_grid::Pane>,
    funchelp: MdHelp,
    editor: Editor,
    svg: SvgView,
    terminal: Terminal,
}

impl Default for MainWindow {
    fn default() -> Self {
        let (panes, _) = pane_grid::State::new(Pane::new(0));
        Self {
            light_theme: false,
            panes,
            panes_count: 1,
            focus: None,
            funchelp: MdHelp::default().embed(),
            editor: Editor::default().embed(),
            svg: SvgView::default().embed(),
            terminal: Terminal::default().embed(),
        }
    }
}

impl MainWindow {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeChange(t) => {
                self.light_theme = t;
            }
            Message::Terminal(m) => return self.terminal.update(m).map(Message::Terminal),
            Message::SvgView(m) => return self.svg.update(m).map(Message::SvgView),
            Message::Editor(m) => {
                return match m {
                    editor::Message::RunAllTask => {
                        let buf = self.editor.content.text();
                        Task::perform((async || buf)(), terminal::Message::RunTasks)
                            .map(Message::Terminal)
                    }
                    editor::Message::RunTask => {
                        if let Some(sel) = self.editor.content.selection() {
                            Task::perform((async || sel)(), terminal::Message::RunTasks)
                                .map(Message::Terminal)
                        } else {
                            Task::none()
                        }
                    }
                    editor::Message::SearchHelp => {
                        if let Some(sel) = self.editor.content.selection() {
                            Task::perform((async || sel)(), help::Message::SearchChange)
                                .map(Message::FuncHelp)
                        } else {
                            Task::none()
                        }
                    }
                    editor::Message::HelpTask => {
                        if let Some(func) = self.editor.function.clone() {
                            Task::perform((async || func)(), |(t, f)| help::Message::Function(t, f))
                                .map(Message::FuncHelp)
                        } else {
                            Task::none()
                        }
                    }
                    _ => self.editor.update(m).map(Message::Editor),
                };
            }
            Message::FuncHelp(m) => self.funchelp.update(m),
            Message::PaneTypeChanged(p, typ) => {
                if let Some(Pane { ty, .. }) = self.panes.get_mut(p) {
                    *ty = Some(typ);
                }
            }
            Message::PaneAction(m) => match m {
                PaneMessage::Split(axis, pane) => {
                    let result = self.panes.split(axis, pane, Pane::new(self.panes_count));

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_count += 1;
                }
                PaneMessage::Clicked(pane) => {
                    self.focus = Some(pane);
                }
                PaneMessage::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                    self.panes.resize(split, ratio);
                }
                PaneMessage::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                    self.panes.drop(pane, target);
                }
                PaneMessage::Dragged(_) => {}
                PaneMessage::TogglePin(pane) => {
                    if let Some(Pane { is_pinned, .. }) = self.panes.get_mut(pane) {
                        *is_pinned = !*is_pinned;
                    }
                }
                PaneMessage::Maximize(pane) => self.panes.maximize(pane),
                PaneMessage::Restore => {
                    self.panes.restore();
                }
                PaneMessage::Close(pane) => {
                    if let Some((_, sibling)) = self.panes.close(pane) {
                        self.focus = Some(sibling);
                    }
                }
            },
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let focus = self.focus;
        let pane_grid = PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);
            let pin_button = icons::action(
                if pane.is_pinned {
                    icons::unpin_icon()
                } else {
                    icons::pin_icon()
                },
                if pane.is_pinned { "Unpin" } else { "Pin" },
                Some(Message::PaneAction(PaneMessage::TogglePin(id))),
            );
            let title = row![pin_button, "Pane", text(pane.id.to_string()),].spacing(5);
            let title_bar = pane_grid::TitleBar::new(title)
                .controls(pane_controls(id, pane, self.panes_count, is_maximized))
                .padding(1)
                .style(if is_focused {
                    style::title_bar_focused
                } else {
                    style::title_bar_active
                });
            pane_grid::Content::new(pane_content(self, &pane.ty))
                .title_bar(title_bar)
                .style(if is_focused {
                    style::pane_focused
                } else {
                    style::pane_active
                })
        })
        .width(Fill)
        .height(Fill)
        .spacing(10)
        .on_click(|p| Message::PaneAction(PaneMessage::Clicked(p)))
        .on_drag(|p| Message::PaneAction(PaneMessage::Dragged(p)))
        .on_resize(10, |p| Message::PaneAction(PaneMessage::Resized(p)));
        let controls = row![
            horizontal_space(),
            toggler(self.light_theme).on_toggle(Message::ThemeChange),
        ]
        .spacing(20)
        .padding(10);
        column![
            controls,
            container(pane_grid).width(Fill).height(Fill).padding(10),
        ]
        .into()
    }

    fn theme(&self) -> Theme {
        if self.light_theme {
            Theme::Light
        } else {
            Theme::Dark
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    PaneAction(PaneMessage),
    PaneTypeChanged(pane_grid::Pane, PaneType),
    FuncHelp(nadi::help::Message),
    Editor(nadi::editor::Message),
    SvgView(nadi::svg::Message),
    Terminal(nadi::terminal::Message),
    ThemeChange(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneType {
    FunctionHelp,
    TextEditor,
    SvgView,
    Terminal,
}

impl std::fmt::Display for PaneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::FunctionHelp => "Function Help",
                Self::TextEditor => "Text Editor",
                Self::SvgView => "Svg Viewer",
                Self::Terminal => "Terminal",
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
enum PaneMessage {
    Split(pane_grid::Axis, pane_grid::Pane),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Maximize(pane_grid::Pane),
    Restore,
    Close(pane_grid::Pane),
}

struct Pane {
    id: usize,
    pub is_pinned: bool,
    pub ty: Option<PaneType>,
}

impl Pane {
    fn new(id: usize) -> Self {
        Self {
            id,
            is_pinned: false,
            ty: None,
        }
    }
}
fn pane_controls<'a>(
    id: pane_grid::Pane,
    pane: &Pane,
    panes_count: usize,
    is_maximized: bool,
) -> Element<'a, Message> {
    row![
        pick_list(
            [
                PaneType::FunctionHelp,
                PaneType::TextEditor,
                PaneType::SvgView,
                PaneType::Terminal
            ],
            pane.ty,
            move |t| Message::PaneTypeChanged(id, t),
        ),
        icons::action(
            icons::hsplit_icon(),
            "Horizontal Split",
            Some(Message::PaneAction(PaneMessage::Split(
                pane_grid::Axis::Horizontal,
                id
            ))),
        ),
        icons::action(
            icons::vsplit_icon(),
            "Vertical Split",
            Some(Message::PaneAction(PaneMessage::Split(
                pane_grid::Axis::Vertical,
                id
            ))),
        ),
        if is_maximized {
            icons::action(
                icons::resize_small_icon(),
                "Restore",
                Some(Message::PaneAction(PaneMessage::Restore)),
            )
        } else {
            icons::action(
                icons::resize_full_icon(),
                "Maximize",
                (panes_count > 1).then_some(Message::PaneAction(PaneMessage::Maximize(id))),
            )
        },
        icons::danger_action(
            icons::cancel_icon(),
            "Close",
            (panes_count > 1).then_some(Message::PaneAction(PaneMessage::Close(id))),
        ),
    ]
    .spacing(5)
    .into()
}
fn pane_content<'a>(win: &'a MainWindow, ty: &'a Option<PaneType>) -> Element<'a, Message> {
    match ty {
        None => text("Select a Pane Type").into(),
        Some(PaneType::FunctionHelp) => win.funchelp.view().map(Message::FuncHelp),
        Some(PaneType::TextEditor) => win.editor.view().map(Message::Editor),
        Some(PaneType::SvgView) => win.svg.view().map(Message::SvgView),
        Some(PaneType::Terminal) => win.terminal.view().map(Message::Terminal),
    }
}
