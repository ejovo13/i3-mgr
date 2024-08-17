use ratatui::style::Style;

#[derive(PartialEq)]
pub(crate) enum RunningState {
    Running,
    Done,
}

#[derive(Copy, Clone, Debug)]
enum HomeLayoutSelectedWindow {
    Workspaces,
    Focused,
    Attached,
    Floating,
}

impl HomeLayoutSelectedWindow {
    fn render(frame: &mut Frame, model: &Model, home_layout: &HomeLayout) {
        let mut workspaces_state = ListState::default();

        frame.render_stateful_widget(
            List::new(
                model
                    .workspaces
                    .iter()
                    .map(|ws| format!("{} [{}]", ws.name.clone(), &ws.id.to_string())),
            )
            .block(Block::bordered().title("Workspaces"))
            .highlight_style(SELECTED_STYLE),
            home_layout.workspaces,
            &mut workspaces_state,
        );
    }
}

use crate::shutils;
use crate::shutils::i3_cmd;
use crate::shutils::pipe;

#[derive(Debug, Clone)]
struct HomeLayout {
    workspaces: Rect,
    focused_window: Rect,
    attached_windows: Rect,
    floating_windows: Rect,
    status: Rect,
    selected: HomeLayoutSelectedWindow,
    workspaces_index: u64,
    workspace_state: Option<ListState>,
    attached_windows_index: u64,
}

impl HomeLayout {
    /// handle adding an element to one of our windows
    fn handle_add(&mut self, model: &mut Model) -> Result<()> {
        match self.selected {
            HomeLayoutSelectedWindow::Workspaces => {
                self.add_workspace(model)?;
            }
            _ => (),
        }

        Ok(())
    }

    fn handle_consolidate(&mut self, target_workspace: &str, model: &mut Model) -> Result<()> {
        match self.selected {
            HomeLayoutSelectedWindow::Workspaces => {
                self.consolidate_workspaces(target_workspace, model)?
            }
            _ => (),
        }

        Ok(())
    }

    /// Move all windows from all workspaces to a single workspace.
    fn consolidate_workspaces(&mut self, target_workspace: &str, model: &mut Model) -> Result<()> {
        // Create a command to move all windows over
        //
        //
        //

        for (workspace_name, workspace_nodes) in model.ws_map.iter() {
            if workspace_name == target_workspace {
                // Do nothing.
            } else {
                let nodes = workspace_nodes
                    .iter()
                    .flat_map(|window| window.flatten())
                    .filter(|ws| ws.name.is_some());
                for node in nodes {
                    shutils::move_window_to_workspace(node.id, target_workspace)?;
                }
            }
        }

        self.workspaces_index = 0;
        model.refresh();

        Ok(())
    }

    fn show_scratchpad(&self) -> Result<String> {
        i3_cmd(&["scratchpad", "show"])
    }

    /// Send a message to i3 to create a new message
    fn add_workspace(&mut self, model: &mut Model) -> Result<String> {
        let _ = i3_cmd(&["workspace", &(self.workspaces_index + 1).to_string()])?;
        model.refresh();
        self.show_scratchpad()
    }

    fn decrement_attached_index(&mut self, n_windows: usize) {
        if self.attached_windows_index as usize == 0 {
            self.attached_windows_index = (n_windows - 1) as u64
        } else {
            self.attached_windows_index -= 1
        }
    }

    fn decrement_workspace_index(&mut self, n_workspaces: usize) {
        if self.workspaces_index as usize == 0 {
            self.workspaces_index = (n_workspaces - 1) as u64
        } else {
            self.workspaces_index -= 1
        }
    }

    fn increment_attached_index(&mut self, n_windows: usize) {
        if self.attached_windows_index as usize == n_windows - 1 {
            self.attached_windows_index = 0
        } else {
            self.attached_windows_index += 1
        }
    }

    /// Select the next workspace
    fn increment_workspace_index(&mut self, n_workspaces: usize) {
        if self.workspaces_index as usize == n_workspaces - 1 {
            self.workspaces_index = 0
        } else {
            self.workspaces_index += 1
        }
    }

    fn move_down_inside(&mut self, n_workspaces: usize, n_windows: usize) {
        match self.selected {
            HomeLayoutSelectedWindow::Workspaces => self.increment_workspace_index(n_workspaces),
            HomeLayoutSelectedWindow::Attached => self.increment_attached_index(n_windows),
            _ => (),
        }
    }

