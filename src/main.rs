use color_eyre::{
    eyre::{WrapErr},
    Result,
};
use std::io;
use std::time::{SystemTime, Duration, Instant};
use chrono::{Local, NaiveTime};
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

const TICK_RATE: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase { Work , Break }

impl Phase {
    fn duration(self) -> Duration{
	match self {
	    Phase::Work => Duration::from_secs(25 * 60),
	    Phase::Break => Duration::from_secs(5 * 60),
	}
    }
}

#[derive(Debug)]
struct Timer {
    phase: Phase,
    remaining: Duration,
    running: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Timer {
    fn new() -> Self {
	Self { phase: Phase::Work, remaining: Phase::Work.duration(), running: false }
    }

    fn toggle(&mut self) {self.running = !self.running}

    fn tick(&mut self, dt: Duration) {
	if !self.running { return; }
	self.remaining = self.remaining.saturating_sub(dt);
	if self.remaining.is_zero() {
	    self.phase = match self.phase {
		Phase::Work => Phase::Break,
		Phase::Break => Phase::Work,
	    };
	    self.remaining = self.phase.duration();
	}
    }

    fn display(&self) -> String {
	let s = self.remaining.as_secs();
	format!("{:02}:{:02}", s/60, s%60)
    }

    
}


#[derive(Debug, Default)]
pub struct App {
    timer: Timer,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
	let mut last_tick = Instant::now();	
	while !self.exit {
	    terminal.draw(|frame| self.draw(frame))?;
	    
	    let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
	    if event::poll(timeout)? {
		self.handle_events()?;
	    }

	    let now = Instant::now();
	    self.timer.tick(now - last_tick);
	    last_tick = now;
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
	    KeyCode::Char(' ') => self.timer.toggle(),
	    _ => {}
	}
	Ok(())
    }

    fn exit(&mut self) {
	self.exit = true;
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
	    .border_set(border::THICK);
	
	let text = Text::from(vec![
	    Line::from(self.timer.display()).bold(),
	    Line::from(self.timer.running.to_string()).bold(),
	]);

	Paragraph::new(text)
	    .centered()
	    .block(block)
	    .render(area, buf);

    }
}




