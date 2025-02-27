use regex::Regex;
use serenity::{
    framework::standard::{
        macros::{command, group, help},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};
use std::time::Instant;

use super::super::bot;
use super::super::error::BotError;
use super::super::settings;
use super::super::sort::Sorted;
use super::{executors, parser};
use crate::bot::ContestData;
use crate::try_say;
use indexmap::IndexMap;
use serenity::framework::standard::{help_commands, CommandGroup, HelpOptions};
use serenity::model::id::UserId;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;

macro_rules! count {
    ( $x:ident ) => (1usize);
    ( $x:ident, $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! quiz_commands {
    () => {};
    ( $command:ident, $( $commands:ident ),* ) => {
        group!({
            name: "quiz",
            options: {
                description: "A group with commands providing a quiz with specific language as response.",
            },
            commands: [$command, $($commands),*],
        });
        const COMMAND_NUM: usize = count!($($commands),*) + 1;
        lazy_static! {
            pub static ref QUIZ_COMMANDS: [String; COMMAND_NUM] = [stringify!($command).to_string(), $(stringify!($commands).to_string(),)*];
            pub static ref QUIZ_COMMANDS_REGEX: Regex = Regex::new(
                &vec!["^(contest|", stringify!($command), $("|", stringify!($commands),)* ")$"].join("")
            ).unwrap();
        }
    };
}

quiz_commands!(en, ja, fr, de, it, ru, eo);
/*
quiz_commands! {
    en: {
        dictionary = english,
    },
    ja: {
        dictionary = japanese,
    },
    fr: {
        dictionary = french,
    },
    de: {
        dictionary = german,
    },
};
*/
group!({
    name: "extra",
    options: {
        description: "A group with commands providing hint and giveup.",
    },
    commands: [giveup, hint],
});

group!({
    name: "contest",
    options: {
        description: "A group with commands providing contest mode.",
    },
    commands: [contest, unrated],
});

group!({
    name: "settings",
    options: {
        description: "A group with commands providing settings of enable/disable switch in channel.",
    },
    commands: [enable, disable],
});

#[command]
#[description = "Provides a quiz of English as response."]
#[bucket = "basic"]
pub fn en(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~en' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut guard) = bot::QUIZ.lock();
        then {
            let ans = executors::prob(ctx, &msg, bot::Lang::En);
            *guard = bot::Status::Holding(ans, bot::Lang::En, Instant::now());
        }
    }
    Ok(())
}

#[command]
#[description = "Provides a quiz of Japanese as response."]
#[bucket = "basic"]
pub fn ja(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~ja' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut guard) = bot::QUIZ.lock();
        then {
            let ans = executors::prob(ctx, &msg, bot::Lang::Ja);
            *guard = bot::Status::Holding(ans, bot::Lang::Ja, Instant::now());
        }
    }
    Ok(())
}
#[command]
#[description = "Provides a quiz of French as response."]
#[bucket = "basic"]
pub fn fr(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~fr' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut guard) = bot::QUIZ.lock();
        then {
            let ans = executors::prob(ctx, &msg, bot::Lang::Fr);
            *guard = bot::Status::Holding(ans, bot::Lang::Fr, Instant::now());
        }
    }
    Ok(())
}
#[command]
#[description = "Provides a quiz of German as response."]
#[bucket = "basic"]
pub fn de(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~de' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut guard) = bot::QUIZ.lock();
        then {
            let ans = executors::prob(ctx, &msg, bot::Lang::De);
            *guard = bot::Status::Holding(ans, bot::Lang::De, Instant::now());
        }
    }
    Ok(())
}
#[command]
#[description = "Provides a quiz of Italian as response."]
#[bucket = "basic"]
pub fn it(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~it' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut guard) = bot::QUIZ.lock();
        then {
            let ans = executors::prob(ctx, &msg, bot::Lang::It);
            *guard = bot::Status::Holding(ans, bot::Lang::It, Instant::now());
        }
    }
    Ok(())
}
#[command]
#[description = "Provides a quiz of Russian as response."]
#[bucket = "basic"]
pub fn ru(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~ru' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut guard) = bot::QUIZ.lock();
        then {
            let ans = executors::prob(ctx, &msg, bot::Lang::Ru);
            *guard = bot::Status::Holding(ans, bot::Lang::Ru, Instant::now());
        }
    }
    Ok(())
}
#[command]
#[description = "Provides a quiz of Esperanto as response."]
#[bucket = "basic"]
pub fn eo(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~eo' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut guard) = bot::QUIZ.lock();
        then {
            let ans = executors::prob(ctx, &msg, bot::Lang::Eo);
            *guard = bot::Status::Holding(ans, bot::Lang::Eo, Instant::now());
        }
    }
    Ok(())
}

