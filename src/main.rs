pub use tdiscord::error::Error;
pub use tdiscord::database::{Identity, Database};


use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::all::ExecuteWebhook;
use serenity::json::json;
use serenity::all::ChannelId;
use serenity::all::UserId;


const COMMAND_PREFIX: &'static str = ";";



struct Handler;

fn sanitize<'a>(txt: &'a str) -> &'a str
{
    let mut start: usize = 0;
    while txt[start..].starts_with(COMMAND_PREFIX)
    {
        start += COMMAND_PREFIX.len();
    }
    let out = &txt[start..];
    out
}



fn parse_text<'a>(raw_txt: &'a str) -> Option<(&'a str, &'a str)>
{
    let text = sanitize(raw_txt);
    let keyword: &str = match text.split_whitespace().nth(0)
    {
        None => {print!("No nickname keyword!"); return None;},
        Some(s) => s
    };
    let keywordpos = match text.find(keyword)
    {
        None => {print!("Cannot find start of keyword!!"); return None;},
        Some(i) => i
    };
    let actual_text = sanitize(&text[(keywordpos+keyword.len()) ..]);

    Some((keyword, actual_text))
}


async fn say_with_hook(ctx: &Context, channel_id: ChannelId, nick: String, avatar: Option<String>, text: &str, audit: Option<&str>) -> Result<(), Error>
{
    let map = json!({"name": "Momo's hook"});
    let hook = ctx.http.create_webhook(channel_id, &map, audit).await?;

    let mut builder = ExecuteWebhook::new();
    builder = builder.username(nick.as_str());

    if let Some(avatar) = avatar
    {
        builder = builder.avatar_url(avatar.as_str());
    }
    
    builder = builder.content(text);

    hook.execute(&ctx.http, false, builder).await?;
    
    hook.delete(&ctx.http).await?;
    Ok(())
}


    


async fn say_with_identity(ctx: &Context, channel_id: ChannelId, user_id: u64, keyword: &str, text: &str) -> Result<(), Error>
{
    let identity = Database::get_identity(user_id, keyword).await?;
    
    let (nick, avatar): (String, String)= match identity
    {
        None =>
        {
            channel_id.say(&ctx.http, format!("Nickname {keyword} does not exist for you!")).await?;
            return Ok(());
        },
        Some(Identity{keyword:_, nick, avatar}) => (nick, avatar)
    };
    let name = UserId::new(user_id).to_user(&ctx.http).await?.name;
    let audit = format!("User {} (id: {}) speaks with nick {}", name, user_id, nick);

    say_with_hook(ctx, channel_id, nick, Some(avatar), text, Some(audit.as_str())).await?;

    
    
    Ok(())
}

async fn process_message(ctx: Context, msg: Message) -> Result<(), Error>
{
    let user_id: u64 = msg.author.id.get();
    if msg.content.starts_with(COMMAND_PREFIX)
    {
        //            let guild_id: u64 = msg.guild_id.ok_or(Error::new("Couldnt retrieve guild id"))?.get();
        let (keyword, actual_text) = match parse_text(msg.content.as_str())
        {
            None => return Ok(()),
            Some((a,b)) => (a,b)
        };
        
        match keyword
        {
            "help" =>
            {
                msg.channel_id.say(&ctx.http,
                                   format!(r#"Command list:
```
| display this message                  | {COMMAND_PREFIX}help                                                      |
| list all your identities              | {COMMAND_PREFIX}list                                                      |
| add a new identity                    | {COMMAND_PREFIX}register <keyword> <nickname> [attached image for avatar] |
| remove identity                       | {COMMAND_PREFIX}forget <keyword>                                          |
| talk through to identity automaticaly | {COMMAND_PREFIX}switch <keyword>                                          |
| stop speaking through identity        | {COMMAND_PREFIX}stop                                                      |

Any other command will be interpreted as a keyword for a personality```"#)
                ).await?;
            },
            "list" =>
            {
                let identities = Database::get_identities(user_id).await?;

                if identities.len() == 0
                {
                    msg.channel_id.say(&ctx.http, "You have no registerd identities").await?;
                }
                else
                {
                    let mut txt = format!("Your registered identities:```");
                    for Identity{keyword, nick, ..} in identities
                    {
                        txt = format!("{}\n{} | {}", txt, keyword, nick);
                    }
                    txt += "```";
                    msg.channel_id.say(&ctx.http, txt.as_str()).await?;

                }
                
            },
            "register" =>
            {
                let (keyword, nick) = match parse_text(actual_text)
                {
                    Some(stuff) => stuff,
                    None => {
                        msg.channel_id.say(&ctx.http, "Malformed register command!").await?;
                        return Ok(());
                    }
                };
                
                let avatar = if msg.attachments.len() == 0
                {
                    match msg.author.avatar.map(|h| format!("{h:?}"))
                    {
                        Some(hash) => format!("https://cdn.discordapp.com/avatars/{:?}/{:?}.png", user_id, &hash[1..hash.len()-1]),
                        None => "https://pbs.twimg.com/media/GvIpQVUWAAAmcsO?format=jpg&name=4096x4096".to_owned()
                    }
                }
                else
                {
                    msg.attachments[0].url.clone()
                };

                Database::add_identity(user_id, keyword, nick, avatar.as_str()).await?;

                say_with_identity(&ctx, msg.channel_id, user_id, keyword, "Identity registered.").await?;

                
            },
            "forget" =>
            {
                let (keyword, _) = match parse_text(actual_text)
                {
                    Some(stuff) => stuff,
                    None => {
                        msg.channel_id.say(&ctx.http, "Malformed forget command!").await?;
                        return Ok(());
                    }
                };

                Database::remove_identity(user_id, keyword).await?;
                msg.channel_id.say(&ctx.http, format!("Removed identity '{}'", keyword)).await?;

            },
            "switch" =>
            {
                let (keyword, _) = match parse_text(actual_text)
                {
                    Some(stuff) => stuff,
                    None => {
                        msg.channel_id.say(&ctx.http, "Malformed switch command!").await?;
                        return Ok(());
                    }
                };
                if let Some(identity) = Database::get_identity(user_id, keyword).await?
                {
                    Database::set_switch(user_id, keyword).await?;
                    msg.channel_id.say(&ctx.http, format!("User now speaks as '{}'", identity.nick)).await?;
                }
                else
                {
                    msg.channel_id.say(&ctx.http, format!("No identity '{}' for this user", keyword)).await?;
                }
            },
            "stop" =>
            {
                Database::delete_switch(user_id).await?;
                msg.channel_id.say(&ctx.http, format!("User unswitched")).await?;
            },
            _ =>
            {
                say_with_identity(&ctx, msg.channel_id, user_id, keyword, actual_text).await?;
                msg.delete(&ctx.http).await?;
                
            }
        }

        
    }
    else if let Some(keyword) = Database::get_switch(user_id).await?
    {
        
        say_with_identity(&ctx, msg.channel_id, user_id, keyword.as_str(), msg.content.as_str()).await?;
        msg.delete(&ctx.http).await?;
    }
    Ok(())
        
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message)
    {
        if let Err(why) = process_message(ctx, msg).await
        {
            println!("Error processing message: {why:?}");
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() -> Result<(), Error>
{
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

        Database::init()?;

    
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
    Ok(())
}
