use std::{fmt, sync::Arc, time::Duration};

use druid::{im::Vector, lens::Map, Data, Lens};
use psst_core::item_id::ItemId;
use serde::{Deserialize, Serialize};

use super::{
    AlbumLink, ArtistLink, Episode, Library, Nav, PlaylistLink, RecommendationsRequest, ShowLink,
    Track,
};

#[derive(Clone, Data, Lens)]
pub struct Playback {
    pub state: PlaybackState,
    pub now_playing: Option<NowPlaying>,
    pub queue_behavior: QueueBehavior,
    pub queue: Vector<QueueEntry>,
    pub volume: f64,
}

#[derive(Clone, Debug, Data, Lens)]
pub struct QueueEntry {
    pub item: PlaybackItem,
    pub origin: PlaybackOrigin,
}

#[derive(Clone, Debug, Data)]
pub enum PlaybackItem {
    Track(Arc<Track>),
    Episode(Arc<Episode>),
}

impl PlaybackItem {
    pub fn lens_track() -> impl Lens<Self, Option<Arc<Track>>> {
        Map::new(
            |item: &Self| item.track().cloned(),
            |_, _| {
                // Immutable.
            },
        )
    }

    pub fn lens_episode() -> impl Lens<Self, Option<Arc<Episode>>> {
        Map::new(
            |item: &Self| item.episode().cloned(),
            |_, _| {
                // Immutable.
            },
        )
    }

    pub fn track(&self) -> Option<&Arc<Track>> {
        if let Self::Track(track) = self {
            Some(track)
        } else {
            None
        }
    }

    pub fn episode(&self) -> Option<&Arc<Episode>> {
        if let Self::Episode(episode) = self {
            Some(episode)
        } else {
            None
        }
    }

    pub fn id(&self) -> ItemId {
        match self {
            PlaybackItem::Track(track) => track.id.0,
            PlaybackItem::Episode(episode) => episode.id.0,
        }
    }

    pub fn name(&self) -> &Arc<str> {
        match self {
            PlaybackItem::Track(track) => &track.name,
            PlaybackItem::Episode(episode) => &episode.name,
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            PlaybackItem::Track(track) => track.duration,
            PlaybackItem::Episode(episode) => episode.duration,
        }
    }
}

#[derive(Copy, Clone, Debug, Data, Eq, PartialEq, Serialize, Deserialize)]
pub enum QueueBehavior {
    Sequential,
    Random,
    LoopTrack,
    LoopAll,
}

impl Default for QueueBehavior {
    fn default() -> Self {
        QueueBehavior::Sequential
    }
}

#[derive(Copy, Clone, Debug, Data, Eq, PartialEq)]
pub enum PlaybackState {
    Loading,
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Data, Lens)]
pub struct NowPlaying {
    pub item: PlaybackItem,
    pub origin: PlaybackOrigin,
    pub progress: Duration,

    // Although keeping a ref to the `Library` here is a bit of a hack, it dramatically
    // simplifies displaying the track context menu in the playback bar.
    pub library: Arc<Library>,
}

impl NowPlaying {
    pub fn cover_image_url(&self, width: f64, height: f64) -> Option<&str> {
        match &self.item {
            PlaybackItem::Track(track) => {
                let album = track.album.as_ref().or(match &self.origin {
                    PlaybackOrigin::Album(album) => Some(album),
                    _ => None,
                })?;
                Some(&album.image(width, height)?.url)
            }
            PlaybackItem::Episode(episode) => Some(&episode.image(width, height)?.url),
        }
    }
}

#[derive(Clone, Debug, Data)]
pub enum PlaybackOrigin {
    Library,
    Album(AlbumLink),
    Artist(ArtistLink),
    Playlist(PlaylistLink),
    Show(ShowLink),
    Search(Arc<str>),
    Recommendations(Arc<RecommendationsRequest>),
}

impl PlaybackOrigin {
    pub fn to_nav(&self) -> Nav {
        match &self {
            PlaybackOrigin::Library => Nav::SavedTracks,
            PlaybackOrigin::Album(link) => Nav::AlbumDetail(link.clone()),
            PlaybackOrigin::Artist(link) => Nav::ArtistDetail(link.clone()),
            PlaybackOrigin::Playlist(link) => Nav::PlaylistDetail(link.clone()),
            PlaybackOrigin::Show(link) => Nav::ShowDetail(link.clone()),
            PlaybackOrigin::Search(query) => Nav::SearchResults(query.clone()),
            PlaybackOrigin::Recommendations(request) => Nav::Recommendations(request.clone()),
        }
    }
}

impl fmt::Display for PlaybackOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            PlaybackOrigin::Library => f.write_str("Saved Tracks"),
            PlaybackOrigin::Album(link) => link.name.fmt(f),
            PlaybackOrigin::Artist(link) => link.name.fmt(f),
            PlaybackOrigin::Playlist(link) => link.name.fmt(f),
            PlaybackOrigin::Show(link) => link.name.fmt(f),
            PlaybackOrigin::Search(query) => query.fmt(f),
            PlaybackOrigin::Recommendations(_) => f.write_str("Recommended"),
        }
    }
}

#[derive(Clone, Debug, Data)]
pub struct PlaybackPayload {
    pub origin: PlaybackOrigin,
    pub items: Vector<PlaybackItem>,
    pub position: usize,
}
