use serenity::{
    client::Client,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use std::collections::{BTreeSet, BTreeMap};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::iter::FromIterator;
use std::str::from_utf8;
use itertools::Itertools;

use super::super::bot;
use super::super::dictionary;
use super::super::sort::Sorted;
use crate::bot::get_dictionary;

pub(crate) fn prob(
    ctx: &mut Context,
    msg: &Message,
    lang: bot::Lang,
) -> String {
    println!("called prob");
    let dic = match lang {
        bot::Lang::En => &*dictionary::ENGLISH,
        bot::Lang::Ja => &*dictionary::JAPANESE,
        bot::Lang::Fr => &*dictionary::FRENCH,
        bot::Lang::De => &*dictionary::GERMAN,
        bot::Lang::It => &*dictionary::ITALIAN,
        bot::Lang::Ru => &*dictionary::RUSSIAN,
    };
    let ans = dic.get(&mut rand::thread_rng());
    let sorted = ans.sorted();
    msg.channel_id
        .say(
            &ctx,
            format!(
                "ソートなぞなぞ ソート前の {as_str} な〜んだ？\n{prob}",
                as_str = lang.as_symbol(),
                prob = sorted
            ),
        )
        .expect("fail to post");
    ans.clone()
}

pub(crate) fn kick(ctx: &mut Context, msg: &Message) -> std::io::Result<()> {
    use std::process::Command;
    let mut src = BufWriter::new(File::create("/tmp/main.rs")?);
    let code = format!(
        r#"fn kick() {{
    println!("ヒィンｗ");
}}
fn main() {{
    {}
}}
"#,
        &msg.content
    );
    println!("{}", code);
    src.write_all(code.as_bytes())?;
    src.flush()?;
    match Command::new("rustc").arg("/tmp/main.rs").output() {
        Ok(output) => {
            if output.status.success() {
                msg.channel_id.say(&ctx, "ヒィンｗ").expect("fail to post");
            } else {
                msg.channel_id
                    .say(&ctx, from_utf8(output.stderr.as_slice()).unwrap())
                    .expect("fail to post");
            }
        }
        Err(e) => {
            msg.channel_id
                .say(&ctx, format!("{:?}", e))
                .expect("fail to post");
        }
    }
    Ok(())
}

pub(crate) fn answer_check(ctx: &mut Context, msg: &Message) {
    if let Ok(mut quiz_guard) = bot::QUIZ.lock() {
        match quiz_guard.answer_check(&msg.content) {
            bot::CheckResult::WA => { // includes the case that bot is standing by.
                return;
            },
            bot::CheckResult::Assumed(ans) => {
                msg.channel_id
                    .say(
                        &ctx,
                        format!(
                            "{} さん、正解です！\n正解は\"{}\"でした！",
                            &msg.author.name, &ans
                        ),
                    )
                    .expect("fail to post");
            },
            bot::CheckResult::Anagram(ans) => {
                msg.channel_id
                    .say(
                        &ctx,
                        format!(
                            "{} さん、{} は非想定解ですが正解です！",
                            &msg.author.name,
                            &msg.content.to_lowercase()
                        ),
                    )
                    .expect("fail to post");
            },
            bot::CheckResult::Full(ans) => {
                msg.channel_id
                    .say(
                        &ctx,
                        format!(
                            "{} さん、{} は出題辞書に非想定解ですが正解です！",
                            &msg.author.name,
                            &msg.content.to_lowercase()
                        ),
                    )
                    .expect("fail to post");
            }
        }

        if quiz_guard.is_holding() {
            *quiz_guard = bot::Status::StandingBy;
        }

        if quiz_guard.is_contesting() {
            let mut contest_result = &mut *bot::CONTEST_REUSLT.lock().unwrap();

            *contest_result
                .entry(msg.author.name.clone())
                .or_insert(0) += 1;

            let (count, num) = quiz_guard.get_contest_num().unwrap();

            if quiz_guard.is_contest_end() {
                msg.channel_id
                    .say(
                        &ctx,
                        format!(
                            "{num}問連続のコンテストが終了しました。\n{result}",
                            num = num,
                            result = contest_result
                                .iter()
                                .sorted_by(|&(_, a), &(_, b)| b.cmp(&a))
                                .map(|tuple| format!(
                                    "{} AC: {}\n",
                                    tuple.1, tuple.0
                                ))
                                .collect::<String>()
                        ),
                    )
                    .expect("fail to post");
                *contest_result = BTreeMap::new();
            } else {
                quiz_guard.contest_continue(ctx, msg);
            }
        }
    }
}

