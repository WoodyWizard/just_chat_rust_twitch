
use std::{io::{self, Stdout}, thread, time::Duration, ops::Deref};
use tui::{backend::CrosstermBackend, widgets::{Widget, Block, Borders, ListItem, List},
    layout::{Layout, Constraint, Direction}, Terminal};
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
use tokio::*;
 

#[tokio::main]
pub async fn main() -> Result<(), io::Error> {

    let (tx,rx): (Sender<String>, Receiver<String>) = channel();
    let mut Messages_buff: Vec<ListItem> = Vec::new();

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
                    let sending = String::from(msg.sender.name + " : " + &msg.message_text);
                    tx.send(sending).unwrap();
                }
                _ => {}
            }
        }
    });

    let chat_handle = tokio::spawn( async move {
        while true {
             Messages_buff.push(ListItem::new(rx.recv().unwrap()));

                terminal.draw(|f| {
                    let size = f.size();
                    let itemlist = List::new(Messages_buff.as_slice())
                        .block(Block::default().title("List").borders(Borders::ALL));
                    f.render_widget(itemlist, size);
                });
        }
       
        disable_raw_mode();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
            );
        terminal.show_cursor();
    });

    client.join("screamlark".to_owned()).unwrap();
    join_handle.await.unwrap();
    chat_handle.await.unwrap();
   
    Ok(())
}
