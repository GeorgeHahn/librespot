use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use protobuf::Message;
use protocol::playlist4_external::SelectedListContent;

use crate::request::{MercuryRequest, RequestResult};

use librespot_core::{spotify_id::NamedSpotifyId, Error, Session};

use librespot_protocol as protocol;
pub use protocol::playlist_annotate3::AbuseReportState;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RootPlaylist {
    pub items: Vec<Item>,
    pub spotify: SelectedListContent,
}

#[derive(Debug, Clone)]
enum LockedItem {
    Playlist(RootPlaylistEntry),
    Group(String, Arc<Mutex<Vec<LockedItem>>>),
}

impl LockedItem {
    fn dock(&self) -> Item {
        match self {
            LockedItem::Playlist(p) => Item::Playlist(p.clone()),
            LockedItem::Group(n, l) => Item::Group(
                n.to_owned(),
                l.lock().unwrap().iter().map(|i| i.dock()).collect(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RootPlaylistEntry {
    id: NamedSpotifyId,
    timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Item {
    Playlist(RootPlaylistEntry),
    Group(String, Vec<Item>),
}

impl RootPlaylist {
    async fn request_for_user(session: &Session, username: &str) -> RequestResult {
        let uri = format!("hm://playlist/user/{}/rootlist", username);
        <Self as MercuryRequest>::request(session, &uri).await
    }

    #[allow(dead_code)]
    pub async fn get(session: &Session) -> Result<Self, Error> {
        let current_user = session.username();
        let response = Self::request_for_user(session, &current_user).await?;
        let msg = protocol::playlist4_external::SelectedListContent::parse_from_bytes(&response)?;
        trace!("Received root playlist: {:#?}", msg);

        let hier: Arc<Mutex<Vec<LockedItem>>> = Default::default();
        let mut stack: Vec<Arc<Mutex<Vec<LockedItem>>>> = vec![hier.clone()];

        for list in msg.get_contents().items.iter() {
            let uri = list.get_uri();
            let splits = uri.split(':').collect::<Vec<_>>();

            if splits[1] == "start-group" {
                let _id = splits[2];
                let name = splits[3].to_string();

                let new: Arc<Mutex<Vec<LockedItem>>> = Default::default();

                stack
                    .last()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .push(LockedItem::Group(name, new.clone()));

                stack.push(new);

                continue;
            }
            if splits[1] == "end-group" {
                let _id = splits[2];
                let _ = stack.pop();
                continue;
            }

            stack
                .last()
                .unwrap()
                .lock()
                .unwrap()
                .push(LockedItem::Playlist(RootPlaylistEntry {
                    id: NamedSpotifyId::from_uri(uri)?,
                    timestamp: list.get_attributes().get_timestamp(),
                }));
        }

        let hier: Vec<_> = hier.lock().unwrap().iter().map(|i| i.dock()).collect();

        trace!("hier: {:#?}", hier);

        Ok(Self {
            items: hier,
            spotify: msg,
        })
    }
}

impl MercuryRequest for RootPlaylist {}