fn giveup_impl(ctx: &mut Context, msg: &Message, quiz_stat: &mut bot::Status) -> CommandResult {
    if !msg.author.bot {
        if quiz_stat.is_standing_by() {
            try_say!(ctx, msg, "現在問題は出ていません。");
        } else if quiz_stat.is_holding() {
            try_say!(
                ctx,
                msg,
                format!("正解は \"{}\" でした...", quiz_stat.ans().unwrap())
            );
            *quiz_stat = bot::Status::StandingBy;
        } else {
            let contest_result = &mut *bot::CONTEST_RESULT.lock().unwrap();
            *contest_result
                .entry("~giveup".to_string())
                .or_insert(ContestData::default()) += quiz_stat.elapsed().unwrap();
            if !quiz_stat.is_contest_end() {
                try_say!(
                    ctx,
                    msg,
                    format!("正解は \"{}\" でした...", quiz_stat.ans().unwrap())
                );
                quiz_stat.contest_continue(ctx, &msg);
            } else {
                let (_, num) = quiz_stat.get_contest_num().unwrap();
                msg.channel_id
                    .say(
                        &ctx,
                        format!(
                            "正解は \"{ans}\" でした...\n{num}問連続のコンテストが終了しました。\n{result}",
                            ans = quiz_stat.ans().unwrap(),
                            num = num,
                            result = bot::aggregates(contest_result)
                        ),
                    )
                    .expect("fail to post");
                *contest_result = IndexMap::new();
                *quiz_stat = bot::Status::StandingBy;
            }
        }
    }
    Ok(())
}

#[command]
#[description = "Allows to give up current quiz and shows answer as response."]
#[bucket = "basic"]
pub fn giveup(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~giveup' by user '{}'", msg.author.name);
    if let Ok(mut guard) = bot::QUIZ.lock() {
        println!("giveup is accepted");
        giveup_impl(ctx, msg, &mut *guard)?;
    }
    Ok(())
}

#[command]
#[description = "Starts contest mode."]
#[bucket = "long"]
pub fn contest(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    use crate::bot::CONTEST_LIBRARY;
    println!("Got command '~contest' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut quiz_guard) = bot::QUIZ.lock();
        if quiz_guard.is_standing_by();
        then {
            match parser::contest(&mut args) {
                Err(err_msg) => {
                    try_say!(ctx,msg,err_msg);
                    return Ok(());
                }
                Ok((num, mut languages)) => {
                    languages.sort();
                    languages.dedup();
                    CONTEST_LIBRARY.lock().unwrap().set(languages);
                    let (dic, lang) = CONTEST_LIBRARY
                        .lock()
                        .unwrap()
                        .select(&mut rand::thread_rng());
                    let ans = dic.get(&mut rand::thread_rng());
                    msg.channel_id
                        .say(
                            &ctx,
                            format!(
                                "{number}問のコンテストを始めます。\n問 1 (1/{number})\nソートなぞなぞ ソート前の {symbol} な〜んだ？\n`{prob}`",
                                number = num,
                                prob = ans.sorted(),
                                symbol = lang.as_symbol(),
                            ),
                        )
                        .expect("fail to post");
                    *quiz_guard = bot::Status::Contesting(ans.to_string(), lang, (1, num), Instant::now());
                }
            }
        }
    }
    Ok(())
}

#[command]
#[description = "Force closes current contest."]
#[bucket = "long"]
pub fn unrated(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~unrated' by user '{}'", msg.author.name);
    loop {
        if let (Ok(mut quiz), Ok(mut result)) = (bot::QUIZ.lock(), bot::CONTEST_RESULT.lock()) {
            if quiz.is_contesting() {
                try_say!(ctx, msg, "コンテストを中止します。");
                *quiz = bot::Status::StandingBy;
                *result = IndexMap::new();
            } else {
                try_say!(ctx, msg, "現在コンテストは開催されていません。");
            }
            break;
        }
    }
    Ok(())
}

