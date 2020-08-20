extern crate rusqlite;
use fancy_regex::Regex;
use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};
use tbot::prelude::*;

struct Todo {
    name: String,
}

#[tokio::main]
async fn main() {
    let mut bot = tbot::from_env!("BOT_TOKEN").event_loop();
    bot.command("list", |context| async move {
        let conn = createDatabase();
        let foo = match getActiveTodos(&conn, context.chat.id.to_string()) {
            Ok(n) => n,
            Err(e) => "ERROR".to_owned(),
        };

        let mut message: String = String::from(format!("Hash {} saved", 214));
        let call_result = context.send_message(&foo).call().await;

        if let Err(err) = call_result {
            dbg!(err);
        }
    });

    bot.command("fin", |context| async move {
        let conn = createDatabase();

        let hashtag = getHashtagFromMessage(format!("{}", context.text.value));
        if hashtag == "" {
            return;
        }
        let conn = createDatabase();
        saveHashtag(conn, &hashtag, context.chat.id.to_string(), false);

        let mut message: String = String::from(format!("Hash {} saved", 214));
        let call_result = context.send_message("Hashtag archived").call().await;

        if let Err(err) = call_result {
            dbg!(err);
        }
    });
    bot.text(|context| async move {
        // println!("{} me this: {}", context.chat.id, context.text.value);
        let hashtag = getHashtagFromMessage(format!("{}", context.text.value));
        if hashtag == "" {
            return;
        }
        let conn = createDatabase();
        saveHashtag(conn, &hashtag, context.chat.id.to_string(), true);
        // let mut message: String = String::from(format!("Hash {} saved", hashtag));
        // let call_result = context.send_message(&message).call().await;

        // if let Err(err) = call_result {
        //     dbg!(err);
        // }
    });

    bot.polling().start().await.unwrap();
}

fn getHashtagFromMessage(message: String) -> String {
    let re = Regex::new(r"\B(\#[a-zA-Z0-9]+\b)(?!;)").unwrap();

    let tag = re.captures(&message).expect("Error running regex");

    match tag {
        Some(n) => return format!("{}",n.get(0).expect("No group").as_str()),
        None  => return "".to_owned(),
    }
}

pub fn saveHashtag(conn: Connection, hashtag: &String, chatId: String, active: bool) -> bool {
    conn.execute(
        "INSERT OR REPLACE INTO todos (
            name,
            chatId,
            active
        ) VALUES (?,?,?);",
        params![hashtag, chatId, active],
    )
    .unwrap();

    return true;
}

fn createDatabase() -> Connection {
    let conn = Connection::open("/root/todos.sql").unwrap();

    let res = conn.execute(
        "create table if not exists todos (
             name string primary key,
             chatId int not null,
             active boolean default 1
         )",
        NO_PARAMS,
    );

    match res {
        Ok(n) => return conn,
        Err(e) => println!("SQLITE ERROR:{}", e),
    }

    return conn;
}

fn getActiveTodos(conn: &Connection, chatId: String) -> Result<String> {
    let mut stmt = conn.prepare("SELECT name FROM todos WHERE active = 1 AND chatId = ?")?;
    let todo_iter = stmt.query_map(params![chatId], |row| Ok(Todo { name: row.get(0)? }))?;
    let mut foo = "Current active hashtags:".to_owned();
    for todo in todo_iter {
        foo.push_str("\n");

        match todo {
            Ok(n) => foo.push_str(&n.name),
            Err(e) => return Err(e),
        }
    }

    Ok(foo)
}
