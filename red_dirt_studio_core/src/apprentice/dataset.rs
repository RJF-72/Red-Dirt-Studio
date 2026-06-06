use super::SongLog;
use std::collections::VecDeque;

pub struct SongDataset {
    songs: VecDeque<SongLog>,
    favorites: Vec<String>,  // song IDs pinned as anchors
    max_capacity: usize,
}

impl SongDataset {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            songs: VecDeque::with_capacity(max_capacity),
            favorites: Vec::new(),
            max_capacity,
        }
    }

    pub fn add_song(&mut self, song: SongLog) {
        if self.songs.len() >= self.max_capacity {
            self.songs.pop_front();
        }
        self.songs.push_back(song);
    }

    pub fn pin_favorite(&mut self, song_id: String) {
        if !self.favorites.contains(&song_id) {
            self.favorites.push(song_id);
        }
    }

    pub fn unpin_favorite(&mut self, song_id: &str) {
        self.favorites.retain(|id| id != song_id);
    }

    pub fn get_all_songs(&self) -> Vec<&SongLog> {
        self.songs.iter().collect()
    }

    pub fn get_favorite_songs(&self) -> Vec<&SongLog> {
        self.songs
            .iter()
            .filter(|s| self.favorites.contains(&s.id))
            .collect()
    }

    pub fn sample_batch(&self, batch_size: usize) -> Vec<SongLog> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();

        let mut batch: Vec<SongLog> = self.songs.iter().cloned().collect();
        batch.shuffle(&mut rng);
        batch.truncate(batch_size);
        batch
    }

    pub fn sample_with_anchors(&self, batch_size: usize) -> Vec<SongLog> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();

        let mut batch = Vec::new();

        // Prioritize favorites/anchors
        for favorite_id in &self.favorites {
            if let Some(song) = self.songs.iter().find(|s| &s.id == favorite_id) {
                batch.push(song.clone());
            }
        }

        // Fill remainder with random songs
        let mut all_songs: Vec<SongLog> = self.songs.iter().cloned().collect();
        all_songs.shuffle(&mut rng);

        for song in all_songs {
            if !batch.contains(&song) && batch.len() < batch_size {
                batch.push(song);
            }
        }

        batch
    }

    pub fn len(&self) -> usize {
        self.songs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.songs.is_empty()
    }
}
