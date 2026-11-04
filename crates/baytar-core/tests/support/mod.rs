use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use rusqlite::{params, types::Value, Connection};

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "baytar-legacy-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("create temporary test directory");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Default)]
pub struct LegacySpec {
    pub owners: Vec<Owner>,
    pub patients: Vec<Patient>,
    pub todos: Vec<Todo>,
}

pub struct Owner {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<Value>,
}

pub struct Patient {
    pub id: Option<i64>,
    pub owner_id: Option<i64>,
    pub name: Option<String>,
    pub patient_type: Option<String>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub registration_date: Option<String>,
    pub note: Option<String>,
}

pub struct Todo {
    pub id: Option<i64>,
    pub owner_id: Option<i64>,
    pub patient_id: Option<i64>,
    pub name: Option<String>,
    pub date: Option<String>,
    pub note: Option<String>,
}

pub fn legacy_db(dir: &Path, spec: &LegacySpec) -> PathBuf {
    let path = dir.join("synthetic-legacy.db");
    let connection = Connection::open(&path).expect("create synthetic fixture");
    connection
        .execute_batch("PRAGMA foreign_keys = OFF;")
        .expect("allow documented legacy orphan rows");
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS PatientOwner(\n\
          \"Id\" INTEGER, \"OwnerName\" TEXT, \"OwnerAdress\" TEXT, \"OwnerPhone\" INTEGER,\n\
          PRIMARY KEY(\"Id\"));\n\
        CREATE TABLE IF NOT EXISTS Patient(\n\
          \"Id\" INTEGER, \"OwnerId\" INTEGER, \"PatientName\" TEXT, \"PatientType\" TEXT, \"PatientGender\" TEXT,\n\
          \"BirthDate\" TEXT, \"RegistrationDate\" TEXT, \"PatientNote\" TEXT,\n\
          PRIMARY KEY(\"Id\"), FOREIGN KEY(\"OwnerId\") REFERENCES \"PatientOwner\"(\"Id\"));\n\
        CREATE TABLE IF NOT EXISTS ToDo(\n\
          \"Id\" INTEGER, \"OwnerId\" INTEGER, \"PatientId\" INTEGER, \"ToDoName\" TEXT, \"ToDoDate\" DATETIME, \"ToDoNote\" TEXT,\n\
          PRIMARY KEY(\"Id\"),\n\
          FOREIGN KEY(\"OwnerId\") REFERENCES \"PatientOwner\"(\"Id\"),\n\
          FOREIGN KEY(\"PatientId\") REFERENCES \"Patient\"(\"Id\"));",
    ).expect("create documented legacy schema");
    for owner in &spec.owners {
        connection.execute("INSERT INTO PatientOwner(Id, OwnerName, OwnerAdress, OwnerPhone) VALUES (?1, ?2, ?3, ?4)",
            params![owner.id, owner.name, owner.address, owner.phone])
            .expect("insert synthetic owner");
    }
    for patient in &spec.patients {
        connection.execute("INSERT INTO Patient(Id, OwnerId, PatientName, PatientType, PatientGender, BirthDate, RegistrationDate, PatientNote) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![patient.id, patient.owner_id, patient.name, patient.patient_type, patient.gender, patient.birth_date, patient.registration_date, patient.note])
            .expect("insert synthetic patient");
    }
    for todo in &spec.todos {
        connection.execute("INSERT INTO ToDo(Id, OwnerId, PatientId, ToDoName, ToDoDate, ToDoNote) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![todo.id, todo.owner_id, todo.patient_id, todo.name, todo.date, todo.note])
            .expect("insert synthetic to-do");
    }
    drop(connection);
    path
}