    fn move_up_inside(&mut self, n_workspaces: usize, n_windows: usize) {
        match self.selected {
            HomeLayoutSelectedWindow::Workspaces => self.decrement_workspace_index(n_workspaces),
            HomeLayoutSelectedWindow::Attached => self.decrement_attached_index(n_windows),
            _ => (),
        }
    }

    fn move_up(&mut self) {
        match self.selected {
            HomeLayoutSelectedWindow::Focused => {
                self.selected = HomeLayoutSelectedWindow::Workspaces;
            }
            HomeLayoutSelectedWindow::Floating => {
                self.selected = HomeLayoutSelectedWindow::Attached;
            }
            _ => (),
        }
    }

    fn move_down(&mut self) {
        match self.selected {
            HomeLayoutSelectedWindow::Workspaces => {
                self.selected = HomeLayoutSelectedWindow::Focused;
            }
            HomeLayoutSelectedWindow::Attached => {
                self.selected = HomeLayoutSelectedWindow::Floating;
            }
            _ => (),
        }
    }

    fn move_right(&mut self) {
        match self.selected {
            HomeLayoutSelectedWindow::Workspaces => {
                self.selected = HomeLayoutSelectedWindow::Attached;
            }
            HomeLayoutSelectedWindow::Focused => {
                self.selected = HomeLayoutSelectedWindow::Floating;
            }
            _ => (),
        }
    }

    fn move_left(&mut self) {
        match self.selected {
            HomeLayoutSelectedWindow::Attached => {
                self.selected = HomeLayoutSelectedWindow::Workspaces;
            }
            HomeLayoutSelectedWindow::Floating => {
                self.selected = HomeLayoutSelectedWindow::Focused;
            }
            _ => (),
        }
    }

    fn render_workspace(&self, frame: &mut Frame, model: &mut Model) {
        let mut workspaces_state = ListState::default();
        workspaces_state.select(Some(self.workspaces_index as usize));

        let border_style = match self.selected {
            HomeLayoutSelectedWindow::Workspaces => Style::new().blue(),
            _ => Style::new(),
        };

        // frame.render_widget(
        //     Text::raw(format!("{:?}", model.workspaces)),
        //     self.workspaces,
        // );

        model.update_status(&format!("Selected workspace: {}", self.workspaces_index));

        frame.render_stateful_widget(
            List::new(model.workspaces.iter().map(|ws| ws.name.clone()))
                .block(
                    Block::bordered()
                        .title("Workspaces")
                        .border_style(border_style),
                )
                .highlight_style(SELECTED_STYLE)
                // .highlight_style(Style::default().red().italic())
                .highlight_symbol("> "),
            self.workspaces,
            &mut workspaces_state,
        );
    }

    fn render_focused(&self, frame: &mut Frame, model: &Model) {
        let border_style = match self.selected {
            HomeLayoutSelectedWindow::Focused => Style::new().blue(),
            _ => Style::new(),
        };

        frame.render_widget(
            Paragraph::new(format!(
                "{}",
                model.fcsd_window.as_ref().unwrap().name_str()
            ))
            .wrap(Wrap::default())
            .block(
                Block::bordered()
                    .title("Focused Window")
                    .border_style(border_style),
            ),
            self.focused_window,
        );
    }

    fn render_floating(&self, frame: &mut Frame, model: &Model) {
        let mut windows_state = ListState::default();

        let border_style = match self.selected {
            HomeLayoutSelectedWindow::Floating => Style::new().blue(),
            _ => Style::new(),
        };

        // Render Attached windows on the far right
        frame.render_stateful_widget(
            List::new(
                model
                    .floating_windows
                    .iter()
                    .filter(|ws| ws.name.is_some())
                    .map(|ws| ws.name_str()),
            )
            .block(
                Block::bordered()
                    .title("Floating Windows")
                    .border_style(border_style),
            )
            .highlight_style(SELECTED_STYLE),
            self.floating_windows,
            &mut windows_state,
        );
    }

