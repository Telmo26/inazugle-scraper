use std::{fs::File, sync::{Arc, RwLock}};

use rusqlite::{params, Connection};

use crate::utils::{Character, Stats};

pub struct Database {
    conn: Arc<RwLock<Connection>>,
}

impl Database {
    pub fn connect(path: &str) -> Database {
        // Ensure file exists
        if !std::path::Path::new(path).exists() {
            File::create(path).expect("Unable to create database file");
        }

        let conn = Connection::open(path).expect("Unable to open database");

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS characters (
                id INTEGER PRIMARY KEY,
                name TEXT,
                nickname TEXT,
                element TEXT,
                position TEXT,
                kick INTEGER,
                control INTEGER,
                technique INTEGER,
                pressure INTEGER,
                physical INTEGER,
                agility INTEGER,
                intelligence INTEGER
            )
            "#,
            [],
        )
        .expect("Failed to create table");

        Database {
            conn: Arc::new(RwLock::new(conn)),
        }
    }

    pub fn populate_character_data(&self, character: &mut Character) -> bool {
        let read_lock = self.conn.read().unwrap();
        let mut stmt = match read_lock.prepare(
            r#"
            SELECT
                kick,
                control,
                technique,
                pressure,
                physical,
                agility,
                intelligence
            FROM characters
            WHERE id = ?
            "#,
        ) {
            Ok(stmt) => stmt,
            Err(_) => return false,
        };

        let result = stmt.query_row(
            params![character.number],
            |row| {
                Ok(Stats {
                    kick: row.get("kick")?,
                    control: row.get("control")?,
                    technique: row.get("technique")?,
                    pressure: row.get("pressure")?,
                    physical: row.get("physical")?,
                    agility: row.get("agility")?,
                    intelligence: row.get("intelligence")?,
                })
            },
        );

        match result {
            Ok(stats) => {
                character.stats = Some(stats);
                true
            }
            Err(_) => false,
        }
    }

    pub fn store_character(&self, character: &Character) {
        let write_lock = self.conn.write().unwrap();
        let stats = character.stats.as_ref().expect("Character has no stats");

        write_lock
            .execute(
                r#"
                INSERT INTO characters (
                    id,
                    name,
                    nickname,
                    element,
                    position,
                    kick,
                    control,
                    technique,
                    pressure,
                    physical,
                    agility,
                    intelligence
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    character.number,
                    character.name,
                    character.nickname,
                    character.element.db_str(),
                    character.position.to_str(),
                    stats.kick,
                    stats.control,
                    stats.technique,
                    stats.pressure,
                    stats.physical,
                    stats.agility,
                    stats.intelligence
                ],
            )
            .expect("Failed to insert character");
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            conn: self.conn.clone(),
        }
    }
}

/// SAFETY: The Database can only be cloned by duplicating the Arc
/// reference, which is in itself Send. Then, the RwLock ensures
/// no concurrent write operation on the database.
unsafe impl Send for Database {}