#[command]
#[description = "Gives hint as response."]
#[bucket = "long"]
pub fn hint(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("Got command '~hint' by user '{}'", msg.author.name);
    if_chain! {
        if !msg.author.bot;
        if let Ok(mut guard) = bot::QUIZ.lock();
        if !guard.is_standing_by();
        then {
            let mut g = UnicodeSegmentation::graphemes(guard.ans().unwrap().as_str(), true).collect::<Vec<&str>>();
            match parser::hint(&mut args) {
                Err(err_msg) => {
                    try_say!(ctx,msg,format!("{}", err_msg));
                },
                Ok(parser::Hint::First(num)) | Ok(parser::Hint::Random(num)) if num == 0 => {
                    try_say!(ctx,msg,"ゼロ文字ヒントはだせません。");
                },
                Ok(parser::Hint::First(num)) | Ok(parser::Hint::Random(num)) if num == g.len() || num == g.len() - 1 => {
                    try_say!(ctx,msg,"答えが一意に定まるためギブアップとみなされました！");
                    giveup_impl(ctx, msg, &mut *guard)?;
                },
                Ok(parser::Hint::First(num)) | Ok(parser::Hint::Random(num)) if num > g.len() => {
                    try_say!(ctx,msg,"ヒントが文字数を超えていますｗ");
                },
                Ok(parser::Hint::First(num)) => {
                    g.truncate(num);
                    msg.channel_id
                        .say(
                            &ctx,
                            format!(
                                "答えの先頭 {len} 文字は... => `{hint}` ",
                                len = num,
                                hint = g.into_iter().collect::<String>(),
                            ),
                        )
                        .expect("fail to post");
                },
                Ok(parser::Hint::Random(num)) => {
                    let star = "*";
                    let mut hit_str: Vec<&str> = std::iter::repeat(star).take(g.len()).collect();
                    for idx in rand::seq::index::sample(&mut rand::thread_rng(), g.len(), num).into_iter() {
                        if let Some(elem) = hit_str.get_mut(idx) {
                            *elem = g.get(idx).unwrap();
                        }
                    }
                    msg.channel_id
                        .say(
                            &ctx,
                            format!(
                                "ランダムヒント {len} 文字... => `{hint}` ",
                                len = num,
                                hint = hit_str.join(""),
                            ),
                        )
                        .expect("fail to post");
                },
            }
        } else {
            match parser::hint(&mut args) {
                Err(err_msg) => {
                    try_say!(ctx,msg,format!("{}", err_msg));
                },
                Ok(_) => {
                    try_say!(ctx,msg,"問題が出てないですよ？");
                },
            }
        }
    }
    Ok(())
}

#[help]
fn nazonazo_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

fn sync_setting() -> Result<(), BotError> {
    use quick_error::ResultExt;
    let path = std::path::Path::new("/tmp/settings/settings.toml");
    let mut conf = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .context(path)?;
    conf.write_all(
        toml::to_string(&*settings::SETTINGS.lock().unwrap())
            .context("/tmp/settings/settings.toml")?
            .as_bytes(),
    )
    .context(path)?;
    conf.sync_all().context(path)?;
    Ok(())
}

#[command]
#[description = "Enable nazonazo bot on a channel."]
#[bucket = "long"]
pub fn enable(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~enable' by user '{}'", msg.author.name);
    if !settings::SETTINGS
        .lock()
        .unwrap()
        .channel
        .enabled
        .contains(&*msg.channel_id.as_u64())
    {
        settings::SETTINGS
            .lock()
            .unwrap()
            .channel
            .enabled
            .push(*msg.channel_id.as_u64());
        try_say!(
            ctx,
            msg,
            "このチャンネルでソートなぞなぞが有効になりました。"
        );
        Ok(sync_setting()?)
    } else {
        try_say!(ctx, msg, "このチャンネルでソートなぞなぞはすでに有効です。");
        Ok(())
    }
}

#[command]
#[description = "Disable nazonazo bot on a channel."]
#[bucket = "long"]
pub fn disable(ctx: &mut Context, msg: &Message) -> CommandResult {
    println!("Got command '~disable' by user '{}'", msg.author.name);
    settings::SETTINGS
        .lock()
        .unwrap()
        .channel
        .enabled
        .retain(|id| *id != *msg.channel_id.as_u64());
    try_say!(
        ctx,
        msg,
        "このチャンネルでソートなぞなぞが無効になりました。"
    );
    Ok(sync_setting()?)
}
