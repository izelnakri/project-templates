//! A prototype ContentProvider-style service in Rust using D-Bus (zbus)
//! Designed to run inside a Flatpak sandbox with access to clipboard, contacts and Documents folder
//! Test setup reviewed and ongoing on this one

use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};
use zbus::{ObjectServer, connection, Connection, interface, proxy};
use once_cell::sync::Lazy;
use directories::UserDirs;

static CLIPBOARD: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static CONTACTS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
struct ContentProvider;

#[interface(name = "org.example.ContentProvider")]
impl ContentProvider {
    fn get_clipboard(&self) -> String {
        CLIPBOARD.lock().unwrap().clone()
    }

    fn set_clipboard(&self, text: &str) {
        *CLIPBOARD.lock().unwrap() = text.to_string();
    }

    fn add_contact(&self, name: &str, phone: &str) {
        CONTACTS.lock().unwrap().insert(name.to_string(), phone.to_string());
    }

    fn get_contact(&self, name: &str) -> zbus::fdo::Result<String> {
        CONTACTS
            .lock()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| zbus::fdo::Error::Failed(format!("No contact found for '{}'", name)))
    }

    fn list_contacts(&self) -> Vec<(String, String)> {
        CONTACTS.lock().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    fn list_documents(&self) -> Vec<String> {
        let mut docs = Vec::new();
        if let Some(user_dirs) = UserDirs::new() {
            if let Some(documents_dir) = user_dirs.document_dir() {
                if let Ok(entries) = fs::read_dir(documents_dir) {
                    for entry in entries.flatten() {
                        if let Ok(path) = entry.path().into_os_string().into_string() {
                            docs.push(path);
                        }
                    }
                }
            }
        }
        docs
    }

    fn read_document(&self, filename: &str) -> zbus::fdo::Result<String> {
        if let Some(user_dirs) = UserDirs::new() {
            if let Some(doc_dir) = user_dirs.document_dir() {
                let path = doc_dir.join(filename);
                let content = fs::read_to_string(&path).map_err(|e| {
                    zbus::fdo::Error::Failed(format!("Failed to read file {}: {}", path.display(), e))
                })?;
                return Ok(content);
            }
        }
        Err(zbus::fdo::Error::Failed("Documents directory not found".into()))
    }
}

#[proxy(
    interface = "org.example.ContentProvider",
    default_service = "org.example.ContentProvider",
    default_path = "/org/example/ContentProvider"
)]
trait ContentProvider {
    async fn get_clipboard(&self) -> Result<String, zbus::Error>;
    async fn set_clipboard(&self, text: &str) -> Result<(), zbus::Error>;

    // fn add_contact(&self, name: &str, phone: &str) {
    //     CONTACTS.lock().unwrap().insert(name.to_string(), phone.to_string());
    // }
    //
    // fn get_contact(&self, name: &str) -> zbus::fdo::Result<String> {
    //     CONTACTS
    //         .lock()
    //         .unwrap()
    //         .get(name)
    //         .cloned()
    //         .ok_or_else(|| zbus::fdo::Error::Failed(format!("No contact found for '{}'", name)))
    // }
    //
    // fn list_contacts(&self) -> Vec<(String, String)> {
    //     CONTACTS.lock().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    // }
    //
    // fn list_documents(&self) -> Vec<String> {
    //     let mut docs = Vec::new();
    //     if let Some(user_dirs) = UserDirs::new() {
    //         if let Some(documents_dir) = user_dirs.document_dir() {
    //             if let Ok(entries) = fs::read_dir(documents_dir) {
    //                 for entry in entries.flatten() {
    //                     if let Ok(path) = entry.path().into_os_string().into_string() {
    //                         docs.push(path);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     docs
    // }
    //
    // fn read_document(&self, filename: &str) -> zbus::fdo::Result<String> {
    //     if let Some(user_dirs) = UserDirs::new() {
    //         if let Some(doc_dir) = user_dirs.document_dir() {
    //             let path = doc_dir.join(filename);
    //             let content = fs::read_to_string(&path).map_err(|e| {
    //                 zbus::fdo::Error::Failed(format!("Failed to read file {}: {}", path.display(), e))
    //             })?;
    //             return Ok(content);
    //         }
    //     }
    //     Err(zbus::fdo::Error::Failed("Documents directory not found".into()))
    // }
}

