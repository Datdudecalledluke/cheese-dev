pub enum Intents {
	Guilds = 1 << 0,
	GuildMembers = 1 << 1,
	GuildBans = 1 << 2,
	GuildEmojis = 1 << 3,
	GuildIntegrations = 1 << 4,
	GuildWebhooks = 1 << 5,
	GuildInvites = 1 << 6,
	GuildVoiceStates = 1 << 7,
	GuildPresences = 1 << 8,
	GuildMessages = 1 << 9,
	GuildMessageReactions = 1 << 10,
	GuildMessageTyping = 1 << 11,
	DirectMessages = 1 << 12,
	DirectMessageReactions = 1 << 13,
	DirectMessageTyping = 1 << 14,
}

#[allow(dead_code)]
pub const INTENTS_ALL_WITHOUT_PRIVILEDGED: u64 = Intents::Guilds as u64
	| Intents::GuildBans as u64
	| Intents::GuildEmojis as u64
	| Intents::GuildIntegrations as u64
	| Intents::GuildWebhooks as u64
	| Intents::GuildInvites as u64
	| Intents::GuildVoiceStates as u64
	| Intents::GuildMessages as u64
	| Intents::GuildMessageReactions as u64
	| Intents::GuildMessageTyping as u64
	| Intents::DirectMessages as u64
	| Intents::DirectMessageReactions as u64
	| Intents::DirectMessageTyping as u64;
#[allow(dead_code)]
pub const INTENTS_ALL: u64 = INTENTS_ALL_WITHOUT_PRIVILEDGED | Intents::GuildMembers as u64 | Intents::GuildPresences as u64;
#[allow(dead_code)]
pub const INTENTS_NONE: u64 = 0;
