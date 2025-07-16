use rusqlite::{Connection, Result};
use crate::error::Error;
#[derive(Debug, Clone)]
pub struct Identity
{
    pub keyword: String,
    pub nick: String,
    pub avatar: String
}


pub struct Database{}


impl Database
{
    pub fn init() -> Result<(), Error>
    {
        let conn = Connection::open("database.db")?;

        conn.execute(
            "create table if not exists identities_table (
             id integer primary key,
             userid integer,
             keyword text not null,
             nick text not null,
             avatar text not null
             )",
            (),
        )?;

        conn.execute(
            "create table if not exists switch_table (
             id integer primary key,
             userid integer unique,
             keyword text not null
             )",
            (),
        )?;

        conn.execute(
            "create table if not exists msg_table (
             id integer primary key,
             msgid integer,
             userid integer
             )",
            (),
        )?;
        
        Ok(())
    }
    pub async fn get_switch(user_id: u64) -> Result<Option<String>, Error>
    {
        let conn = Connection::open("database.db")?;
        let mut stmt = conn.prepare("SELECT userid, keyword
                                     FROM switch_table
                                     WHERE userid=(?1);")?;

        let identities_iter = stmt.query_map(&[&user_id], |row| {
            Ok(row.get::<_, String>(1)?)
        })?;
        let res = identities_iter
            .filter_map(|result| result.ok())
           .next();
        Ok(res)
        
    }
    pub async fn delete_switch(user_id: u64) -> Result<(), Error>
    {
        let conn = Connection::open("database.db")?;

        if let Err(why) = conn.execute(
            "DELETE FROM switch_table  where userid = (?1);",
            &[&user_id],
        )
        {
            println!("Error deleting switch ({why:?})");
        }

        Ok(())
        
    }
    pub async fn set_switch(user_id: u64, keyword: &str) -> Result<(), Error>
    {
        let conn = Connection::open("database.db")?;
        
        if let Err(why) = conn.execute(
            "INSERT OR REPLACE INTO switch_table (userid, keyword)
                     values (?1, ?2)",
            (user_id, keyword),
        )
        {
            println!("Error inserting new switch ({why:?})");
        }
        Ok(())
        
    }

    pub async fn record_message(user_id: u64, msg_id: u64) -> Result<(), Error>
    {
        let conn = Connection::open("database.db")?;
        
        if let Err(why) = conn.execute(
            "INSERT INTO msg_table (msgid, userid)
                     values (?1, ?2)",
            (msg_id, user_id),
        )
        {
            println!("Error inserting new message ({why:?})");
        }
        Ok(())
        
    }
    pub async fn get_message_owner(msg_id: u64) -> Result<Option<u64>, Error>
    {
        let conn = Connection::open("database.db")?;
        let mut stmt = conn.prepare("SELECT msgid, userid FROM msg_table WHERE msgid=(?1);")?;

        Ok(stmt.query_map(&[&msg_id], |row|
                          {
                              Ok(row.get::<_, u64>(1)?)
                          })?.filter_map(|result| result.ok()).next())
            

    }


    pub async fn get_identities(user_id: u64) -> Result<Vec<Identity>, Error>
    {

        
        let conn = Connection::open("database.db")?;
        let mut stmt = conn.prepare("SELECT userid, keyword, nick, avatar FROM identities_table WHERE userid=(?1);")?;

        let identities_iter = stmt.query_map(&[&user_id], |row| {
            Ok(Identity {
                keyword: row.get::<_, String>(1)?,
                nick: row.get::<_, String>(2)?,
                avatar: row.get::<_, String>(3)?,
            })
        })?;

        Ok(identities_iter
           .filter_map(|result| result.ok())
           .collect())
    }
    pub async fn get_identity(user_id: u64, keyword: &str) -> Result<Option<Identity>, Error>
    {
        let conn = Connection::open("database.db")?;
        let mut stmt = conn.prepare("SELECT userid, keyword, nick, avatar
                                     FROM identities_table
                                     WHERE userid=(?1) AND keyword=(?2);")?;

        let identities_iter = stmt.query_map((&user_id, keyword), |row| {
            Ok(Identity {
                keyword: row.get::<_, String>(1)?,
                nick: row.get::<_, String>(2)?,
                avatar: row.get::<_, String>(3)?,
            })
        })?;
        let res = identities_iter
            .filter_map(|result| result.ok())
           .next();
        Ok(res)
    }
    pub async fn add_identity(user_id: u64, keyword: &str, nick: &str, avatar: &str) -> Result<(), Error>
    {
        let conn = Connection::open("database.db")?;
        
        if let Err(why) = conn.execute(
            "INSERT INTO identities_table (userid, keyword, nick, avatar)
                     values (?1, ?2, ?3, ?4)",
            (user_id, keyword, nick, avatar),
        )
        {
            println!("Error inserting new nick ({why:?})");
        }
        Ok(())
    }
    
    pub async fn remove_identity(user_id: u64, keyword: &str) -> Result<(), Error>
    {
        let conn = Connection::open("database.db")?;

        if let Err(why) = conn.execute(
            "DELETE FROM identities_table  where userid = (?1) AND keyword = (?2);",
            (user_id, keyword),
        )
        {
            println!("Error deleting nick ({why:?})");
        }

        Ok(())
    }
}
