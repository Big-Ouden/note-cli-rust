/*!
 * Project: note-cli
 * Description: A minimal command-line note manager in Rust.
 * Author: BigOuden
 * GitHub: https://github.com/Big-Ouden/note-cli
 *
 * Features:
 *  - Add, remove, and edit notes
 *  - Add tags to notes
 *  - List notes with sorting options
 *  - Search notes by keyword
 *  - Reuse IDs of deleted notes
 *
 * Notes:
 *  - Written for learning Rust and CLI application development
 *  - Uses serde for JSON serialization and prettytable for display
 */

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum, error::ErrorKind};
use prettytable::{Table, cell, row};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

const NOTES_PATH: &str = "notes.json";

type NoteResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(name = "note-cli")]
#[command(about="Minimal note manager in Rust", long_about=None)]
struct Cli {
    #[arg(long, default_value = "notes.json")]
    file: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // Add new note
    Add {
        // Note content
        content: String,

        // Associated tags (repeatable)
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    // List all notes
    List {
        #[arg(long = "sort", value_enum, default_value = "id")]
        method: SortMethod,
    },
    // Remove a Note
    Remove {
        // Note to remove id
        id: u32,
    },

    // Add a tag to an existing note
    AddTag {
        // note id
        id: u32,

        // tag to add
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    // Edit the content of an
    Edit {
        // note id
        id: u32,

        // content
        #[arg(long = "content")]
        content: String,
    },

    Search {
        keyword: String,
        #[arg(long = "sort", value_enum, default_value = "id")]
        method: SortMethod,
    },
}

#[derive(Clone, ValueEnum)]
enum SortMethod {
    Id,
    Date,
    Update,
    Content,
}

// Struct for a single note
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Note {
    id: u32,
    content: String,
    tags: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// Struct of json file
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct NoteData {
    notes: Vec<Note>,
    free_ids: Vec<u32>, // to give ids to new note and keep clear
}

/// Cleans all notes by writing an empty JSON array to the file.
///
/// # Parameters
/// - `path: &str` - File path to clean
///
/// # Returns
/// `std::io::Result<()>` - Success or I/O error
fn clean_notes(path: &str) -> std::io::Result<()> {
    fs::write(path, "[]")
}

/// Loads note data from a JSON file, returns empty data if file doesn't exist.
///
/// # Parameters
/// - `path: &str` - File path to read from
///
/// # Returns
/// `NoteResult<NoteData>` - Loaded note data or error
fn load_notes(path: &str) -> NoteResult<NoteData> {
    use std::io::ErrorKind;

    match fs::read_to_string(path) {
        Ok(content) => {
            if content.trim().is_empty() {
                Ok(NoteData {
                    notes: vec![],
                    free_ids: vec![],
                }) // empty file = no notes
            } else {
                Ok(serde_json::from_str(&content)?)
            }
        }

        Err(e) if e.kind() == ErrorKind::NotFound => Ok(NoteData {
            notes: vec![],
            free_ids: vec![],
        }),

        Err(e) => Err(Box::new(e)),
    }
}

/// Saves note data to a JSON file with pretty formatting.
///
/// # Parameters
/// - `path: &str` - File path to write to
/// - `notes: &NoteData` - Note data to serialize
///
/// # Returns
/// `NoteResult<()>` - Success or serialization/I/O error
fn save_notes(path: &str, notes: &NoteData) -> NoteResult<()> {
    // serialize to a json
    let data = serde_json::to_string_pretty(notes)?;

    //write on filesystem
    fs::write(path, data)?;
    Ok(())
}

/// Adds a new note with auto-assigned ID (reuses free IDs when available).
///
/// # Parameters
/// - `path: &str` - File path where notes are stored
/// - `content: String` - Text content for the new note
///
/// # Returns
/// `NoteResult<()>` - Success or error during load/save operations
fn add_note(path: &str, content: String, tags: Vec<String>) -> NoteResult<()> {
    let mut data = load_notes(path)?;

    // determine id
    let mut new_id: u32 = match data.notes.last() {
        Some(note) => note.id + 1,
        None => 1,
    }; // last known id

    // if free id not empty take one from it
    if !data.free_ids.is_empty() {
        new_id = data.free_ids.remove(0);
    }

    // create new note
    let new_note = Note {
        id: new_id,
        content: content,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tags: tags,
    };

    // push new note into data
    data.notes.push(new_note);
    save_notes(path, &data)?;
    Ok(())
}

/// Removes a note by ID and adds the ID to the free list for reuse.
///
/// # Parameters
/// - `path: &str` - File path where notes are stored
/// - `id: u32` - ID of the note to remove
///
/// # Returns
/// `NoteResult<()>` - Success or error if ID not found or I/O fails
fn remove_note(path: &str, id: u32) -> NoteResult<()> {
    // load data
    let mut data = load_notes(path)?;

    // get index of note to remove
    let index = data
        .notes
        .iter()
        .position(|note| note.id == id)
        .ok_or_else(|| format!("ID {} not found", id))?;

    // remove note
    data.notes.swap_remove(index);
    // push its id into free_ids
    data.free_ids.push(id);
    save_notes(path, &data)?;
    Ok(())
}

/// Displays all notes in a formatted table or shows empty message.
///
/// # Parameters
/// - `path: &str` - File path where notes are stored
///
/// # Returns
/// `NoteResult<()>` - Success or error during load operation
fn list_note(path: &str, method: SortMethod) -> NoteResult<()> {
    let mut data = load_notes(path)?;

    if data.notes.is_empty() {
        println!("📭 No notes saved.");
    } else {
        // Sort notes by ID
        match method {
            SortMethod::Id => data.notes.sort_by_key(|note| note.id),
            SortMethod::Date => data.notes.sort_by_key(|note| note.created_at),
            SortMethod::Update => data.notes.sort_by_key(|note| note.updated_at),
            SortMethod::Content => data.notes.sort_by_key(|note| note.content.clone()),
            _ => (),
        }

        let mut table = Table::new();
        table.add_row(row!["ID", "Content", "Tags", "Created at", "Update at"]);

        for note in data.notes {
            let tag_str = if note.tags.is_empty() {
                "-".to_string()
            } else {
                note.tags.join(", ")
            };
            table.add_row(row![
                note.id,
                note.content,
                tag_str,
                note.created_at.format("%d/%m/%Y - %H:%M").to_string(),
                note.updated_at.format("%d/%m/%Y - %H:%M").to_string()
            ]);
        }

        table.printstd();
    }
    Ok(())
}

/// Add a tag to a note
///
/// # Parameters
/// - `path: &str` - File path where notes are stored
/// - `id: u32` - Id of the note we want to add a tag
/// - `tag: String` - Tag to add to the note specified by id
///
/// # Returns
/// `NoteResult<()>` - Success or error during load operation
fn add_tag(path: &str, id: u32, tags: Vec<String>) -> NoteResult<()> {
    let mut data = load_notes(path)?;

    if tags.is_empty() {
        println!("No tags given.");
    } else if let Some(note) = data.notes.iter_mut().find(|n| n.id == id) {
        // add tags
        for tag in tags {
            if !note.tags.contains(&tag) {
                note.tags.push(tag);
            }
        }
        save_notes(path, &data)?;
    } else {
        println!("Note {} not found", id);
    }

    Ok(())
}

/// Edit a note
///
/// # Parameters
/// - `path: &str` - File path where notes are stored
/// - `id: u32` - Id of the note to edit
/// - `content: String` - New content
///
/// # Returns
/// `NoteResult<()>` - Success or error during load operation
fn edit_note(path: &str, id: u32, content: String) -> NoteResult<()> {
    let mut data = load_notes(path)?;

    if content.is_empty() {
        println!("No content given.");
    } else if let Some(note) = data.notes.iter_mut().find(|n| n.id == id) {
        note.content = content;
        note.updated_at = Utc::now();
        save_notes(path, &data)?;
    } else {
        println!("Note {} not found", id);
    }

    Ok(())
}

/// Search field in all notes
///
/// # Parameters
/// - `path: &str` - File path where notes are stored
/// - `content: String` - Field to search
///
/// # Returns
/// `NoteResult<()>` - Success or error during load operation
fn search_note(path: &str, keyword: String, method: SortMethod) -> NoteResult<()> {
    let mut data = load_notes(path)?;

    if keyword.is_empty() {
        print!("No keyword given");
    } else {
        let mut results: Vec<&Note> = data
            .notes
            .iter()
            .filter(|n| n.content.to_lowercase().contains(&keyword.to_lowercase()))
            .collect();

        // Sort notes by ID
        match method {
            SortMethod::Id => results.sort_by_key(|note| note.id),
            SortMethod::Date => results.sort_by_key(|note| note.created_at),
            SortMethod::Update => results.sort_by_key(|note| note.updated_at),
            SortMethod::Content => results.sort_by_key(|note| note.content.clone()),
            _ => (),
        }

        let mut table = Table::new();
        table.add_row(row!["ID", "Content", "Tags", "Created at", "Update at"]);
        for note in results {
            let tag_str = if note.tags.is_empty() {
                "-".to_string()
            } else {
                note.tags.join(", ")
            };
            table.add_row(row![
                note.id,
                note.content,
                tag_str,
                note.created_at.format("%d/%m/%Y - %H:%M").to_string(),
                note.updated_at.format("%d/%m/%Y - %H:%M").to_string()
            ]);
        }
        table.printstd();
    }
    Ok(())
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // Helper to verify if a datetime is in the past or present
    fn assert_time_valid(dt: &DateTime<Utc>) {
        let now = Utc::now();
        assert!(dt <= &now, "datetime should be in the past or present");
    }

    #[test]
    fn test_add_note() -> NoteResult<()> {
        let tmpfile = NamedTempFile::new()?;
        let path = tmpfile.path().to_str().unwrap();

        add_note(path, "content0".to_string(), vec![])?;
        add_note(path, "content1".to_string(), vec![])?;

        let data = load_notes(path)?;

        assert_eq!(data.notes.len(), 2);

        let note1 = &data.notes[0];
        assert_eq!(note1.id, 1);
        assert_eq!(note1.content, "content0");
        assert!(note1.tags.is_empty());
        assert_time_valid(&note1.created_at);
        assert_time_valid(&note1.updated_at);

        let note2 = &data.notes[1];
        assert_eq!(note2.id, 2);
        assert_eq!(note2.content, "content1");
        assert!(note2.tags.is_empty());
        assert_time_valid(&note2.created_at);
        assert_time_valid(&note2.updated_at);

        Ok(())
    }

    #[test]
    fn test_remove_note() -> NoteResult<()> {
        let tmpfile = NamedTempFile::new()?;
        let path = tmpfile.path().to_str().unwrap();

        add_note(path, "content0".to_string(), vec![])?;
        add_note(path, "content1".to_string(), vec![])?;

        remove_note(path, 2)?;

        let data = load_notes(path)?;
        assert_eq!(data.notes.len(), 1);
        let note1 = &data.notes[0];
        assert_eq!(note1.id, 1);
        assert_eq!(note1.content, "content0");
        assert!(note1.tags.is_empty());

        // free_ids contient l'ID libéré
        assert_eq!(data.free_ids, vec![2]);

        Ok(())
    }

    #[test]
    fn test_id_allocation() -> NoteResult<()> {
        let tmpfile = NamedTempFile::new()?;
        let path = tmpfile.path().to_str().unwrap();

        add_note(path, "content1".to_string(), vec![])?;
        add_note(path, "content2".to_string(), vec![])?;

        remove_note(path, 1)?;
        remove_note(path, 2)?;

        let mut data = load_notes(path)?;
        assert_eq!(data.free_ids, vec![1, 2]);

        // Ajouter de nouvelles notes doit réutiliser les IDs
        add_note(path, "new1".to_string(), vec![])?;
        add_note(path, "new2".to_string(), vec![])?;
        data = load_notes(path)?;
        assert_eq!(data.free_ids.len(), 0);
        assert_eq!(
            data.notes.iter().map(|n| n.id).collect::<Vec<_>>(),
            vec![1, 2]
        );

        Ok(())
    }

    #[test]
    fn test_add_note_with_tag() -> NoteResult<()> {
        let tmpfile = NamedTempFile::new()?;
        let path = tmpfile.path().to_str().unwrap();

        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        add_note(path, "hello world".to_string(), tags.clone())?;

        let data = load_notes(path)?;
        let note = &data.notes[0];
        assert_eq!(note.tags, tags);
        assert_time_valid(&note.created_at);
        assert_time_valid(&note.updated_at);

        Ok(())
    }

    #[test]
    fn test_add_tag_existing_note() -> NoteResult<()> {
        let tmpfile = NamedTempFile::new()?;
        let path = tmpfile.path().to_str().unwrap();

        add_note(path, "note".to_string(), vec![])?;
        add_tag(path, 1, vec!["rust".to_string(), "cli".to_string()])?;

        let data = load_notes(path)?;
        let note = &data.notes[0];
        assert_eq!(note.tags, vec!["rust", "cli"]);

        // Ajouter un tag déjà existant ne doit pas créer de doublon
        add_tag(path, 1, vec!["rust".to_string()])?;
        let data = load_notes(path)?;
        let note = &data.notes[0];
        assert_eq!(note.tags, vec!["rust", "cli"]);

        Ok(())
    }

    #[test]
    fn test_edit_note() -> NoteResult<()> {
        let tmpfile = NamedTempFile::new()?;
        let path = tmpfile.path().to_str().unwrap();

        add_note(path, "old content".to_string(), vec![])?;
        edit_note(path, 1, "new content".to_string())?;

        let data = load_notes(path)?;
        let note = &data.notes[0];
        assert_eq!(note.content, "new content");
        assert_time_valid(&note.updated_at);

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { content, tags } => {
            add_note(&cli.file, content, tags)?;
        }
        Commands::List { method } => {
            list_note(&cli.file, method)?;
        }
        Commands::Remove { id } => {
            remove_note(&cli.file, id)?;
        }
        Commands::AddTag { id, tags } => {
            add_tag(&cli.file, id, tags)?;
        }
        Commands::Edit { id, content } => {
            edit_note(&cli.file, id, content)?;
        }
        Commands::Search { keyword, method } => {
            search_note(&cli.file, keyword, method)?;
        }
    }
    Ok(())
}