    fn render_attached(&self, frame: &mut Frame, model: &mut Model) -> Result<()> {
        let mut state = ListState::default();
        state.select(Some(self.attached_windows_index as usize));

        let border_style = match self.selected {
            HomeLayoutSelectedWindow::Attached => Style::new().blue(),
            _ => Style::new(),
        };

        frame.render_stateful_widget(
            List::new(
                model
                    .workspace_window_names(&model.selected_workspace())
                    .unwrap(),
            )
            .block(
                Block::bordered()
                    .title("Attached Windows")
                    .border_style(border_style),
            )
            .highlight_style(SELECTED_STYLE),
            self.attached_windows,
            &mut state,
        );

        Ok(())
    }
}

pub enum AppLayout {
    HomeLayout,
}

use std::fmt::format;
use std::time::SystemTime;

pub(crate) struct Model {
    windows: Vec<Window>,
    workspaces: Vec<Workspace>,
    attached_windows: Vec<Window>,
    floating_windows: Vec<Window>,
    fcsd_window: Option<Window>,
    ws_map: HashMap<String, Vec<Window>>,
    ws_map_names: HashMap<String, Vec<String>>,
    pub(crate) running_state: RunningState,
    status_msg: String,
    status_timestamp: SystemTime,
    startup_time: SystemTime,
    current_menu: AppLayout,
    home_layout: Option<HomeLayout>,
}

use crate::prelude::*;
use crate::shutils::cmd;
use crate::window::*;
use crate::workspace::Workspace;

use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use ratatui::widgets::Wrap;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
    Frame,
};
use std::time::Duration;

pub(crate) enum Message {
    Quit,
    ShowKey,
    RefreshModel,
    MoveUp,
    MoveRight,
    MoveLeft,
    MoveDown,
    MoveUpMenu,
    MoveRightMenu,
    MoveDownMenu,
    MoveLeftMenu,
    /// Used to add a workspace, window, etc
    Add,
    /// Consolidate all windows to a single workspace
    Consolidate,
    /// Focuse on a given window
    GoTo,
    /// Delete a workspace or window
    Delete,
}

enum AppScreen {
    Home,
}

use ratatui::layout::{Constraint, Direction, Layout};

fn home_layout(
    frame: &Frame,
    selected: HomeLayoutSelectedWindow,
    workspace_index: u64,
    attached_windows_index: u64,
) -> HomeLayout {
    let home_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(95), Constraint::Percentage(5)])
        .split(frame.area());

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(home_layout[0]);

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(layout[0]);

    HomeLayout {
        attached_windows: right_layout[0],
        floating_windows: right_layout[1],
        focused_window: left_layout[1],
        workspaces: left_layout[0],
        status: home_layout[1],
        selected,
        workspaces_index: workspace_index,
        attached_windows_index: attached_windows_index,
        workspace_state: None,
    }
}

use ratatui::prelude::*;

impl HomeLayout {
    fn render(&self, frame: &mut Frame, model: &mut Model) -> Result<()> {
        // Render all the workspaces that exist.
        self.render_workspace(frame, model);
        self.render_focused(frame, model);
        self.render_floating(frame, model);
        self.render_attached(frame, model)?;
        frame.render_widget(Text::raw(model.status_msg_display()), self.status);
        Ok(())
    }
}

const SELECTED_STYLE: style::Style = style::Style::new()
    .bg(SLATE.c800)
    .add_modifier(style::Modifier::BOLD);

impl Model {
    /// Initialize a new Model.
    pub(crate) fn new(frame: &Frame) -> Self {
        let windows = list_windows();
        let workspaces = list_workspaces();
        let ws_map = list_workspaces_and_windows();
        let ws_map_names = list_workspaces_and_window_names();
        let fcsd_window = get_focused_window();
        let floating_windows = list_floating_windows();
        let attached_windows = list_attached_windows();
        let startup_time = SystemTime::now();

        Model {
            windows,
            workspaces,
            fcsd_window,
            ws_map,
            ws_map_names,
            running_state: RunningState::Running,
            attached_windows,
            floating_windows,
            status_msg: "Initialized Application".to_string(),
            status_timestamp: startup_time,
            startup_time,
            current_menu: AppLayout::HomeLayout,
            home_layout: Some(home_layout(
                frame,
                HomeLayoutSelectedWindow::Workspaces,
                0,
                0,
            )),
        }
    }

    fn workspace_windows(&self, workspace_name: &str) -> Result<Vec<Window>> {
        let cloned_map = self.ws_map.clone();
        let nodes = cloned_map.get(workspace_name).unwrap();

        Ok(nodes
            .iter()
            .flat_map(|window| window.flatten())
            .filter(|ws| ws.name.is_some())
            .collect())
    }

