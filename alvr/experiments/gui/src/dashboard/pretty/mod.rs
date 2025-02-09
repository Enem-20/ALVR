use alvr_common::ServerEvent;
use alvr_session::SessionDesc;
use iced::{
    executor,
    futures::{
        channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
        lock::Mutex,
        stream::{self, BoxStream},
        StreamExt,
    },
    window::{self, Position},
    Application, Command, Element, Settings, Subscription, Text,
};
use iced_native::subscription::Recipe;
use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Debug)]
enum DashboardEvent {
    ServerEvent(ServerEvent),
}

pub struct EventsRecipe {
    receiver: Arc<Mutex<UnboundedReceiver<ServerEvent>>>,
}

impl<H: Hasher, E> Recipe<H, E> for EventsRecipe {
    type Output = ServerEvent;

    fn hash(&self, state: &mut H) {
        TypeId::of::<Self>().hash(state);
    }

    fn stream(self: Box<Self>, _: BoxStream<E>) -> BoxStream<ServerEvent> {
        let receiver = Arc::clone(&self.receiver);
        Box::pin(stream::unfold((), move |_| {
            let receiver = Arc::clone(&receiver);
            async move { Some((receiver.lock().await.next().await?, ())) }
        }))
    }
}

struct InitData {
    session: SessionDesc,
    request_handler: Box<dyn FnMut(String) -> String>,
    event_receiver: Arc<Mutex<UnboundedReceiver<ServerEvent>>>,
}

struct DashboardApp {
    session: SessionDesc,
    request_handler: Box<dyn FnMut(String) -> String>,
    event_receiver: Arc<Mutex<UnboundedReceiver<ServerEvent>>>,
}

impl Application for DashboardApp {
    type Executor = executor::Default;
    type Message = DashboardEvent;
    type Flags = InitData;

    fn new(init_data: InitData) -> (Self, Command<DashboardEvent>) {
        (
            Self {
                session: init_data.session,
                request_handler: init_data.request_handler,
                event_receiver: init_data.event_receiver,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "ALVR Dashboard".into()
    }

    fn update(&mut self, event: DashboardEvent) -> Command<DashboardEvent> {
        Command::none()
    }

    fn view(&mut self) -> Element<DashboardEvent> {
        Text::new("test").into()
    }

    fn subscription(&self) -> Subscription<DashboardEvent> {
        Subscription::from_recipe(EventsRecipe {
            receiver: Arc::clone(&self.event_receiver),
        })
        .map(DashboardEvent::ServerEvent)
    }
}

pub struct Dashboard {
    event_sender: UnboundedSender<ServerEvent>,
    event_receiver: Arc<Mutex<UnboundedReceiver<ServerEvent>>>,
}

impl Dashboard {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded();
        Self {
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }
    }

    pub fn run(&self, session: SessionDesc, request_handler: Box<dyn FnMut(String) -> String>) {
        DashboardApp::run(Settings {
            id: None,
            window: window::Settings {
                size: (800, 600),
                position: Position::Centered,
                icon: None, // todo
                ..Default::default()
            },
            flags: InitData {
                session,
                request_handler,
                event_receiver: Arc::clone(&self.event_receiver),
            },
            default_font: None,
            default_text_size: 20,
            text_multithreading: false,
            antialiasing: false,
            exit_on_close_request: true,
        })
        .unwrap();
    }

    pub fn report_event(&self, event: ServerEvent) {
        self.event_sender.unbounded_send(event).unwrap();
    }
}
