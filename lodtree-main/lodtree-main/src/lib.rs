//! # LodTree
//! LodTree, a simple tree data structure for doing chunk-based level of detail.
//!
//! # Goals
//! The aim of this crate is to provide a generic, easy to use tree data structure that can be used to make Lod Quadtrees, Octrees and more.
//!
//! Internally, the tree tries to keep as much memory allocated, to avoid the cost of heap allocation, and stores the actual chunks data seperate from the tree data.
//!  
//! This does come at a cost, mainly, only the chunks that are going to be added and their locations can be retreived as a slice, although for most (procedural) terrain implementations
//! making new chunks and editing them will be the highest cost to do, so that shouldn't be the biggest issue.
//!
//! # Usage:
//! Import the crate
//! ```rust
//! use lodtree::*;
//! use lodtree::coords::OctVec; // or QuadVec if you're making an octree
//! ```
//!
//! The tree is it's own struct, and accepts a chunk (anything that implements Sized) and the lod vector (Anything that implements the LodVec trait).
//! ```rust
//! # use lodtree::*;
//! # use lodtree::coords::OctVec;
//! # struct Chunk {}
//! let mut tree = Tree::<Chunk, OctVec>::new();
//! ```
//!
//! If you want to update chunks due to the camera being moved, you can check if it's needed with prepare_update.
//! It takes in 3 parameters.
//!
//! Targets: where to generate the most detail around.
//!
//! The given LodVec implementations (OctVec and QuadVec) take in 4 and 3 arguments respectively.
//! The first 3/2 are the position in the tree, which is dependant on the lod level.
//! and the last parameter is the lod level. No lods smaller than this will be generated for this target.
//!
//! Detail: The amount of detail for the targets.
//! The default implementation defines this as the amount of chunks at the target lod level surrounding the target chunk.
//!
//! Chunk creator:
//! Internally a buffer for new chunks is filled, and this function is called to create the new chunk.
//! It takes in the LodVec of the position of the chunk.
//! ```rust
//! # use lodtree::*;
//! # use lodtree::coords::OctVec;
//! # struct Chunk {}
//! # let mut tree = Tree::<Chunk, OctVec>::new(64);
//! let needs_updating = tree.prepare_update(
//! 	&[OctVec::new(8, 8, 8, 8)], // the target positions to generate the lod around
//! 	4, // amount of detail
//! 	|pos| Chunk {} // and the function to construct the chunk with
//!                    // NOTE: this is only called for completely new chunks, not the ones loaded from the chunk cache!
//! );
//! ```
//!
//! Now, the tree is ready for an update, so now we'll want to do something with that.
//! First, we want to process all chunks that are going to be added.
//! This is the only thing the API exposes as a slice, so we can nicely iterate over that in parallel with rayon.
//! ```rust
//! # use lodtree::*;
//! # use lodtree::coords::QuadVec;
//! # struct Chunk {}
//! # impl Chunk {
//! #     fn expensive_init(&mut self, pos: QuadVec) {}
//! # }
//! # let mut tree = Tree::<Chunk, QuadVec>::new();
//! tree.get_chunks_to_add_slice_mut()
//! 	.iter_mut() // or par_iter_mut() if you're using rayon
//! 	.for_each(|(position, chunk)| {
//!
//! 		// and run expensive init, probably does something with procedural generation
//! 		chunk.expensive_init(*position);
//! 	});
//! ```
//!
//! Next, we'll also want to change the visibility of some chunks so they don't overlap with higher detail lods.
//! ```rust
//! # use lodtree::*;
//! # use lodtree::coords::QuadVec;
//! # struct Chunk {}
//! # impl Chunk {
//! #     fn set_visible(&mut self, v: bool) {}
//! # }
//! # let mut tree = Tree::<Chunk, QuadVec>::new();
//! // and make all chunks visible or not
//! for chunk in tree.iter_chunks_to_activate_mut() {
//! 	chunk.set_visible(true);
//! }
//!
//! for chunk in tree.iter_chunks_to_deactivate_mut() {
//! 	chunk.set_visible(false);
//! }
//! ```
//! We'll probably also want to do some cleanup with chunks that are removed.
//! Note that these are not permanently removed, but are instead added to a cache, so it's possible that these chunks come back in the tree
//! ```rust
//! # use lodtree::*;
//! # use lodtree::coords::QuadVec;
//! # struct Chunk {}
//! # impl Chunk {
//! #     fn cleanup(&mut self) {}
//! # }
//! # let mut tree = Tree::<Chunk, QuadVec>::new();
//! for chunk in tree.iter_chunks_to_remove_mut() {
//! 	chunk.cleanup();
//! }
//! ```
//! And finally, actually update the tree with the new chunks.
//! Note that it's likely needed to do the prepare_update and do_update cycle a number of times before no new chunks need to be added, as the tree only adds one lod level at a time.
//! ```rust
//! # use lodtree::*;
//! # use lodtree::coords::QuadVec;
//! # struct Chunk {}
//! # let mut tree = Tree::<Chunk, QuadVec>::new();
//! tree.do_update();
//! ```
//! But we're not done yet!
//! After this step there's a number of chunks that are removed from the cache, and will not be added back into the tree
//! We'll want to clean those up now
//! ```rust
//! # use lodtree::*;
//! # use lodtree::coords::QuadVec;
//! # struct Chunk {}
//! # impl Chunk {
//! #     fn true_cleanup(&mut self) {}
//! # }
//! # let mut tree = Tree::<Chunk, QuadVec>::new();
//! for (position, chunk) in tree.get_chunks_to_delete_slice_mut().iter_mut() { // there's also an iterator for just chunks here
//! 	chunk.true_cleanup();
//! }
//!
//! // and finally, complete the entire update
//! tree.complete_update();
//! ```
//! # Caching
//! When making a new tree, you can specify an internal cache size as follows:
//! ```rust
//! # use lodtree::*;
//! # use lodtree::coords::QuadVec;
//! # struct Chunk {}
//! let cache_size = 64;
//! let mut tree = Tree::<Chunk, QuadVec>::new(cache_size);
//! ```
//! When a chunk is removed from the tree, it will be put in the cache.
//! When a new chunk is then added to the tree, it's fetched from the cache when possible.
//! This should help avoid needing to regenerate all new chunks, as they are fetched from the internal cache.
//!
//! Caching is most effective with a larger cache size as well as the target position moving around in roughly the same area.
//! Of course, it comes at a memory tradeoff, as it will keep all chunks in the cache stored in memory
//!
//! # Chunk groups
//! There's several groups of chunks that can be accessed inside the tree.
//! - `chunks`: All chunks currently stored inside the tree
//! - `chunks_to_add`: Chunks that will be added after the next `tree.do_update();`
//! - `chunks_to_deactivate`: Chunks that have subdivided and thus need to be invisible.
//! - `chunks_to_activate`: Chunks that were previously subdivided, but are now going to be leaf nodes. This means they should be visible again
//! - `chunks_to_remove`: Chunks that will be removed from the tree after the next `tree.do_update()`. Note that these can be put in the chunk cache and appear in `chunks_to_add` at a later point
//! - `chunks_to_delete`: Chunks that are permanently removed from the tree, as they were removed from the tree itself, and will now also be removed from the chunk cache
//!
//! Cached chunks are also stored seperate from the tree, inside a HashMap. These can't be accessed.
//!
//! # Iterators
//! Iterators are provided for each chunk group, in the flavour of chunks, mutable chunks, chunk and positions and mutable chunk and positions.
//!
//! # Getters
//! Getters are also given for all chunk groups, in the flavor of get a chunk, get a mutable chunk, get a mutable pointer to a chunk and get the position of a chunk.

pub mod coords;
pub mod iter;
pub mod traits;
pub mod tree;

pub use crate::iter::*;
pub use crate::traits::*;
pub use crate::tree::*;
