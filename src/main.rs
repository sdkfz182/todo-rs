use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, Frame,
    layout::{Constraint, Alignment, Layout, Direction, Rect},
    widgets::{Block, List, ListItem, ListState, Borders, Paragraph},
    text::{Line, Text},
    style::{Style, Color},
};
use color_eyre::Result;

enum TodoModes {
    PageSelect,
    Normal,
    Insert,
    Edit,
    Popup,
}

enum ActiveInput {
    None,
    AddPage,
    AddGroup,
    AddTodo,
}

struct ApplicationState {
    mode: TodoModes,
    input_mode: ActiveInput,
    title: String,
    page_list: Vec<TodoPage>,
    id_counter: u32,
    should_quit: bool,

    selected_page: Option <usize>,
    selected_group: Option<usize>,
    selected_todo: Option<usize>, 

    buffer_string: String,

    // UI
    page_list_state: ListState,
}

struct TodoItem {
    id: u32,
    title: String,
    description: String,
}

struct TodoGroup {
    title: String,
    item_list: Vec<TodoItem>,
}

struct TodoPage {
    title: String,
    group_list: Vec<TodoGroup>,
}

impl ApplicationState {
    fn new(title_: String) -> Self { // Initialization
        Self {
            mode: TodoModes::PageSelect,
            input_mode: ActiveInput::None,
            title: title_,
            page_list: Vec::new(),
            id_counter: 1,
            should_quit: false,

            selected_page: None,
            selected_group: None,
            selected_todo: None,

            buffer_string: String::new(),

            page_list_state: ListState::default(),
        }
    }

    fn add_page(&mut self, _title: String) {
        self.page_list.push(TodoPage::new(_title))
    }
}

impl TodoPage {
    fn new(_title: String) -> Self {
        Self {
            title: _title,
            group_list: Vec::new(),
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app_state: ApplicationState = ApplicationState::new("Balls-on-fire Todo(RUST)".to_string());

    
    // app_state.page_list.push(TodoPage::new("Page1".to_string()));
    // app_state.page_list.push(TodoPage::new("Page2".to_string()));
    // app_state.page_list.push(TodoPage::new("Page3".to_string()));

    // app_state.page_list_state.select(Some(0));
    // app_state.selected_page = Some(0); 
    

    loop {
        // LOGIC 

        // RENDER 
        let _ = terminal.draw(|f| render(f, &mut app_state));

        // INPUT
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_input(key, &mut app_state);
            }
        }

        if app_state.should_quit {
            break;
        }
    }
    Ok(())
}

// --------------------------------- RENDER ---------------------------------

fn render(frame: &mut Frame, app_state: &mut ApplicationState) { // Handles logic and Routes
    match app_state.mode {
        TodoModes::PageSelect => render_page_select(frame, app_state),
        TodoModes::Normal => render_page(frame, app_state),
        TodoModes::Insert => {
            match app_state.input_mode {
                ActiveInput::AddPage => {
                    render_popup_input_field(frame, app_state);
                }
                _ => (),
            }
        }
        _ => (),
    }
}

fn render_page(frame: &mut Frame, app_state: &mut ApplicationState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(5)])
        .split(frame.area());

    let header = chunks[0]; let body = chunks[1]; let footer = chunks[2];

 
    let title = format!("{} {}", "Page:", app_state.page_list[app_state.selected_page.unwrap()].title);

    let header_block = Block::default().borders(Borders::ALL).title("Header");
    let main_block = Block::default().borders(Borders::ALL).title(title);
    let footer_block = Block::default().borders(Borders::ALL).title("Controls:");

    frame.render_widget(header_block, header);
    frame.render_widget(main_block, body);
    frame.render_widget(footer_block, footer);
}

