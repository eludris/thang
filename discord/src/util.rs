use twilight_model::channel::Message;

// enum DefaultAvatar {
//     Blurple = 0,
//     Grey = 1,
//     Green = 2,
//     Orange = 3,
//     Red = 4,
// }

pub fn get_default_avatar(discriminator: u16) -> String {
    let index = discriminator % 5;

    format!("https://cdn.discordapp.com/embed/avatars/{}.png", index)
}

pub fn get_avatar_url(message: &Message) -> String {
    match message.member.as_ref() {
        Some(member) => match member.avatar.as_ref() {
            Some(avatar) => Some(avatar),
            None => message.author.avatar.as_ref(),
        },
        None => message.author.avatar.as_ref(),
    }
    .map(|i| {
        format!(
            "https://cdn.discordapp.com/embed/avatars/{}.{}",
            String::from_utf8(i.bytes().to_vec()).unwrap_or_else(|_| String::from("0")),
            if i.is_animated() { "gif" } else { "png" }
        )
    })
    .unwrap_or_else(|| get_default_avatar(message.author.discriminator))
}
