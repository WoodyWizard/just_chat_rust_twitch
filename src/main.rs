
use std::{io::{self, Stdout}, thread, time::Duration, ops::Deref};
use tui::{backend::CrosstermBackend, widgets::{Widget, Block, Borders, ListItem, List},
    layout::{Layout, Constraint, Direction}, Terminal, style::Color, style::Style};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::TwitchIRCClient;
use twitch_irc::{ClientConfig, SecureTCPTransport};
use twitch_irc::message::*;
use std::cell::{RefCell, Ref};
use std::sync::mpsc::{channel, Sender, Receiver};
use tokio::{*, task::JoinHandle};
use std::rc::Rc; 

#[tokio::main]
pub async fn main() -> Result<(), io::Error> {

    let (tx,rx): (Sender<PrivmsgMessage>, Receiver<PrivmsgMessage>) = channel();
    let mut messages_buff: Vec<ListItem> = Vec::new();
    let mut message_range: Vec<ListItem> = Vec::new();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
 
    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);
    let join_handle = tokio::spawn(async move{
        while let Some(message) = incoming_messages.recv().await {
            match message {
                ServerMessage::Privmsg(msg) => {
                    //let sending = String::from(msg.sender.name + " : " + &msg.message_text);
                    let ms = msg.clone();
                    tx.send(ms).unwrap();
                }
                _ => {}
            }
        }
    });

    let mut chat_handle =  tokio::spawn(async move  {
        while true {
            let msg = rx.recv().unwrap();
            let complex_msg = String::from(msg.sender.name + " : " + &msg.message_text);
            let mut color = msg.name_color;
            if let None = color {
                color = Some(RGBColor{r: 255, g: 255, b: 255});
            }
            let color = color.unwrap();
            messages_buff.push(ListItem::new(complex_msg).style(Style::default().fg(Color::Rgb(color.r, color.g, color.b))));
            let message_1 = messages_buff.last().unwrap().clone();
            message_range.push(message_1);

                terminal.draw(|f| {
                    let mut size = f.size();
                    if (message_range.len() as u16 > size.height - 2) { message_range.remove(0); }
                    let itemlist = List::new(message_range.as_slice())
                        .block(Block::default().title("Chat").borders(Borders::ALL));
                    f.render_widget(itemlist, size);
                }).unwrap();
        }
       
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
            ).unwrap();
        terminal.show_cursor().unwrap();
    });

    client.join("yatororain".to_owned()).unwrap();
    join_handle.await.unwrap();
    chat_handle.await.unwrap();

    Ok(())
}
