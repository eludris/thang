mod commands;

use std::collections::HashSet;
use std::env;

use dotenv::dotenv;
use serenity::async_trait;
use serenity::framework::standard::macros::{group, help};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
};
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;

use crate::commands::misc::*;

#[group("general")]
#[summary = "Miscellaneous commands (surprised?)"]
#[description = "Commands that are so random they don't fit into one specific group :lol:"]
#[commands(ping)]
struct General;

#[help]
#[individual_command_tip = "Hi.\n
Want info about a command? I have what you want.\n
Will I tell you it? depends."]
#[command_not_found_text = "The command you looked for literally doesn't exist L."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "->"]
#[lacking_permissions = "Hide"]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await?;
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        log::info!(
            "Connected as {}#{}",
            ready.user.name,
            ready.user.discriminator
        );
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let token = env::var("TOKEN").expect("Couldn't find \"TOKEN\" enviroment variable");

    let http = Http::new(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                team.members.iter().for_each(|m| {
                    owners.insert(m.user.id);
                });
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot) => (owners, bot.id),
                Err(why) => panic!("Couldn't get the bot's id: {:?}", why),
            }
        }
        Err(why) => panic!("Couldn't fetch owner & bot ids: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("-").on_mention(Some(bot_id)).owners(owners))
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