fn render_page_select(frame: &mut Frame, app_state: &mut ApplicationState) {
    let area = frame.area().centered(
        Constraint::Length(50),
        Constraint::Length(25)
    );

    let menu = Block::default()
        .borders(Borders::ALL)
        .title(app_state.title.clone())
        .title_alignment(Alignment::Center);

    if app_state.page_list.len() > 0 {

        let list = List::new(app_state.page_list
            .iter().map(|i| ListItem::new(Line::from(i.title.as_str()).alignment(Alignment::Center))))
            .block(menu)
            .highlight_symbol(">>")
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow),
            );

        frame.render_stateful_widget(list, area, &mut app_state.page_list_state);
    }
    else {
        let text = Paragraph::new(Text::from(vec![
                Line::from("No pages found..."),
                Line::from("Press \'a\' to create one."),
        ])).block(menu);

        frame.render_widget(text, area);
    }
}

fn render_popup_input_field(frame: &mut Frame, app_state: &mut ApplicationState) {
    let width = 50;
    let height = 3;
    
    let rect = Rect::new((frame.area().width - width) / 2, (frame.area().height - height) / 2, width, height);
    let textbox = Block::default().borders(Borders::ALL).title("Enter new Page name:");
    let buffer = Paragraph::new(app_state.buffer_string.clone()).block(textbox);
    
    frame.render_widget(buffer, rect); 
}

// ----------------------------- END OF RENDER -----------------------------

// --------------------------------- INPUT ---------------------------------

fn handle_input(key: KeyEvent, app_state: & mut ApplicationState) { // Routes input 
    match app_state.mode {
        TodoModes::Normal => handle_normal_input(key, app_state),
        TodoModes::PageSelect => handle_page_select_input(key, app_state),
        TodoModes::Insert => handle_insert(key, app_state),
        _ => (),
    }
}

fn handle_page_select_input(key: KeyEvent, app_state: & mut ApplicationState) {
    match key.code {
        KeyCode::Esc => app_state.should_quit = true,
        KeyCode::Char('k') | KeyCode::Up => { 
            if app_state.page_list.len() > 0 {
                let up = match app_state.page_list_state.selected() {
                    Some(up) => {
                        if up == 0 {app_state.page_list.len() - 1} else {up - 1}
                    },
                    None => 0,
                };
                app_state.page_list_state.select(Some(up));
                app_state.selected_page = Some(up);
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if app_state.page_list.len() > 0 {
                let down = match app_state.page_list_state.selected() {
                    Some(down) => { 
                        if down == app_state.page_list.len() - 1 {0} else {down + 1}
                    },
                    None => 0,
                };
                app_state.page_list_state.select(Some(down));
                app_state.selected_page = Some(down);
            }
        }
        KeyCode::Enter => {
            if !app_state.page_list_state.selected().is_none() && app_state.page_list.len() > 0 {
                app_state.mode = TodoModes::Normal;
            }
        }
        KeyCode::Char('a') => { // Add page
            app_state.mode = TodoModes::Insert;
            app_state.input_mode = ActiveInput::AddPage;
        }
        _ => (),
    }
}
fn handle_normal_input(key: KeyEvent, app_state: &mut ApplicationState) {
    match key.code {
        KeyCode::Esc => app_state.mode = TodoModes::PageSelect,
        _ => (),
    }
}

fn handle_insert(key: KeyEvent, app_state: &mut ApplicationState) {
    match key.code {
        KeyCode::Char(c) => {
            app_state.buffer_string.push(c);
        }
        KeyCode::Backspace => {
            app_state.buffer_string.pop();
        }
        KeyCode::Esc => { // Cancel
            app_state.buffer_string.clear();
            match app_state.input_mode {
                ActiveInput::AddPage => {
                    app_state.mode = TodoModes::PageSelect;
                    app_state.input_mode = ActiveInput::None;
                }
                _ => (),
            }
        }
        KeyCode::Enter => {
            app_state.buffer_string.trim();
            match app_state.input_mode {
                ActiveInput::AddPage => {
                    if !app_state.buffer_string.is_empty() {
                        app_state.add_page(app_state.buffer_string.clone());
                    }

                    app_state.mode = TodoModes::PageSelect;
                    app_state.input_mode = ActiveInput::None;
                }
                _ => (),
            }

            app_state.buffer_string.clear();
        }
        _ => (),
    }
}
