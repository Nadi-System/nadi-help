use crate::icons;
use iced::widget::{
    button, center, column, horizontal_space, markdown, row, scrollable, text, text_input, toggler,
};
use iced::{Color, Element, Length, Theme, widget::Column};
use nadi_core::functions::{FuncArg, NadiFunctions};

pub static MAIN_HELP: &str = include_str!("../markdown/main.md");
static FUNC_WIDTH: f32 = 300.0;

#[derive(Clone, Debug)]
pub enum FuncType {
    Node,
    Network,
    Env,
}

impl std::fmt::Display for FuncType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Node => "node",
                Self::Network => "network",
                Self::Env => "env",
            }
        )
    }
}

pub struct MdHelp {
    pub light_theme: bool,
    functions: NadiFunctions,
    state: Option<FuncType>,
    search: String,
    markdown: Vec<markdown::Item>,
    collapsed: bool,
    embedded: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    LinkClicked(markdown::Url),
    Home,
    Github,
    Book,
    ToggleCollapsed,
    Function(FuncType, String),
    FuncTypeChange(Option<FuncType>),
    ThemeChange(bool),
    SearchChange(String),
}

impl Default for MdHelp {
    fn default() -> Self {
        Self {
            light_theme: false,
            functions: NadiFunctions::new(),
            state: None,
            search: String::new(),
            markdown: markdown::parse(MAIN_HELP).collect(),
            collapsed: false,
            embedded: false,
        }
    }
}

// Macro instead of function as func are different types, but the
// traits have same functions
macro_rules! help {
    ($ty:expr, $name:expr, $func:expr) => {
        help_to_markdown(
            $ty,
            &$name,
            &$func.args(),
            &$func.short_help(),
            &$func.help(),
            &$func.code(),
        )
    };
}

impl MdHelp {
    pub fn embed(mut self) -> Self {
        self.embedded = true;
        self.collapsed = true;
        self
    }
    pub fn view(&self) -> Element<'_, Message> {
        let mut controls = row![
            button("Home").on_press(Message::Home),
            button("Book").on_press(Message::Book),
            button("GitHub").on_press(Message::Github),
            horizontal_space()
        ]
        .spacing(20)
        .padding(10);
        if !self.embedded {
            controls = controls.push(toggler(self.light_theme).on_toggle(Message::ThemeChange));
        }
        let md = markdown::view(
            &self.markdown,
            markdown::Settings::default(),
            md_style(self.light_theme),
        )
        .map(Message::LinkClicked);

        let toggle_view = button(center(if self.collapsed {
            icons::right_icon()
        } else {
            icons::left_icon()
        }))
        .on_press(Message::ToggleCollapsed)
        .height(Length::Fill)
        .style(button::secondary)
        .width(25);

        let functions: Element<_> = if self.collapsed {
            toggle_view.into()
        } else {
            let ftypes = row![
                button("All")
                    .on_press(Message::FuncTypeChange(None))
                    .style(match self.state {
                        None => button::success,
                        _ => button::primary,
                    }),
                button("Env")
                    .on_press(Message::FuncTypeChange(Some(FuncType::Env)))
                    .style(match self.state {
                        Some(FuncType::Env) => button::success,
                        _ => button::primary,
                    }),
                button("Node")
                    .on_press(Message::FuncTypeChange(Some(FuncType::Node)))
                    .style(match self.state {
                        Some(FuncType::Node) => button::success,
                        _ => button::primary,
                    }),
                button("Network")
                    .on_press(Message::FuncTypeChange(Some(FuncType::Network)))
                    .style(match self.state {
                        Some(FuncType::Network) => button::success,
                        _ => button::primary,
                    }),
            ]
            .spacing(20)
            .padding(10);
            let funcs: Vec<Element<_>> = list_functions(&self.functions, &self.state, &self.search)
                .into_iter()
                .enumerate()
                .map(|(i, n)| {
                    button(text(format!("{}  {}", n.0, n.1)))
                        .on_press(Message::Function(n.0.clone(), n.1.to_string()))
                        .width(Length::Fill)
                        .style(if (i % 2) == 0 {
                            secondary_even
                        } else {
                            secondary_odd
                        })
                        .into()
                })
                .collect();

            let list = Column::from_vec(funcs).width(FUNC_WIDTH);
            let search = text_input("Search", &self.search)
                .on_input(Message::SearchChange)
                .padding(10)
                .width(FUNC_WIDTH);
            row![
                column![ftypes, search, scrollable(list)].spacing(10),
                toggle_view
            ]
            .into()
        };