#[tokio::main]
async fn main() -> zbus::Result<()> {
    let connection = connection::Builder::session()?
        .name("org.example.ContentProvider")?

        // .object_server()
        .serve_at("/org/example/ContentProvider", ContentProvider)?
        .build()
        .await?;

    println!("ContentProvider service running...");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::{connection, Proxy};
    use uuid::Uuid;
    use serial_test::serial;
    use tokio;

    async fn spawn_service() -> zbus::Result<(Connection, String)> {
        let service_name = format!("org.example.ContentProvider.instance_{}", Uuid::new_v4().simple());
        let connection = connection::Builder::session()?
            .name(service_name.clone())?
            .serve_at("/org/example/ContentProvider", ContentProvider {})? // NOTE: Remove {}
            .build()
            .await?;

        Ok((connection, service_name))
    }

    #[tokio::test]
    #[serial]
    async fn test_clipboard_roundtrip() -> zbus::Result<()> {
        let (service_conn, service_name) = spawn_service().await?;
        let proxy = Proxy::new(
            &service_conn,
            zbus::names::WellKnownName::try_from(service_name.as_str())?,
            "/org/example/ContentProvider",
            "org.example.ContentProvider",
        )
        .await?;

        proxy.call_method("SetClipboard", &("Hello, Clipboard!")).await?;

        let msg = proxy.call_method("GetClipboard", &()).await?;
        let clipboard: String = msg.body().deserialize()?;

        assert_eq!(clipboard, "Hello, Clipboard!");

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_another_clipboard_roundtrip() -> zbus::Result<()> {
        let (service_conn, service_name) = spawn_service().await?;
        let proxy = ContentProviderProxy::builder(&service_conn)
            .destination(zbus::names::WellKnownName::try_from(service_name.as_str())?)?
            .path("/org/example/ContentProvider")?
            .build()
            .await?;
        proxy.set_clipboard("Hello, another Clipboard!").await?;
        let clipboard = proxy.get_clipboard().await?;

        assert_eq!(clipboard, "Hello, another Clipboard!");

        Ok(())
    }

    // #[tokio::test]
    // async fn test_contacts_roundtrip() -> zbus::Result<()> {
    //     let service_conn = spawn_service().await?;
    //     let proxy = Proxy::new(
    //         &service_conn,
    //         "org.example.ContentProvider",
    //         "/org/example/ContentProvider",
    //         "org.example.ContentProvider",
    //     )
    //     .await?;
    //
    //     proxy.call_method("add_contact", &("Alice", "123456")).await?;
    //
    //     let msg = proxy.call_method("get_contact", &("Alice")).await?;
    //     let phone: String = msg.body().unwrap();
    //
    //     assert_eq!(phone, "123456");
    //
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn test_document_list_and_read() -> zbus::Result<()> {
    //     let service_conn = spawn_service().await?;
    //     let proxy = Proxy::new(
    //         &service_conn,
    //         "org.example.ContentProvider",
    //         "/org/example/ContentProvider",
    //         "org.example.ContentProvider",
    //     )
    //     .await?;
    //
    //     let msg = proxy.call_method("list_documents", &()).await?;
    //     let docs: Vec<String> = msg.body().unwrap();
    //
    //     if let Some(first_doc) = docs.first() {
    //         let filename = std::path::PathBuf::from(first_doc)
    //             .file_name()
    //             .unwrap()
    //             .to_string_lossy()
    //             .to_string();
    //
    //         let msg = proxy.call_method("read_document", &(filename,)).await?;
    //         let content: String = msg.body().unwrap();
    //
    //         assert!(!content.is_empty());
    //     }
    //
    //     Ok(())
    // }
}

/*
flatpak manifest (snippet):

"finish-args": [
  "--filesystem=xdg-documents",
  "--talk-name=org.freedesktop.portal.Clipboard",
  "--talk-name=org.freedesktop.portal.Contacts",
  "--own-name=org.example.ContentProvider"
]
*/
