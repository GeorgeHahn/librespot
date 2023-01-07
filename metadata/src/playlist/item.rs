use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::util::{impl_deref_wrapped, impl_try_from_repeated};

use super::{
    attribute::{PlaylistAttributes, PlaylistItemAttributes},
    permission::Capabilities,
};

use librespot_core::{date::Date, SpotifyId};

use librespot_protocol as protocol;
use protocol::playlist4_external::Item as PlaylistItemMessage;
use protocol::playlist4_external::ListItems as PlaylistItemsMessage;
use protocol::playlist4_external::MetaItem as PlaylistMetaItemMessage;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PlaylistItem {
    pub id: SpotifyId,
    pub attributes: PlaylistItemAttributes,
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct PlaylistItems(pub Vec<PlaylistItem>);

impl_deref_wrapped!(PlaylistItems, Vec<PlaylistItem>);

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PlaylistItemList {
    pub position: i32,
    pub is_truncated: bool,
    pub items: PlaylistItems,
    pub meta_items: PlaylistMetaItems,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PlaylistMetaItem {
    pub revision: SpotifyId,
    pub attributes: PlaylistAttributes,
    pub length: i32,
    pub timestamp: Date,
    pub owner_username: String,
    pub has_abuse_reporting: bool,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct PlaylistMetaItems(pub Vec<PlaylistMetaItem>);

impl_deref_wrapped!(PlaylistMetaItems, Vec<PlaylistMetaItem>);

impl TryFrom<&PlaylistItemMessage> for PlaylistItem {
    type Error = librespot_core::Error;
    fn try_from(item: &PlaylistItemMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            id: item.try_into()?,
            attributes: item.get_attributes().try_into()?,
        })
    }
}

impl_try_from_repeated!(PlaylistItemMessage, PlaylistItems);

impl TryFrom<&PlaylistItemsMessage> for PlaylistItemList {
    type Error = librespot_core::Error;
    fn try_from(list_items: &PlaylistItemsMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            position: list_items.get_pos(),
            is_truncated: list_items.get_truncated(),
            items: PlaylistItems(
                list_items
                    .get_items()
                    .iter()
                    .filter_map(|i| i.try_into().ok())
                    .collect(),
            ),
            meta_items: list_items.get_meta_items().try_into()?,
        })
    }
}

impl TryFrom<&PlaylistMetaItemMessage> for PlaylistMetaItem {
    type Error = librespot_core::Error;
    fn try_from(item: &PlaylistMetaItemMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            revision: item.try_into()?,
            attributes: item.get_attributes().try_into()?,
            length: item.get_length(),
            timestamp: Date::from_timestamp_ms(item.get_timestamp())?,
            owner_username: item.get_owner_username().to_owned(),
            has_abuse_reporting: item.get_abuse_reporting_enabled(),
            capabilities: item.get_capabilities().into(),
        })
    }
}

impl_try_from_repeated!(PlaylistMetaItemMessage, PlaylistMetaItems);
