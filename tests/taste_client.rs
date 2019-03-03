use std::io::stdin;
use std::sync::mpsc::channel;
use std::thread;
use std::env;

use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};
use headless_chrome::{protocol::page::ScreenshotFormat, Browser, LaunchOptionsBuilder};
use headless_chrome::browser::tab::element::BoxModel;
use std::fs;
extern crate env_logger;

#[macro_use]
extern crate log;

mod logging;

const CONNECTION: &'static str = "ws://127.0.0.1:9222";


#[test]
fn browse_wikipedia() -> Result<(), failure::Error> {
	::std::env::set_var("RUST_LOG", "headless_chrome=trace,taste_client=debug");
	// logging::enable_logging();
	// ::std::env::set_var("RUST_LOG", "play_rs=debug");
    env_logger::init();
    let options = LaunchOptionsBuilder::default().build().expect("Failed to find chrome");
    let browser = Browser::new1(options)?;

	// let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    let tab = browser.wait_for_initial_tab()?;
	// tab.navigate_to(url)?
        // .wait_until_navigated()?;

	std::thread::sleep(std::time::Duration::from_secs(20));

	// info!("{:?}", tab);
	// let vp = tab.find_element("#ddlogin")?
	// 	.get_box_model()?
	// 	.content_viewport();

	// info!("{:?}", vp);
	

	// let jpeg_data = tab.capture_screenshot(ScreenshotFormat::JPEG(Some(100)), Some(vp), true)?;
	// let jpeg_data = tab.capture_screenshot(ScreenshotFormat::JPEG(None), Some(vp), true)?;
	// let jpeg_data = tab.capture_screenshot(ScreenshotFormat::JPEG(None), None, false)?;

	// 	    let jpeg_data = tab
    //     .navigate_to(url)?
    //     .wait_until_navigated()?
    //     .capture_screenshot(ScreenshotFormat::JPEG(Some(100)), None, true)?;
    // fs::write("screenshot.jpg", &jpeg_data)?;

	// std::thread::sleep(std::time::Duration::from_secs(20));
        
    // fs::write("screenshot.jpg", &jpeg_data)?;
    Ok(())
}


#[test]
fn entry() {
	println!("Connecting to {}", CONNECTION);

	let client = ClientBuilder::new(CONNECTION)
		.unwrap()
		.add_protocol("rust-websocket")
		.connect_insecure()
		.unwrap();

	println!("Successfully connected");

	let (mut receiver, mut sender) = client.split().unwrap();

	let (tx, rx) = channel();

	let tx_1 = tx.clone();

	let send_loop = thread::spawn(move || {
		loop {
			// Send loop
			let message = match rx.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Send Loop: {:?}", e);
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					let _ = sender.send_message(&message);
					// If it's a close message, just send it and then return.
					return;
				}
				_ => (),
			}
			// Send the message
			match sender.send_message(&message) {
				Ok(()) => (),
				Err(e) => {
					println!("Send Loop: {:?}", e);
					let _ = sender.send_message(&Message::close());
					return;
				}
			}
		}
	});

	let receive_loop = thread::spawn(move || {
		// Receive loop
		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(m) => m,
				Err(e) => {
					println!("Receive Loop: {:?}", e);
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					// Got a close message, so send a close message and return
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
				OwnedMessage::Ping(data) => {
					match tx_1.send(OwnedMessage::Pong(data)) {
						// Send a pong in response
						Ok(()) => (),
						Err(e) => {
							println!("Receive Loop: {:?}", e);
							return;
						}
					}
				}
				// Say what we received
				_ => println!("Receive Loop: {:?}", message),
			}
		}
	});

	loop {
		let mut input = String::new();

		stdin().read_line(&mut input).unwrap();

		let trimmed = input.trim();

		let message = match trimmed {
			"/close" => {
				// Close the connection
				let _ = tx.send(OwnedMessage::Close(None));
				break;
			}
			// Send a ping
			"/ping" => OwnedMessage::Ping(b"PING".to_vec()),
			// Otherwise, just send text
			_ => OwnedMessage::Text(trimmed.to_string()),
		};

		match tx.send(message) {
			Ok(()) => (),
			Err(e) => {
				println!("Main Loop: {:?}", e);
				break;
			}
		}
	}

	// We're exiting

	println!("Waiting for child threads to exit");

	let _ = send_loop.join();
	let _ = receive_loop.join();

	println!("Exited");
}