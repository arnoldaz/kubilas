use std::marker::PhantomData;

use crate::{gpu_mesh::GpuMesh, texture::Texture};

pub type TextureRegistry = Registry<Texture, TextureId>;
pub type MeshRegistry = Registry<GpuMesh, MeshId>;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct TextureId(pub usize);

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct MeshId(pub usize);

pub struct Registry<T, Id> {
    items: Vec<T>,
    item_id: PhantomData<Id>,
}

impl<T, Id> Registry<T, Id> where Id: From<usize> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            item_id: PhantomData,
        }
    }

    pub fn add(&mut self, item: T) -> Id {
        let id = self.items.len();
        self.items.push(item);

        Id::from(id)
    }

    pub fn get(&self, id: Id) -> &T where Id: Into<usize> {
        let index: usize = id.into();
        self.items.get(index)
            .expect(&format!("Registry size {:?}, tried to access {:?}", self.items.len(), index))
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn into_items(self) -> Vec<T> {
        self.items
    }
}

impl From<usize> for TextureId {
    fn from(value: usize) -> Self {
        TextureId(value)
    }
}

impl From<TextureId> for usize {
    fn from(id: TextureId) -> usize {
        id.0
    }
}

impl From<usize> for MeshId {
    fn from(value: usize) -> Self {
        MeshId(value)
    }
}

impl From<MeshId> for usize {
    fn from(id: MeshId) -> usize {
        id.0
    }
}