use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc,
};

use discord::{async_channel::Sender, *};

#[macro_use]
extern crate log;

// Use simplelog with a file and the console.
fn init_logger() {
	use simplelog::*;
	use std::fs::File;

	CombinedLogger::init(vec![
		TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
		WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("CheeseBot.log").unwrap()),
	])
	.unwrap();

	info!("Initalised logger!");
}

async fn send_heartbeat(send_outgoing_message: &Sender<String>, sequence_number: &Arc<AtomicUsize>) {
	let val = sequence_number.load(Ordering::SeqCst);
	let heartbeat = if val == usize::MAX { None } else { Some(val) };

	send_outgoing_message
		.send(serde_json::to_string(&GatewaySend::Heartbeat { d: heartbeat }).unwrap())
		.await
		.unwrap();
}

/// Sends a heartbeat every `period` milliseconds
async fn heartbeat(send_outgoing_message: Sender<String>, sequence_number: Arc<AtomicUsize>, interval: u64) {
	let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(interval));
	loop {
		interval.tick().await;
		send_heartbeat(&send_outgoing_message, &sequence_number).await;
	}
}

async fn handle_interaction(interaction: Interaction, client: &mut DiscordClient) {
	if let InteractionType::ApplicationCommand = interaction.interaction_type {
		//let data = interaction.data.unwrap();

		// InteractionCallback::new(InteractionResponse::ChannelMessageWithSource {
		// 	data: ChannelMessage::new()
		// 		.with_content("Hello")
		// 		//.with_components(ActionRows::new().with_components(Button::new().with_custom_id("yyy").with_label("yyy")))
		// 		// .with_components(
		// 		// 	ActionRows::new().with_components(TextInput::new().with_custom_id("amount").with_label("Amount")),
		// 		// ),
		// })
		// .post_respond(&mut client, interaction.id, interaction.token)
		// .await
		// .unwrap();
		InteractionCallback::new(InteractionResponse::ChannelMessageWithSource {
			data: ChannelMessage::new().with_embeds(Embed::standard().with_description("Hello world").with_title("The end of the world!")),
		})
		.post_respond(client, interaction.id, interaction.token)
		.await
		.unwrap();
	} else {
		warn!("Recieved interaction of type {:?} which was not handled", interaction.interaction_type);
	}
}

/// Runs the bot
async fn run() {
	let mut client = DiscordClient::new(include_str!("token.txt"));

	let gateway = GatewayMeta::get_gateway_meta(&mut client).await.unwrap();
	info!("Recieved gateway metadata: {:?}", gateway);

	let Connection {
		send_outgoing_message,
		mut read,
		sequence_number,
	} = client.connect_gateway(gateway.url).await;

	while let Some(Ok(Message::Text(text))) = read.next().await {
		match serde_json::from_str(&text) {
			Ok(deserialised) => match deserialised {
				GatewayRecieve::Dispatch { d, s } => {
					sequence_number.store(s, Ordering::SeqCst);

					info!("Recieved dispatch {:?}", d);
					match d {
						Dispatch::Ready(r) => create_commands(&mut client, &r.application.id).await,
						Dispatch::InteractionCreate(interaction) => handle_interaction(interaction, &mut client).await,
						_ => warn!("Unhandled dispatch"),
					}
				}
				GatewayRecieve::Heartbeat { .. } => {
					warn!("Discord wants a heartbeat, sending");
					send_heartbeat(&send_outgoing_message, &sequence_number).await;
				}
				GatewayRecieve::Reconnect => todo!(),
				GatewayRecieve::InvalidSession { d } => error!("Invalid session, can reconnect {}", d),
				GatewayRecieve::Hello { d } => {
					let identify = GatewaySend::Identify {
						d: Identify::new()
							.with_intents(INTENTS_ALL_WITHOUT_PRIVILEDGED)
							.with_token(&client.token)
							.with_properties(ConnectionProperties::new().with_device("Cheese")),
					};

					info!("Recieved hello {:?}, sending identify {:?}", d, identify);

					send_outgoing_message.send(serde_json::to_string(&identify).unwrap()).await.unwrap();
					tokio::spawn(heartbeat(send_outgoing_message.clone(), sequence_number.clone(), d.heartbeat_interval));
				}
				GatewayRecieve::HeartbeatACK => {}
			},
			Err(_) => {
				error!("Error decoding gateway message");
			}
		}
	}
}

async fn create_commands(client: &mut DiscordClient, application_id: &String) {
	ApplicationCommandList::new()
		.with_commands(
			ApplicationCommand::new()
				.with_command_type(CommandType::Chat)
				.with_name("about")
				.with_description("Description of the bot."),
		)
		.with_commands(
			ApplicationCommand::new()
				.with_command_type(CommandType::Chat)
				.with_name("balances")
				.with_description("All of your balances."),
		)
		.with_commands(
			ApplicationCommand::new()
				.with_command_type(CommandType::Chat)
				.with_name("pay")
				.with_description("Give someone cheesecoins.")
				.with_options(
					ApplicationCommandOption::new()
					.with_option_type(CommandOptionType::String)
						.with_name("recipiant").with_description("Recipiant of the payment")
						.with_required(true).with_autocomplete(true),
				)
				.with_options(
					ApplicationCommandOption::new()
						.with_option_type(CommandOptionType::Number)
						.with_name("cheesecoin")
						.with_description("Number of cheesecoin")
						.with_required(true),
				)
				.with_options(
					ApplicationCommandOption::new()
					.with_option_type(CommandOptionType::String)
						.with_name("from").with_description("The account the cheesecoins are from")
						.with_required(true),
				)
		)
		.with_commands(
			ApplicationCommand::new()
				.with_command_type(CommandType::Chat)
				.with_name("organisation")
				.with_description("Organisation commands")
				.with_options(ApplicationCommandOption::new()
					.with_name("create")
					.with_description("Create an organisation.")
				.with_options(ApplicationCommandOption::new().with_option_type(CommandOptionType::String).with_name("name").with_required(true).with_description("The name of the new organisation"))),
		)
		// .with_commands(
		// 	ApplicationCommand::new()
		// 		.with_command_type(CommandType::Chat)
		// 		.with_name("")
		// 		.with_description(""),
		// )
		.put_bulk_override_global(client, application_id)
		.await
		.unwrap();
}

fn main() {
	init_logger();

	tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(run());
}
