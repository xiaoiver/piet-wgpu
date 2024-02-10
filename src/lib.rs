//! The wgpu backend for the Piet 2D graphics abstraction.

pub use ahash::{AHasher, RandomState};
pub use hashbrown;
use hashbrown::hash_map::RawEntryMut;
use std::{
    fmt::Debug,
    hash::{BuildHasher, BuildHasherDefault, Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

use context::{WgpuImage, WgpuRenderContext};
use text::{WgpuText, WgpuTextLayout, WgpuTextLayoutBuilder};

mod context;
mod mesh;
// mod render_graph;
mod render_resource;
pub mod renderer;
mod text;

/// The `RenderContext` for the CoreGraphics backend, which is selected.
pub type Piet<'a> = WgpuRenderContext<'a>;

/// The associated brush type for this backend.
///
/// This type matches `RenderContext::Brush`
pub type Brush = context::Brush;

/// The associated text factory for this backend.
///
/// This type matches `RenderContext::Text`
pub type PietText = WgpuText;

/// The associated text layout type for this backend.
///
/// This type matches `RenderContext::Text::TextLayout`
pub type PietTextLayout = WgpuTextLayout;

/// The associated text layout builder for this backend.
///
/// This type matches `RenderContext::Text::TextLayoutBuilder`
pub type PietTextLayoutBuilder = WgpuTextLayoutBuilder;

/// The associated image type for this backend.
///
/// This type matches `RenderContext::Image`
pub type PietImage = WgpuImage;

/// A shortcut alias for [`hashbrown::hash_map::Entry`].
pub type Entry<'a, K, V> = hashbrown::hash_map::Entry<'a, K, V, BuildHasherDefault<AHasher>>;

/// A hasher builder that will create a fixed hasher.
#[derive(Debug, Clone, Default)]
pub struct FixedState;

impl std::hash::BuildHasher for FixedState {
    type Hasher = AHasher;

    #[inline]
    fn build_hasher(&self) -> AHasher {
        RandomState::with_seeds(
            0b10010101111011100000010011000100,
            0b00000011001001101011001001111000,
            0b11001111011010110111100010110101,
            0b00000100001111100011010011010101,
        )
        .build_hasher()
    }
}

/// A [`HashMap`][hashbrown::HashMap] implementing aHash, a high
/// speed keyed hashing algorithm intended for use in in-memory hashmaps.
///
/// aHash is designed for performance and is NOT cryptographically secure.
pub type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<AHasher>>;

/// A stable hash map implementing aHash, a high speed keyed hashing algorithm
/// intended for use in in-memory hashmaps.
///
/// Unlike [`HashMap`] this has an iteration order that only depends on the order
/// of insertions and deletions and not a random source.
///
/// aHash is designed for performance and is NOT cryptographically secure.
pub type StableHashMap<K, V> = hashbrown::HashMap<K, V, FixedState>;

/// A [`HashSet`][hashbrown::HashSet] implementing aHash, a high
/// speed keyed hashing algorithm intended for use in in-memory hashmaps.
///
/// aHash is designed for performance and is NOT cryptographically secure.
pub type HashSet<K> = hashbrown::HashSet<K, BuildHasherDefault<AHasher>>;

/// A stable hash set implementing aHash, a high speed keyed hashing algorithm
/// intended for use in in-memory hashmaps.
///
/// Unlike [`HashSet`] this has an iteration order that only depends on the order
/// of insertions and deletions and not a random source.
///
/// aHash is designed for performance and is NOT cryptographically secure.
pub type StableHashSet<K> = hashbrown::HashSet<K, FixedState>;

/// An ergonomic abbreviation for [`Default::default()`] to make initializing structs easier.
/// This is especially helpful when combined with ["struct update syntax"](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-instances-from-other-instances-with-struct-update-syntax).
/// ```
/// use bevy_utils::default;
///
/// #[derive(Default)]
/// struct Foo {
///   a: usize,
///   b: usize,
///   c: usize,
/// }
///
/// // Normally you would initialize a struct with defaults using "struct update syntax"
/// // combined with `Default::default()`. This example sets `Foo::bar` to 10 and the remaining
/// // values to their defaults.
/// let foo = Foo {
///   a: 10,
///   ..Default::default()
/// };
///
/// // But now you can do this, which is equivalent:
/// let foo = Foo {
///   a: 10,
///   ..default()
/// };
/// ```
#[inline]
pub fn default<T: Default>() -> T {
    std::default::Default::default()
}

/// A pre-hashed value of a specific type. Pre-hashing enables memoization of hashes that are expensive to compute.
/// It also enables faster [`PartialEq`] comparisons by short circuiting on hash equality.
/// See [`PassHash`] and [`PassHasher`] for a "pass through" [`BuildHasher`] and [`Hasher`] implementation
/// designed to work with [`Hashed`]
/// See [`PreHashMap`] for a hashmap pre-configured to use [`Hashed`] keys.
pub struct Hashed<V, H = FixedState> {
    hash: u64,
    value: V,
    marker: PhantomData<H>,
}

impl<V: Hash, H: BuildHasher + Default> Hashed<V, H> {
    /// Pre-hashes the given value using the [`BuildHasher`] configured in the [`Hashed`] type.
    pub fn new(value: V) -> Self {
        let builder = H::default();
        let mut hasher = builder.build_hasher();
        value.hash(&mut hasher);
        Self {
            hash: hasher.finish(),
            value,
            marker: PhantomData,
        }
    }

    /// The pre-computed hash.
    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl<V, H> Hash for Hashed<V, H> {
    #[inline]
    fn hash<R: Hasher>(&self, state: &mut R) {
        state.write_u64(self.hash);
    }
}

impl<V, H> Deref for Hashed<V, H> {
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: PartialEq, H> PartialEq for Hashed<V, H> {
    /// A fast impl of [`PartialEq`] that first checks that `other`'s pre-computed hash
    /// matches this value's pre-computed hash.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.value.eq(&other.value)
    }
}

impl<V: Debug, H> Debug for Hashed<V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hashed")
            .field("hash", &self.hash)
            .field("value", &self.value)
            .finish()
    }
}

impl<V: Clone, H> Clone for Hashed<V, H> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            value: self.value.clone(),
            marker: PhantomData,
        }
    }
}