    /// Send a focus command to get back to this window
    fn focus(&self) -> Result<String> {
        let focused = self.fcsd_window.clone().unwrap();
        i3_cmd(&[&format!(r#"[con_id="{}"]"#, focused.id), "focus"])
    }

    /// Retrieve all of the windows that belong to a given workspace
    fn workspace_window_names(&self, workspace_name: &str) -> Result<Vec<String>> {
        Ok(self
            .workspace_windows(workspace_name)?
            .iter()
            .map(|ws| ws.name_str())
            .collect())
    }

    /// Retrieve the name of the selected workspace
    fn selected_workspace(&self) -> String {
        self.workspaces
            .get(self.home_layout.as_ref().unwrap().workspaces_index as usize)
            .unwrap_or(&self.workspaces[0])
            .name
            .clone()
    }

    fn selected_attached_window(&self) -> Window {
        let selected_ws = self.selected_workspace();
        let nodes = self.workspace_windows(&selected_ws).unwrap();
        nodes
            .get(self.hl().attached_windows_index as usize)
            .unwrap_or(&nodes[0])
            .clone()
    }

    fn delete_attached_window(&mut self) -> Result<()> {
        let selected_window = self.selected_attached_window();
        selected_window.focus_window()?;
        Ok(())
    }

    /// Update the display value of the update status string
    pub(crate) fn update_status(&mut self, new_status: &str) {
        self.status_timestamp = SystemTime::now();
        self.status_msg = new_status.to_string();
    }

    /// Get the time elapsed from application startup in a human readable format
    fn elapsed_time_string(&self) -> String {
        let elapsed_time = self
            .status_timestamp
            .duration_since(self.startup_time)
            .unwrap();

        format!(
            "{:3}.{:03}",
            elapsed_time.as_secs() % 1000,
            elapsed_time.as_millis() % 1000
        )
    }

    /// Display the status message along with a timestamp.
    fn status_msg_display(&self) -> String {
        format!("[{}] {}", self.elapsed_time_string(), self.status_msg)
    }

    pub(crate) fn view(&mut self, frame: &mut Frame) -> Result<()> {
        match self.current_menu {
            AppLayout::HomeLayout => match &self.home_layout {
                Some(h_layout) => {
                    let layout = home_layout(
                        frame,
                        h_layout.selected,
                        h_layout.workspaces_index,
                        h_layout.attached_windows_index,
                    );
                    let _ = layout.render(frame, self);
                }
                None => (),
            },
        }

        Ok(())
    }

    pub(crate) fn handle_event(&mut self) -> Result<Option<Message>> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    return Ok(self.handle_key(key));
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn handle_key(&mut self, key: event::KeyEvent) -> Option<Message> {
        match key.code {
            // KeyCode::Char('j') => Some(Message::Increment),
            // KeyCode::Char('k') => Some(Message::Decrement),
            KeyCode::Char('q') => Some(Message::Quit),
            KeyCode::Char('s') => Some(Message::ShowKey),
            KeyCode::Char('r') => Some(Message::RefreshModel),
            KeyCode::Char('H') => Some(Message::MoveLeftMenu),
            KeyCode::Char('J') => Some(Message::MoveDownMenu),
            KeyCode::Char('K') => Some(Message::MoveUpMenu),
            KeyCode::Char('L') => Some(Message::MoveRightMenu),
            KeyCode::Char('h') => Some(Message::MoveLeft),
            KeyCode::Char('j') => Some(Message::MoveDown),
            KeyCode::Char('k') => Some(Message::MoveUp),
            KeyCode::Char('l') => Some(Message::MoveRight),
            KeyCode::Char('a') => Some(Message::Add),
            KeyCode::Char('c') => Some(Message::Consolidate),
            KeyCode::Char('d') => Some(Message::Delete),
            KeyCode::Enter => Some(Message::GoTo),
            _ => None,
        }
    }

    /// Delete a window or workspace
    fn handle_delete(&mut self) -> Result<()> {
        let hl = self.home_layout.clone().unwrap();
        match hl.selected {
            HomeLayoutSelectedWindow::Workspaces => {
                // Jump to the selected workspace
                //
                ()
            }
            HomeLayoutSelectedWindow::Attached => {
                let selected_window = self.selected_attached_window();
                selected_window.kill()?;
                self.refresh();
                self.update_status(&format!("Killed: {:?}", selected_window));
            }
            _ => (),
        }
        Ok(())
    }

    /// Jump to a specific window or workspace
    fn handle_goto(&mut self) -> Result<()> {
        let hl = self.home_layout.clone().unwrap();
        match hl.selected {
            HomeLayoutSelectedWindow::Workspaces => {
                // Jump to the selected workspace
                let selected = self.selected_workspace();
                shutils::i3_cmd(&["workspace", &selected])?;
                hl.show_scratchpad()?;
            }
            HomeLayoutSelectedWindow::Attached => {
                let selected_window = self.selected_attached_window();
                selected_window.focus_window()?;
                self.hl().show_scratchpad()?;
                self.update_status(&format!("Focused: {:?}", selected_window));
            }

            _ => (),
        }
        Ok(())
    }

    fn n_attached_windows(&self) -> usize {
        self.workspace_windows(&self.selected_workspace())
            .unwrap()
            .len()
    }

    /// Refresh the workspaces and windows that are being monitored.
    ///
    /// Updates the model in place.
    pub(crate) fn refresh(&mut self) {
        self.ws_map = list_workspaces_and_windows();
        self.ws_map_names = list_workspaces_and_window_names();
        self.fcsd_window = get_focused_window();
        self.ws_map_names = list_workspaces_and_window_names();
        self.floating_windows = list_floating_windows();
        self.attached_windows = list_attached_windows();
        self.workspaces = list_workspaces();
        self.windows = list_windows();
        self.update_status("Refreshed");
    }

    /// Retrieve a copy of the home layout (hl) of a given model
    pub(crate) fn hl(&self) -> HomeLayout {
        self.home_layout.as_ref().unwrap().clone()
    }

    pub(crate) fn hl_mut(&mut self) -> &mut HomeLayout {
        self.home_layout.as_mut().unwrap()
    }

    pub(crate) fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Quit => {
                self.running_state = RunningState::Done;
            }
            Message::RefreshModel => self.refresh(),
            Message::MoveUpMenu => match self.current_menu {
                AppLayout::HomeLayout => {
                    self.home_layout.as_mut().unwrap().move_up();
                }
            },
            Message::MoveDownMenu => match self.current_menu {
                AppLayout::HomeLayout => {
                    self.home_layout.as_mut().unwrap().move_down();
                }
            },
            Message::MoveLeftMenu => match self.current_menu {
                AppLayout::HomeLayout => {
                    self.home_layout.as_mut().unwrap().move_left();
                }
            },
            Message::MoveRightMenu => match self.current_menu {
                AppLayout::HomeLayout => {
                    self.home_layout.as_mut().unwrap().move_right();
                }
            },
            Message::MoveUp => match self.current_menu {
                AppLayout::HomeLayout => {
                    let n_attached = self.n_attached_windows();
                    if let Some(layout) = &mut self.home_layout {
                        layout.move_up_inside(self.workspaces.len(), n_attached);
                    }
                }
            },
            Message::MoveDown => match self.current_menu {
                AppLayout::HomeLayout => {
                    let n_attached = self.n_attached_windows();
                    if let Some(layout) = &mut self.home_layout {
                        layout.move_down_inside(self.workspaces.len(), n_attached);
                    }
                }
            },
            Message::MoveLeft => match self.current_menu {
                AppLayout::HomeLayout => {
                    self.home_layout.as_mut().unwrap().move_left();
                }
            },
            Message::MoveRight => match self.current_menu {
                AppLayout::HomeLayout => {
                    self.home_layout.as_mut().unwrap().move_right();
                }
            },
            Message::Add => match self.current_menu {
                AppLayout::HomeLayout => {
                    let mut hl = self.home_layout.clone().unwrap();
                    hl.handle_add(self).unwrap();
                }
            },
            Message::Consolidate => match self.current_menu {
                AppLayout::HomeLayout => {
                    let mut hl = self.hl();
                    hl.handle_consolidate(&self.selected_workspace(), self)
                        .unwrap();
                    self.refresh()
                }
            },
            Message::GoTo => match self.current_menu {
                AppLayout::HomeLayout => {
                    self.handle_goto().unwrap();
                }
            },
            Message::Delete => match self.current_menu {
                AppLayout::HomeLayout => {
                    self.handle_delete().unwrap();
                }
            },
            _ => (),
        }
        None
    }
}
