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
    AddSelect,
}

enum ActiveInput {
    None,
    AddPage,
    AddGroup,
    AddTodo,
}

enum AlertMode {
    None,
    Error,
    Warning,
    Message,
}

enum TodoState {
    Done,
    Failed,
    Late,
    Idle,
}

struct ApplicationState {
    mode: TodoModes,
    input_mode: ActiveInput,
    alert_mode: AlertMode,
    title: String,
    page_list: Vec<TodoPage>,
    id_counter: u32,
    should_quit: bool,
    has_popup: bool,

    selected_page: Option <usize>,
    selected_group: Option<usize>,
    selected_todo: Option<usize>, 

    buffer_string: String,
    alert_string_buffer: String,

    // UI
    list_length: usize,
    page_list_state: ListState,
    item_list_state: ListState,
}

struct TodoItem {
    id: u32,
    title: String,
    description: String,
    state: TodoState,
}

struct TodoGroup {
    show_items: bool,
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
            alert_mode: AlertMode::None,
            title: title_,
            page_list: Vec::new(),
            id_counter: 1,
            should_quit: false,
            has_popup: false,

            selected_page: None,
            selected_group: None,
            selected_todo: None,

            buffer_string: String::new(),
            alert_string_buffer: String::new(),

            list_length: 0,
            page_list_state: ListState::default(),
            item_list_state: ListState::default(),
        } 
    }

    fn add_page(&mut self, _title: String) {
        self.page_list.push(TodoPage::new(_title))
    }

    fn selected_page(&self) -> Option<&TodoPage> {
        let p = self.selected_page?;
        Some(self.page_list.get(p)?)
    }

    fn selected_group(&self) -> Option<&TodoGroup> {
        let p = self.selected_page?;
        let g = self.selected_group?;

        Some(self.page_list.get(p)?.group_list.get(g)?)
    }

    fn selected_item(&self) -> Option<&TodoItem> {
        let p = self.selected_page?;
        let g = self.selected_group?;
        let t = self.selected_todo?;

        Some(self.page_list.get(p)?.group_list.get(g)?.item_list.get(t)?)
    }

    fn selected_mut_page(&mut self) -> Option<&mut TodoPage> {
        let p = self.selected_page?;
        Some(self.page_list.get_mut(p)?)
    }

    fn selected_mut_group(&mut self) -> Option<&mut TodoGroup> {
        let p = self.selected_page?;
        let g = self.selected_group?;

        Some(self.page_list.get_mut(p)?.group_list.get_mut(g)?)
    }

    fn selected_mut_item(&mut self) -> Option<&mut TodoItem> {
        let p = self.selected_page?;
        let g = self.selected_group?;
        let t = self.selected_todo?;

        Some(self.page_list.get_mut(p)?.group_list.get_mut(g)?.item_list.get_mut(t)?)
    }

    fn selected_item_up(&mut self) {
        if self.list_length > 0 {
            let i = match self.item_list_state.selected() {
                Some(i) => if i == 0 { self.list_length - 1 } else { i - 1 }
                None => 0,
            };
            self.item_list_state.select(Some(i));
        }
    }

    fn selected_item_down(&mut self) {
        if self.list_length > 0 {
            let i = match self.item_list_state.selected() {
                Some(i) => if i == self.list_length - 1 { 0 } else { i + 1 }
                None => 0,
            };
            self.item_list_state.select(Some(i));
        }
    }

    fn alert_box(&mut self, a_mode: AlertMode, message_str: String) {
        self.mode = TodoModes::Popup;
        self.has_popup = true;
        self.alert_mode = a_mode;
        self.alert_string_buffer = message_str;
    }
}

impl TodoPage {
    fn new(_title: String) -> Self {
        Self {
            title: _title,
            group_list: Vec::new(),
        }
    }

    fn add_group(&mut self, _title: String) {
        self.group_list.push(TodoGroup::new(_title));
    }
}

impl TodoGroup {
    fn new(_title: String) -> Self {
        Self {
            show_items: true,
            title: _title,
            item_list: Vec::new(),
        }
    }

    fn add_todo(&mut self, _title: String) {
        self.item_list.push(TodoItem::new(_title));
    } 

