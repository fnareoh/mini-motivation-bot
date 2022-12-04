use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

// sleep
use tokio::time::sleep;
use std::time::Duration;
use tokio::task;

// Date and time info
use chrono::prelude::*;
use chrono::Duration as ChronoDuration;

struct Handler;

// A mutable bool stored in the context of a client.
pub struct Reminder;
impl serenity::prelude::TypeMapKey for Reminder {
    type Value = bool;
}

// TODO in the future: add a check to see if has commit on github and add proper mention.


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
            println!()
        }
        else if msg.content == "!stop" {
            {
                let mut data = ctx.data.write().await;
                let remind = data.get_mut::<Reminder>().unwrap();
                *remind = false;
            }
        }
        else if msg.content == "!done" {
            let _concurrent_future = task::spawn(done_for_today(ctx,msg));
        }
        else if msg.content == "!motivateme" {
            let _concurrent_future = task::spawn(reminder_loop(ctx,msg));
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn done_for_today(ctx: Context, msg: Message){
    {
        let mut data = ctx.data.write().await;
        let remind = data.get_mut::<Reminder>().unwrap();
        *remind = false;
    }
    let starting_day = Local::now().date_naive();
    let time_start_again = NaiveDateTime::new(starting_day+ ChronoDuration::days(1), NaiveTime::from_hms_opt(18, 0, 0).unwrap());
    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Good! I won't bother you until {}!",time_start_again)).await {
        println!("Error sending message: {:?}", why);
    }
    sleep((time_start_again-Local::now().naive_local()).to_std().unwrap()).await;
    let _concurrent_future = task::spawn(reminder_loop(ctx,msg));
}

async fn reminder_loop(ctx: Context, msg: Message) {
    {
        let mut data = ctx.data.write().await;
        let remind = data.get_mut::<Reminder>().unwrap();
        *remind = true;
    }
    let mut remind = true;
    while remind {
        {
            let data = ctx.data.read().await;
            remind = *data.get::<Reminder>().unwrap();
        }
        if let Err(why) = msg.channel_id.say(&ctx.http, "Hey don't forget to have fun coding!").await {
            println!("Error sending message: {:?}", why);
        }
        let time_betwwen_reminder = 60*30;
        sleep(Duration::from_secs(time_betwwen_reminder)).await;
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Reminder>(false);
    }

    // Start a single shard, and start listening to events.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

