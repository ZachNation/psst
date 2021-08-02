use std::{sync::Arc, time::Duration};

use druid::{im::Vector, Data, Lens};
use serde::Deserialize;

use super::{ArtistLink, Promise, Track, TrackId};

#[derive(Clone, Data, Lens)]
pub struct Recommend {
    pub counter: u64,
    pub request: Option<RecommendationsRequest>,
    pub results: Promise<Recommendations, u64>,
}

#[derive(Clone, Debug, Default, Data, PartialEq)]
pub struct RecommendationsRequest {
    pub seed_artists: Vector<ArtistLink>,
    pub seed_tracks: Vector<TrackId>,

    pub duration: Range<Duration>,
    pub popularity: Range<u32>,
    pub key: Range<u32>,
    pub mode: Range<u32>,
    pub tempo: Range<u32>,
    pub time_signature: Range<u32>,

    pub acousticness: Range<f64>,
    pub danceability: Range<f64>,
    pub energy: Range<f64>,
    pub instrumentalness: Range<f64>,
    pub liveness: Range<f64>,
    pub loudness: Range<f64>,
    pub speechiness: Range<f64>,
    pub valence: Range<f64>,
}

#[derive(Clone, Copy, Debug, Default, Data, PartialEq, Eq, Hash)]
pub struct Range<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub target: Option<T>,
}

#[derive(Clone, Data, Deserialize, Lens)]
pub struct Recommendations {
    pub seeds: Vector<RecommendationsSeed>,
    pub tracks: Vector<Arc<Track>>,
}

#[derive(Clone, Data, Deserialize, Lens)]
pub struct RecommendationsSeed {
    #[serde(default)]
    pub after_filtering_size: usize,
    #[serde(default)]
    pub after_relinking_size: usize,
    pub href: Option<Arc<str>>,
    pub id: Arc<str>,
    #[serde(default)]
    pub initial_pool_size: usize,
    #[serde(rename = "type")]
    pub _type: RecommendationsSeedType,
}

#[derive(Clone, Data, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecommendationsSeedType {
    Artist,
    Track,
    Genre,
}
