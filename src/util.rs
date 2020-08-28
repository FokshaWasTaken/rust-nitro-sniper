use serenity::model::user::User;

pub fn user_to_tag(user: &User) -> String {
    format!("{}#{:0>4}", user.name, user.discriminator)
}