    fn toggle_show_item(&mut self) {
        if self.show_items {
            self.show_items = false;
        }
        else {
            self.show_items = true;
        }
    }

    fn clear_list(&mut self) {
        self.item_list.clear() 
    }
    
    fn rename(&mut self, _title: String) {
        self.title = _title;
    }

    fn move_todo_up(&mut self) {

    }
    
    fn move_todo_down(&mut self) {

    }
}

impl TodoItem {
    fn new(_title: String) -> Self {
        Self {
            id: 0,
            title: _title,
            description: String::new(),
            state: TodoState::Idle,
        }    
    }

    fn rename(&mut self, _title: String) {
        self.title = _title;
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
                ActiveInput::AddPage => render_popup_input_field(frame, app_state, "Create new Page:"),
                ActiveInput::AddGroup =>  {
                    render_page(frame, app_state);
                    render_popup_input_field(frame, app_state, "Create new Group:");
                },
                ActiveInput::AddTodo => {
                    render_page(frame, app_state);
                    render_popup_input_field(frame, app_state, "Create new Todo:");
                },
                _ => (),
            }
        }
        TodoModes::AddSelect => {
            render_page(frame, app_state);
            render_add_select(frame, app_state);
        }
        _ => (),
    }

    if app_state.has_popup {
        render_alert_box(frame, &app_state.alert_mode, &app_state.alert_string_buffer.as_str());
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
    frame.render_widget(main_block.clone(), body);
    frame.render_widget(footer_block, footer);

    let inner_area = main_block.inner(body);
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ]).split(inner_area);

    let inner1 = main_chunks[0]; let inner2 = main_chunks[1];

    let mut list_state = app_state.item_list_state.clone();
    let content_block = block_content_list(app_state);

    frame.render_stateful_widget(content_block, inner1, &mut list_state);
}

fn block_content_list(app_state: &mut ApplicationState) -> List {
    let block = Block::default().borders(Borders::ALL);
    app_state.list_length = 0;
    
    let mut items:Vec <ListItem> = Vec::new();
    let mut mapping: Vec<(usize, Option<usize>)> = Vec::new();

    if let page = &app_state.page_list[app_state.selected_page.unwrap()] {
        for (group_index, group) in page.group_list.iter().enumerate() {
            items.push(ListItem::new(Line::from(group.title.clone())));
            mapping.push((group_index, None));
            if group.show_items {
                for (todo_index, todo) in group.item_list.iter().enumerate() {
                    items.push(ListItem::new(Line::from(format!("    {}", todo.title.as_str()))));
                    mapping.push((group_index, Some(todo_index)));
                }
            }
        }
    }


    app_state.list_length = mapping.len();

    if mapping.is_empty() {
        app_state.item_list_state.select(None);
        app_state.selected_group = None;
        app_state.selected_todo = None;
    } else {
        let selected = app_state.item_list_state.selected().unwrap_or(0);
        let selected = selected.min(mapping.len() - 1);
        app_state.item_list_state.select(Some(selected));

        if let Some((group_index, todo_index)) = mapping.get(selected) {
            app_state.selected_group = Some(*group_index);
            app_state.selected_todo = *todo_index;
        }
    }

    List::new(items).block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
        )
}

