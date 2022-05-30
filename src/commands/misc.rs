use std::{error::Error, sync::Arc};
use twilight_http::Client;
use twilight_model::{application::component, gateway::payload::incoming::MessageCreate};

pub async fn ping(
    http: &Arc<Client>,
    msg: &Box<MessageCreate>,
    _: Vec<&str>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    http.create_message(msg.channel_id)
        .content("Pong!")?
        .exec()
        .await?;

    Ok(())
}

pub async fn say(
    http: &Arc<Client>,
    msg: &Box<MessageCreate>,
    args: Vec<&str>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if args.is_empty() {
        http.create_message(msg.channel_id)
            .content("You have to supply some text for me to say smh.")?
            .exec()
            .await?;
        return Ok(());
    }
    http.create_message(msg.channel_id)
        .content(&args.join(" "))?
        .exec()
        .await?;

    Ok(())
}

pub async fn poll(
    http: &Arc<Client>,
    msg: &Box<MessageCreate>,
    args: Vec<&str>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if args.is_empty() {
        http.create_message(msg.channel_id)
            .content("A poll without options, such smart.")?
            .exec()
            .await?;
        return Ok(());
    }

    let args = args.join(" ");
    let options: Vec<&str> = args.split("--").collect();

    if options.len() > 5 {
        http.create_message(msg.channel_id)
            .content("Polls can only have up to 5 options because enoki was too lazy to use more than one action row")?
            .exec()
            .await?;
        return Ok(());
    }

    http.create_message(msg.channel_id)
        .content("Poll.")?
        .components(&vec![component::Component::ActionRow(
            component::ActionRow {
                components: options
                    .iter()
                    .map(|o| {
                        component::Component::Button(component::Button {
                            custom_id: Some("a".to_string()),
                            disabled: false,
                            emoji: None,
                            label: Some(o.to_string()),
                            style: component::button::ButtonStyle::Primary,
                            url: None,
                        })
                    })
                    .collect::<Vec<component::Component>>(),
            },
        )])?
        .exec()
        .await?;

    Ok(())
}
