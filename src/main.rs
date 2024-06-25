use iced::widget::{container, text, Column};
use iced::{executor, Application, Command, Element, Settings, Subscription, Theme};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use chrono::{DateTime, Local};
use objc::{msg_send, sel, sel_impl, class};
use objc::runtime::Object;
use std::sync::{Arc, Mutex};

struct AppUsage {
    start_time: DateTime<Local>,
    duration: Duration,
}

struct ProductivityApp {
    current_app: String,
    start_time: Instant,
    app_usage: Arc<Mutex<HashMap<String, AppUsage>>>,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Exit,
}

impl Application for ProductivityApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let app_usage = Arc::new(Mutex::new(HashMap::new()));
        let app_usage_clone = Arc::clone(&app_usage);

        ctrlc::set_handler(move || {
            print_summary(&app_usage_clone.lock().unwrap());
            std::process::exit(0);
        }).expect("error setting ctrl-c handler");

        (
            Self {
                current_app: String::new(),
                start_time: Instant::now(),
                app_usage,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("kokomi")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                let new_app = get_active_window_info();
                if new_app != self.current_app {
                    let now = Instant::now();
                    let duration = now.duration_since(self.start_time);
                    
                    if !self.current_app.is_empty() {
                        let mut app_usage = self.app_usage.lock().unwrap();
                        app_usage.entry(self.current_app.clone())
                            .and_modify(|usage| usage.duration += duration)
                            .or_insert(AppUsage {
                                start_time: Local::now() - duration,
                                duration,
                            });
                    }
                    
                    self.current_app = new_app;
                    self.start_time = now;
                }
                Command::none()
            }
            Message::Exit => {
                print_summary(&self.app_usage.lock().unwrap());
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = Column::new()
            .spacing(20)
            .push(text("kokomi wants you to do your work").size(40))
            .push(text(format!("current app: {}", self.current_app)).size(20))
            .push(text("press ctrl+c to exit and view summary").size(16));

        container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
}

fn get_active_window_info() -> String {
    unsafe {
        let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        let app: *mut Object = msg_send![workspace, frontmostApplication];
        let app_name: *mut Object = msg_send![app, localizedName];
        
        let chars: *const u8 = msg_send![app_name, UTF8String];
        let len: usize = msg_send![app_name, lengthOfBytesUsingEncoding:4];
        String::from_utf8_lossy(std::slice::from_raw_parts(chars, len)).to_string()
    }
}

fn print_summary(app_usage: &HashMap<String, AppUsage>) {
    println!("\napp usage summary:");
    for (app, usage) in app_usage {
        println!("{}: {} seconds", app, usage.duration.as_secs());
    }
}

fn main() -> iced::Result {
    ProductivityApp::run(Settings::default())
}
