use color_eyre::{
    eyre::{WrapErr},
    Result,
};
use std::io;
use std::time::{SystemTime, Duration, Instant};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Constraint},
    style::{Style,Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Padding},
    DefaultTerminal, Frame,
};

fn main() -> Result<()>{
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
pub struct App {
    timer: Timer,
    clock: u8,
    toggle: bool,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
	while !self.exit {
	    terminal.draw(|frame| self.draw(frame))?;
	    self.handle_events()?;
	    //self.on_time();
	}
	Ok(())
    }

    fn draw(&self, frame:&mut Frame) {
	frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()>{
	let timeout = Duration::from_secs_f64(1.0 / 60.0);
	if !event::poll(timeout)? {
	    return Ok(());
	}
	match event::read()? {
	    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
	    .handle_key_event(key_event)
	    .wrap_err_with(|| format!("handling key failed:\n{key_event:#?}")),
	    _ => Ok(()),
	}
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
	match key_event.code {
	    KeyCode::Char('q') => self.exit(),
	    KeyCode::Char(' ') => self.toggle_timer()?,
	    _ => {}
	}
	Ok(())
    }

    fn exit(&mut self) {
	self.exit = true;
    }

    fn toggle_timer(&mut self) -> Result<()>{
	self.toggle = !self.toggle;
	Ok(())
    }

    fn on_time(&mut self){
	if self.toggle {
	    (1..=10).for_each(|_| self.clock += 1);
	}
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf:&mut Buffer) {
	let title = Line::from(" PomodoroRo ".bold());
	let instructions = Line::from(vec![
		" Time Up ".into(),
		"<Space Bar>".blue().bold(),
		" Quit ".into(),
		"<Q> ".blue().bold(),
	    ]);

	let block = Block::bordered()
	    .title(title.centered())
	    .title_bottom(instructions.centered())
	    //.padding(Padding::top(2))
	    .border_set(border::THICK);
	
	let sys_time = SystemTime::now();
	let time = self.clock.to_string(); 
	let text = Text::from(Line::from(time).bold());
	let elapsed = self.timer.start.elapsed().as_secs_f32();
	let message = format!("Timer: {elapsed:.2?}s");

	Paragraph::new(message)
	    .centered()
	    //.style(Style::new().red().on_white())
	    .block(block)
	    .render(area, buf);

    }
}

#[derive(Debug)]
struct Timer {
    start: Instant,
}

impl Default for Timer {
    fn default() -> Self {
	Self {
	    start: Instant::now(),
	}
    }
}

impl Widget for &Timer {
    fn render(self, area: Rect, buf: &mut Buffer) {
	let elapsed = self.start.elapsed().as_secs_f32();
	let message = format!("Timer: {elapsed:.2?}s");
	message.render(area, buf);
    }
}