        let main = row![functions, scrollable(md)].spacing(10).padding(10);
        column![controls, main].spacing(10).into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::LinkClicked(url) => {
                match url.scheme() {
                    // this way we can make our own schema for the
                    // links to nadi functions
                    "nadi" => todo!(),
                    _ => {
                        _ = webbrowser::open(url.as_ref());
                    }
                }
            }
            Message::Home => {
                self.markdown = markdown::parse(MAIN_HELP).collect();
            }
            Message::Book => {
                _ = webbrowser::open("https://nadi-system.github.io/");
            }
            Message::Github => {
                _ = webbrowser::open("https://github.com/Nadi-System/");
            }
            Message::ToggleCollapsed => {
                self.collapsed = !self.collapsed;
            }
            Message::SearchChange(s) => {
                self.search = s;
            }
            Message::Function(FuncType::Node, func) => {
                if let Some(f) = self.functions.node(&func) {
                    self.markdown = help!("node", func, f);
                }
            }
            Message::Function(FuncType::Network, func) => {
                if let Some(f) = self.functions.network(&func) {
                    self.markdown = help!("network", func, f);
                }
            }
            Message::Function(FuncType::Env, func) => {
                if let Some(f) = self.functions.env(&func) {
                    self.markdown = help!("env", func, f);
                }
            }
            Message::FuncTypeChange(state) => {
                self.state = state;
            }
            Message::ThemeChange(t) => {
                self.light_theme = t;
            }
        }
    }

    pub fn theme(&self) -> Theme {
        if self.light_theme {
            Theme::Light
        } else {
            Theme::Dark
        }
    }
}

pub fn list_functions<'a>(
    functions: &'a NadiFunctions,
    state: &Option<FuncType>,
    search: &str,
) -> Vec<(FuncType, &'a str)> {
    let searches: Vec<&str> = search.trim().split(' ').collect();
    let mut func: Vec<(FuncType, &str)> = match state {
        Some(FuncType::Node) => functions
            .node_functions()
            .iter()
            .filter(|n| searches.iter().all(|&s| n.0.contains(s) || s == "node"))
            .map(|n| (FuncType::Node, n.0.as_str()))
            .collect(),
        Some(FuncType::Network) => functions
            .network_functions()
            .iter()
            .filter(|n| searches.iter().all(|&s| n.0.contains(s) || s == "network"))
            .map(|n| (FuncType::Network, n.0.as_str()))
            .collect(),
        Some(FuncType::Env) => functions
            .env_functions()
            .iter()
            .filter(|n| searches.iter().all(|&s| n.0.contains(s) || s == "env"))
            .map(|n| (FuncType::Env, n.0.as_str()))
            .collect(),
        None => {
            return vec![
                list_functions(functions, &Some(FuncType::Env), search),
                list_functions(functions, &Some(FuncType::Node), search),
                list_functions(functions, &Some(FuncType::Network), search),
            ]
            .into_iter()
            .flatten()
            .collect();
        }
    };
    func.sort_by(|a, b| a.1.cmp(b.1));
    func
}

pub fn help_to_markdown(
    ty: &str,
    name: &str,
    args: &[FuncArg],
    short: &str,
    long: &str,
    code: &str,
) -> Vec<markdown::Item> {
    let mut items = vec![];
    let sig = args
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    items.push(format!(
        "# {ty} <span color=\"blue\">{name}</span>\n```python\n{ty} {name}({sig})\n```\n\n{short}"
    ));
    items.push("## Arguments".to_string());
    args.iter()
        .for_each(|f| items.push(format!("- `{}` => {}", f.to_string(), f.help)));
    items.push("\n".to_string());
    items.push(long[short.len()..].trim().to_string());
    items.push(format!("# Code\n```rust\n{code}\n```\n"));
    markdown::parse(&items.join("\n")).collect()
}

pub fn md_style(light: bool) -> markdown::Style {
    let pc = if light { 0.0 } else { 1.0 };
    let inline_code_highlight = markdown::Highlight {
        background: iced::Background::Color(Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 0.5,
        }),
        border: iced::Border {
            color: Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 0.0,
            },
            width: 1.0,
            radius: iced::border::Radius::from(5.0),
        },
    };
    let inline_code_padding = iced::Padding::from(2.0);
    let inline_code_color = Color {
        r: pc,
        g: pc,
        b: pc,
        a: 1.0,
    };
    let link_color = Color {
        r: 0.5,
        g: 0.5,
        b: 1.0,
        a: 1.0,
    };

    markdown::Style {
        inline_code_highlight,
        inline_code_padding,
        inline_code_color,
        link_color,
    }
}

pub fn secondary_odd(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let pair = palette.secondary.base;
    let base = button::Style {
        background: Some(iced::Background::Color(pair.color)),
        text_color: pair.text,
        border: iced::border::rounded(0),
        ..button::Style::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => base,
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(palette.secondary.strong.color)),
            ..base
        },
        button::Status::Disabled => base,
    }
}

pub fn secondary_even(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let pair = palette.secondary.base;
    let base = button::Style {
        background: Some(iced::Background::Color(pair.color.scale_alpha(0.5))),
        text_color: pair.text,
        border: iced::border::rounded(0),
        ..button::Style::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => base,
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(palette.secondary.strong.color)),
            ..base
        },
        button::Status::Disabled => base,
    }
}