impl<V: Eq, H> Eq for Hashed<V, H> {}

/// A [`BuildHasher`] that results in a [`PassHasher`].
#[derive(Default)]
pub struct PassHash;

impl BuildHasher for PassHash {
    type Hasher = PassHasher;

    fn build_hasher(&self) -> Self::Hasher {
        PassHasher::default()
    }
}

/// A no-op hash that only works on `u64`s. Will panic if attempting to
/// hash a type containing non-u64 fields.
#[derive(Debug, Default)]
pub struct PassHasher {
    hash: u64,
}

impl Hasher for PassHasher {
    fn write(&mut self, _bytes: &[u8]) {
        panic!("can only hash u64 using PassHasher");
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.hash = i;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

/// A [`HashMap`] pre-configured to use [`Hashed`] keys and [`PassHash`] passthrough hashing.
pub type PreHashMap<K, V> = hashbrown::HashMap<Hashed<K>, V, PassHash>;

/// Extension methods intended to add functionality to [`PreHashMap`].
pub trait PreHashMapExt<K, V> {
    /// Tries to get or insert the value for the given `key` using the pre-computed hash first.
    /// If the [`PreHashMap`] does not already contain the `key`, it will clone it and insert
    /// the value returned by `func`.
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V;
}

impl<K: Hash + Eq + PartialEq + Clone, V> PreHashMapExt<K, V> for PreHashMap<K, V> {
    #[inline]
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V {
        let entry = self
            .raw_entry_mut()
            .from_key_hashed_nocheck(key.hash(), key);
        match entry {
            RawEntryMut::Occupied(entry) => entry.into_mut(),
            RawEntryMut::Vacant(entry) => {
                let (_, value) = entry.insert_hashed_nocheck(key.hash(), key.clone(), func());
                value
            }
        }
    }
}

/// A [`BuildHasher`] that results in a [`EntityHasher`].
#[derive(Default)]
pub struct EntityHash;

impl BuildHasher for EntityHash {
    type Hasher = EntityHasher;

    fn build_hasher(&self) -> Self::Hasher {
        EntityHasher::default()
    }
}

/// A very fast hash that is only designed to work on generational indices
/// like `Entity`. It will panic if attempting to hash a type containing
/// non-u64 fields.
#[derive(Debug, Default)]
pub struct EntityHasher {
    hash: u64,
}

// This value comes from rustc-hash (also known as FxHasher) which in turn got
// it from Firefox. It is something like `u64::MAX / N` for an N that gives a
// value close to π and works well for distributing bits for hashing when using
// with a wrapping multiplication.
const FRAC_U64MAX_PI: u64 = 0x517cc1b727220a95;

impl Hasher for EntityHasher {
    fn write(&mut self, _bytes: &[u8]) {
        panic!("can only hash u64 using EntityHasher");
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        // Apparently hashbrown's hashmap uses the upper 7 bits for some SIMD
        // optimisation that uses those bits for binning. This hash function
        // was faster than i | (i << (64 - 7)) in the worst cases, and was
        // faster than PassHasher for all cases tested.
        self.hash = i | (i.wrapping_mul(FRAC_U64MAX_PI) << 32);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

/// A [`HashMap`] pre-configured to use [`EntityHash`] hashing.
pub type EntityHashMap<K, V> = hashbrown::HashMap<K, V, EntityHash>;

/// A [`HashSet`] pre-configured to use [`EntityHash`] hashing.
pub type EntityHashSet<T> = hashbrown::HashSet<T, EntityHash>;
