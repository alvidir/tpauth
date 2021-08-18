use std::error::Error;
use std::time::{SystemTime};
use crate::constants::errors::ALREADY_EXISTS;

pub trait MetadataRepository {
    fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>>;
    fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>>;
    fn save(&self, meta: &Metadata) -> Result<(), Box<dyn Error>>;
    fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone)]
pub struct Metadata {
    pub(super) id: i32,
    pub(super) created_at: SystemTime,
    pub(super) updated_at: SystemTime,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            id: 0,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn touch(&mut self) {
        self.updated_at = SystemTime::now();
    }

    /// inserts the metadata into the repository
    pub fn insert(&mut self) -> Result<(), Box<dyn Error>> {
        if self.id != 0 {
            return Err(ALREADY_EXISTS.into());
        }

        super::get_repository().create(self)?;
        Ok(())
    }

    /// updates the metadata into the repository
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        super::get_repository().save(self)
    }

    /// deletes the metadata from the repository
    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        super::get_repository().delete(self)
    }
}

pub struct InnerMetadata {
    pub created_at: SystemTime,
    pub touch_at: SystemTime,
}

impl InnerMetadata {
    pub fn new() -> Self {
        InnerMetadata {
            created_at: SystemTime::now(),
            touch_at: SystemTime::now(),
        }
    }

    pub fn touch(&mut self) {
        self.touch_at = SystemTime::now();
    }
}


#[cfg(test)]
pub mod tests {
    use std::time::SystemTime;
    use super::{InnerMetadata, Metadata};

    pub fn new_metadata() -> Metadata {
        Metadata{
            id: 999,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    #[test]
    fn metadata_new_should_success() {
        let before = SystemTime::now();
        let meta = Metadata::new();
        let after = SystemTime::now();

        assert_eq!(meta.id, 0);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }

    #[test]
    fn inner_metadata_new_should_success() {        
        let before = SystemTime::now();
        let meta = InnerMetadata::new();
        let after = SystemTime::now();

        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.touch_at >= before && meta.touch_at <= after);
    }
}