fn render_add_select(frame: &mut Frame, app_state: &mut ApplicationState) {
    let w = 30;
    let h = 4;
    let rect = Rect::new((frame.area().width - w) / 2,
        (frame.area().height - h) / 2,
        w, h );

    let text = Paragraph::new(Text::from(vec![
            Line::from("(i) Add Item"),
            Line::from("(g) Add Group"),
    ])).block(Block::default().borders(Borders::ALL).title("Select"));

    frame.render_widget(text, rect);
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
                    .bg(Color::Green),
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

fn render_popup_input_field(frame: &mut Frame, app_state: &mut ApplicationState, title: &str) {
    let width = 50;
    let height = 3;
    
    let rect = Rect::new((frame.area().width - width) / 2, (frame.area().height - height) / 2, width, height);
    let textbox = Block::default().borders(Borders::ALL).title(title);
    let buffer = Paragraph::new(app_state.buffer_string.clone()).block(textbox);
    
    frame.render_widget(buffer, rect); 
}

fn render_alert_box(frame: &mut Frame, alert_mode: &AlertMode, message: &str) {
    let width = 50;
    let height = 7;

    let rect = Rect::new((frame.area().width - width) / 2, (frame.area().height - height) / 2, width, height);

    let _title = match alert_mode {
        AlertMode::Message => "Message...",
        AlertMode::Warning => "Warning!",
        AlertMode::Error => "Error!",
        _ => "None",
    };
    let block = Block::default().borders(Borders::ALL).title(_title);
    let paragraph = Paragraph::new(message).block(block);
    frame.render_widget(paragraph, rect);
}

// ----------------------------- END OF RENDER -----------------------------

// --------------------------------- INPUT ---------------------------------

fn handle_input(key: KeyEvent, app_state: & mut ApplicationState) { // Routes input 
    match app_state.mode {
        TodoModes::Normal => handle_normal_input(key, app_state),
        TodoModes::PageSelect => handle_page_select_input(key, app_state),
        TodoModes::Insert => handle_insert(key, app_state),
        TodoModes::AddSelect => handle_add_select_input(key, app_state),
        TodoModes::Popup => handle_alert_box(key, app_state),
        _ => (),
    }
}

fn handle_page_select_input(key: KeyEvent, app_state: &mut ApplicationState) {
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
        KeyCode::Char('a') => app_state.mode = TodoModes::AddSelect,
        KeyCode::Char('k') | KeyCode::Up => {
            app_state.selected_item_up();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app_state.selected_item_down();
        }
        _ => (),
    }
}

fn handle_add_select_input(key: KeyEvent, app_state: &mut ApplicationState) {
    match key.code {
        KeyCode::Char('i') => {
            app_state.mode = TodoModes::Insert;
            app_state.input_mode = ActiveInput::AddTodo;
        }
        KeyCode::Char('g') => { 
            app_state.mode = TodoModes::Insert; 
            app_state.input_mode = ActiveInput::AddGroup;
        }
        KeyCode::Esc => {
            app_state.mode = TodoModes::Normal;
            app_state.input_mode = ActiveInput::None;
        }
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
                ActiveInput::AddGroup | ActiveInput::AddTodo => {
                    app_state.mode = TodoModes::Normal;
                    app_state.input_mode = ActiveInput::None;
                }
                _ => (),
            }
        }
        KeyCode::Enter => {
            app_state.buffer_string.trim();
            if !app_state.buffer_string.is_empty() {
                match app_state.input_mode {
                    ActiveInput::AddPage => {
                        app_state.add_page(app_state.buffer_string.clone());

                        app_state.mode = TodoModes::PageSelect;
                        app_state.input_mode = ActiveInput::None;
                    }
                    ActiveInput::AddGroup => {
                        let group_title = app_state.buffer_string.clone();
                        if let Some(page) = app_state.selected_mut_page() {
                            page.add_group(group_title);
                        }

                        app_state.mode = TodoModes::Normal;
                        app_state.input_mode = ActiveInput::None;
                    }
                    ActiveInput::AddTodo => {
                        let todo_title = app_state.buffer_string.clone();
                        if let Some(group) = app_state.selected_mut_group() {
                            group.add_todo(todo_title);
                            app_state.mode = TodoModes::Normal;
                            app_state.input_mode = ActiveInput::None;

                        }
                        else {
                            // TODO: have it create a new group "Untitled" and add todo on it. 
                            app_state.alert_box(AlertMode::Error,
                            "Please have a group selected/highlighted \n 
                            to create a todo item".to_string());
                        }
                    }
                    _ => (),
                }

                app_state.buffer_string.clear();
            }
        }
        _ => ()
    }
}

fn handle_alert_box(key: KeyEvent, app_state: &mut ApplicationState) {
    match key.code {
        _ => {
            match app_state.input_mode {
                ActiveInput::AddPage | ActiveInput::None => {
                    app_state.mode = TodoModes::PageSelect;
                    app_state.input_mode = ActiveInput::None;
                    app_state.has_popup = false;
                }
                ActiveInput::AddGroup | ActiveInput::AddTodo => {
                    app_state.mode = TodoModes::Normal;
                    app_state.input_mode = ActiveInput::None;
                    app_state.has_popup = false;
                }
            }
        }
    }
}
