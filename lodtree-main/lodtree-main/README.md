[![Documentation](https://docs.rs/lodtree/badge.svg)](https://docs.rs/lodtree)

# LodTree
LodTree, a simple tree data structure for doing chunk-based level of detail.

## Goals
The aim of this crate is to provide a generic, easy to use tree data structure that can be used to make Quadtrees, Octrees and more for chunked level of detail.

Internally, the tree tries to keep as much memory allocated, to avoid the cost of heap allocation, and stores the actual chunks data seperate from the tree data.
 
This does come at a cost. Mainly, only the chunks that are going to be added and their locations can be retreived as a slice, although for most (procedural) terrain implementations.

## Non-goals
Be a general-usage tree data structure for storing items at specific locations.

## Features
 - Provides sets of chunks that need some action performed on them
 - Tries to avoid memory (re)allocations and moves
 - Stores chunks themselves in a contiguous array
 - Uses an internal chunk cache to allow reusing chunks at a memory tradeoff
 - Provides some extra iterators for finding chunks in certain bounds

### Examples:
 - [rayon](examples/rayon.rs): shows how to use the tree with rayon to generate new chunks in parallel.
 - [glium](examples/glium.rs): shows how a basic drawing setup would work, with glium to do the drawing.

## Usage:
Import the crate
```rust
use lodtree::*;
use lodtree::coords::OctVec; // or Quadvec if you're making an octree
```

The tree is it's own struct, and accepts a chunk (anything that implements Sized) and the lod vector (Anything that implements the LodVec trait).
```rust
let mut tree = Tree::<Chunk, OctVec>::new();
```

If you want to update chunks due to the camera being moved, you can check if it's needed with prepare_update.
It takes in 3 parameters.

Targets: where to generate the most detail around.

The given LodVec implementations (OctVec and QuadVec) take in 4 and 3 arguments respectively.
The first 3/2 are the position in the tree, which is dependant on the lod level.
and the last parameter is the lod level. No lods smaller than this will be generated for this target.

Detail: The amount of detail for the targets
The default implementation defines this as the amount of chunks at the target lod level surrounding the target chunk.

Chunk creator:
Internally a buffer for new chunks is filled, and this function is called to create the new chunk.
It takes in the LodVec of the position of the chunk.
```rust
let needs_updating = tree.prepare_update(
	&[OctVec(8, 8, 8, 8)], // the target positions to generate the lod around
	4, // amount of detail
	|pos| Chunk {} // and the function to construct the chunk with
);
```

Now, the tree is ready for an update, so now we'll want to do something with that.
First, we want to process all chunks that are going to be added.
This is the only thing the API exposes as a slice, so we can nicely iterate over that in parallel with rayon.
```rust
tree.get_chunks_to_add_slice_mut()
	.iter_mut() // or par_iter_mut if you're using rayon
	.for_each(|(position, chunk)| {

		// and run expensive init, probably does something with procedural generation
		chunk.expensive_init(*position);
	});
```

Next, we'll also want to change the visibility of some chunks so they don't overlap with higher detail lods.
```rust
// and make all chunks visible or not
for i in 0..tree.get_num_chunks_to_activate() {
	tree.get_chunk_to_activate_mut(i).set_visible(true);
}

for i in 0..tree.get_num_chunks_to_deactivate() {
	tree.get_chunk_to_deactivate_mut(i).set_visible(false);
}
```
We'll probably also want to do some cleanup with chunks that are removed.
```rust
for i in 0..tree.get_num_chunks_to_remove() {
	tree.get_chunk_to_remove_mut(i).cleanup();
} 
```
And finally, actually update the tree with the new chunks.
Note that it's likely needed to do the prepare_update and do_update cycle a number of times before no new chunks need to be added, as the tree only adds one lod level at a time.
```rust
tree.do_update();
```
But we're not done yet!
After this step there's a number of chunks that are removed from the cache, and will not be added back into the tree
We'll want to clean those up now
```rust
for (position, chunk) in tree.get_chunks_to_delete_slice_mut().iter_mut() {
	chunk.true_cleanup();
}

// and finally, complete the entire update
tree.complete_update();
```

## Roadmap
### 0.2.0:
 - Support getting "edited" chunks, via also passing along a region in which chunks would be edited. NEEDS DOCS AND TESTING
 - caching DONE
 - iterators for all chunk data accessing methods. DONE
 - getting a chunk by position DONE
 - swap L and C, so the key (position) is before the chunk, which is consistent with other key-value datatypes in rust
### 0.3.0:
 - Replace the tree in favour of a list to generate all nodes up front, then use a hashmap for storage
 - this keeps everything in one map, with optional removal from that map. Also simplifies everything as there's only "add", "add from cache", "remove to cache" and "remove entirely" instead of the current add, add from cache, remove, merge, subdivide, and delete
 - no-std (although alloc will be required here)

## License
Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.